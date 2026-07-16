use super::embed::{embed_text, EMBED_DIM};
use super::{RagChunk, RagStore};
use serde::Deserialize;
use serde_json::json;
use tempoforge_common::{AppError, AppResult};
use tracing::{info, warn};
use uuid::Uuid;

fn point_id(id: &str) -> String {
    Uuid::new_v5(&Uuid::NAMESPACE_OID, id.as_bytes()).to_string()
}

#[derive(Clone)]
pub struct QdrantHttpRag {
    http: reqwest::Client,
    base_url: String,
}

impl QdrantHttpRag {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    pub async fn ensure_collection(&self, collection: &str) -> AppResult<()> {
        let url = format!("{}/collections/{collection}", self.base_url);
        let exists = self.http.get(&url).send().await;
        if let Ok(resp) = exists {
            if resp.status().is_success() {
                return Ok(());
            }
        }

        let create_url = format!("{}/collections/{collection}", self.base_url);
        let body = json!({
            "vectors": {
                "size": EMBED_DIM,
                "distance": "Cosine"
            }
        });
        let resp = self
            .http
            .put(create_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Upstream(format!("qdrant create collection failed: {e}")))?;

        if resp.status().is_success() || resp.status().as_u16() == 409 {
            info!(collection, "qdrant collection ready");
            Ok(())
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(AppError::Upstream(format!(
                "qdrant create collection error: {text}"
            )))
        }
    }

    pub async fn seed_defaults(&self) -> AppResult<()> {
        self.ensure_collection("tempo").await?;
        self.ensure_collection("security").await?;
        let tempo_docs = vec![
            RagChunk {
                id: "tempo-rpc".into(),
                source: "docs/tempo/rpc".into(),
                text: "Tempo RPC: mainnet https://rpc.tempo.xyz, testnet https://rpc.moderato.tempo.xyz. eth_getBalance returns a placeholder; use TIP-20 balanceOf. Fees paid in TIP-20 stablecoins. Supports tx type 0x54.".into(),
                score: 1.0,
            },
            RagChunk {
                id: "tempo-fees".into(),
                source: "docs/tempo/fees".into(),
                text: "Tempo has no native gas token. eth_estimateGas accounts for TIP-20 fee token balances. Use pathUSD and related stablecoins for fees on Moderato testnet.".into(),
                score: 1.0,
            },
        ];
        let security_docs = vec![
            RagChunk {
                id: "reentrancy".into(),
                source: "docs/security/reentrancy".into(),
                text: "Reentrancy: follow checks-effects-interactions. Use OpenZeppelin ReentrancyGuard. Prefer pull payments over push.".into(),
                score: 1.0,
            },
            RagChunk {
                id: "access-control".into(),
                source: "docs/security/access-control".into(),
                text: "Access control: prefer Ownable2Step or AccessControl roles. Never use tx.origin for authorization.".into(),
                score: 1.0,
            },
        ];
        self.upsert("tempo", tempo_docs).await?;
        self.upsert("security", security_docs).await?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    result: Option<Vec<SearchHit>>,
}

#[derive(Debug, Deserialize)]
struct SearchHit {
    score: f32,
    payload: Option<SearchPayload>,
}

#[derive(Debug, Deserialize)]
struct SearchPayload {
    id: Option<String>,
    source: Option<String>,
    text: Option<String>,
}

#[async_trait::async_trait]
impl RagStore for QdrantHttpRag {
    async fn upsert(&self, collection: &str, chunks: Vec<RagChunk>) -> AppResult<()> {
        self.ensure_collection(collection).await?;
        let points: Vec<serde_json::Value> = chunks
            .into_iter()
            .map(|chunk| {
                let vector = embed_text(&chunk.text);
                json!({
                    "id": point_id(&chunk.id),
                    "vector": vector,
                    "payload": {
                        "id": chunk.id,
                        "source": chunk.source,
                        "text": chunk.text,
                    }
                })
            })
            .collect();

        let url = format!("{}/collections/{collection}/points?wait=true", self.base_url);
        let resp = self
            .http
            .put(url)
            .json(&json!({ "points": points }))
            .send()
            .await
            .map_err(|e| AppError::Upstream(format!("qdrant upsert failed: {e}")))?;

        if resp.status().is_success() {
            Ok(())
        } else {
            let text = resp.text().await.unwrap_or_default();
            Err(AppError::Upstream(format!("qdrant upsert error: {text}")))
        }
    }

    async fn search(&self, collection: &str, query: &str, limit: u64) -> AppResult<Vec<RagChunk>> {
        if let Err(err) = self.ensure_collection(collection).await {
            warn!(error = %err, "qdrant unavailable during search");
            return Err(err);
        }

        let url = format!("{}/collections/{collection}/points/search", self.base_url);
        let body = json!({
            "vector": embed_text(query),
            "limit": limit,
            "with_payload": true
        });

        let resp = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Upstream(format!("qdrant search failed: {e}")))?;

        if !resp.status().is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::Upstream(format!("qdrant search error: {text}")));
        }

        let parsed: SearchResponse = resp
            .json()
            .await
            .map_err(|e| AppError::Upstream(format!("qdrant search decode failed: {e}")))?;

        Ok(parsed
            .result
            .unwrap_or_default()
            .into_iter()
            .filter_map(|hit| {
                let payload = hit.payload?;
                Some(RagChunk {
                    id: payload.id.unwrap_or_default(),
                    source: payload.source.unwrap_or_default(),
                    text: payload.text.unwrap_or_default(),
                    score: hit.score,
                })
            })
            .collect())
    }
}
