mod embed;
mod qdrant;

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tempoforge_common::AppResult;
use tracing::warn;

pub use embed::embed_text;
pub use qdrant::QdrantHttpRag;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagChunk {
    pub id: String,
    pub source: String,
    pub text: String,
    pub score: f32,
}

/// Retrieval port. Implementations may use Qdrant or an in-memory store.
#[async_trait::async_trait]
pub trait RagStore: Send + Sync {
    async fn upsert(&self, collection: &str, chunks: Vec<RagChunk>) -> AppResult<()>;
    async fn search(&self, collection: &str, query: &str, limit: u64) -> AppResult<Vec<RagChunk>>;
}

/// Deterministic keyword RAG for tests and local boot without Qdrant.
#[derive(Default)]
pub struct InMemoryRag {
    docs: tokio::sync::RwLock<Vec<RagChunk>>,
}

impl InMemoryRag {
    pub fn with_seed_docs() -> Self {
        let seed = vec![
            RagChunk {
                id: "tempo-rpc".into(),
                source: "docs/tempo/rpc".into(),
                text: "Tempo RPC: mainnet https://rpc.tempo.xyz, testnet https://rpc.moderato.tempo.xyz. eth_getBalance returns a placeholder; use TIP-20 balanceOf. Fees paid in TIP-20 stablecoins. Supports tx type 0x54.".into(),
                score: 1.0,
            },
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
        Self {
            docs: tokio::sync::RwLock::new(seed),
        }
    }
}

#[async_trait::async_trait]
impl RagStore for InMemoryRag {
    async fn upsert(&self, _collection: &str, chunks: Vec<RagChunk>) -> AppResult<()> {
        self.docs.write().await.extend(chunks);
        Ok(())
    }

    async fn search(&self, _collection: &str, query: &str, limit: u64) -> AppResult<Vec<RagChunk>> {
        let q = query.to_lowercase();
        let mut scored: Vec<RagChunk> = self
            .docs
            .read()
            .await
            .iter()
            .filter_map(|doc| {
                let text = doc.text.to_lowercase();
                let hits = q
                    .split_whitespace()
                    .filter(|w| w.len() > 2 && text.contains(w))
                    .count();
                if hits == 0 {
                    return None;
                }
                let mut clone = doc.clone();
                clone.score = hits as f32;
                Some(clone)
            })
            .collect();
        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit as usize);
        Ok(scored)
    }
}

/// Prefer Qdrant when reachable; otherwise fall back to in-memory seed docs.
pub async fn build_rag_store(qdrant_url: Option<String>) -> Arc<dyn RagStore> {
    let Some(url) = qdrant_url.filter(|u| !u.is_empty()) else {
        return Arc::new(InMemoryRag::with_seed_docs());
    };

    let qdrant = QdrantHttpRag::new(url);
    match qdrant.seed_defaults().await {
        Ok(()) => {
            tracing::info!("using Qdrant RAG store");
            Arc::new(qdrant)
        }
        Err(err) => {
            warn!(error = %err, "Qdrant unavailable — falling back to in-memory RAG");
            Arc::new(InMemoryRag::with_seed_docs())
        }
    }
}
