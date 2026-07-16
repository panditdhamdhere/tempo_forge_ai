//! Block indexer for Tempo. Polls RPC and persists normalized events.

use chrono::Utc;
use serde::Serialize;
use sqlx::PgPool;
use tempoforge_blockchain::{Network, TempoClient};
use tempoforge_common::{AppError, AppResult};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize)]
pub struct IndexedBlock {
    pub number: u64,
    pub hash: Option<String>,
    pub tx_count: usize,
    pub indexed_at: chrono::DateTime<Utc>,
}

#[derive(Clone)]
pub struct Indexer {
    client: TempoClient,
    db: PgPool,
    network: String,
    cursor: u64,
    batch_size: u64,
}

impl Indexer {
    pub async fn bootstrap(client: TempoClient, db: PgPool, start_block: Option<u64>) -> AppResult<Self> {
        let network = format!("{:?}", client.network()).to_lowercase();
        let stored = sqlx::query_scalar::<_, i64>(
            "SELECT cursor_block FROM indexer_state WHERE network = $1",
        )
        .bind(&network)
        .fetch_optional(&db)
        .await
        .map_err(|e| AppError::Internal(format!("indexer state read failed: {e}")))?;

        let cursor = match (stored, start_block) {
            (Some(v), _) => v.max(0) as u64,
            (None, Some(start)) => start,
            (None, None) => {
                let tip = client.block_number().await.unwrap_or(0);
                tip.saturating_sub(25)
            }
        };

        sqlx::query(
            r#"
            INSERT INTO indexer_state (network, cursor_block, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (network) DO UPDATE SET updated_at = NOW()
            "#,
        )
        .bind(&network)
        .bind(cursor as i64)
        .execute(&db)
        .await
        .map_err(|e| AppError::Internal(format!("indexer state init failed: {e}")))?;

        Ok(Self {
            client,
            db,
            network,
            cursor,
            batch_size: 10,
        })
    }

    pub fn cursor(&self) -> u64 {
        self.cursor
    }

    pub fn network(&self) -> &str {
        &self.network
    }

    pub async fn tick(&mut self) -> AppResult<Vec<IndexedBlock>> {
        let tip = self.client.block_number().await?;
        if self.cursor > tip {
            return Ok(vec![]);
        }

        let end = (self.cursor + self.batch_size - 1).min(tip);
        let mut out = Vec::new();

        for number in self.cursor..=end {
            match self.index_one(number).await {
                Ok(block) => out.push(block),
                Err(err) => {
                    warn!(block = number, error = %err, "failed to index block");
                    break;
                }
            }
            self.cursor = number + 1;
        }

        sqlx::query(
            r#"
            INSERT INTO indexer_state (network, cursor_block, updated_at)
            VALUES ($1, $2, NOW())
            ON CONFLICT (network) DO UPDATE
              SET cursor_block = EXCLUDED.cursor_block, updated_at = NOW()
            "#,
        )
        .bind(&self.network)
        .bind(self.cursor as i64)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal(format!("indexer cursor persist failed: {e}")))?;

        Ok(out)
    }

    async fn index_one(&self, number: u64) -> AppResult<IndexedBlock> {
        let block = self.client.get_block_by_number(number, true).await?;
        let tx_count = block.transactions.len();
        let hash = block.hash.clone();

        sqlx::query(
            r#"
            INSERT INTO indexed_blocks (network, number, hash, tx_count, indexed_at)
            VALUES ($1, $2, $3, $4, NOW())
            ON CONFLICT (network, number) DO UPDATE
              SET hash = EXCLUDED.hash, tx_count = EXCLUDED.tx_count, indexed_at = NOW()
            "#,
        )
        .bind(&self.network)
        .bind(number as i64)
        .bind(&hash)
        .bind(tx_count as i32)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal(format!("indexed_blocks upsert failed: {e}")))?;

        for tx in &block.transactions {
            if let Some(tx_hash) = tx.get("hash").and_then(|v| v.as_str()) {
                let from = tx.get("from").and_then(|v| v.as_str());
                let to = tx.get("to").and_then(|v| v.as_str());
                sqlx::query(
                    r#"
                    INSERT INTO transactions (network, hash, block_number, from_address, to_address, status, raw)
                    VALUES ($1::network_env, $2, $3, $4, $5, 'indexed', $6)
                    ON CONFLICT (network, hash) DO UPDATE
                      SET block_number = EXCLUDED.block_number,
                          from_address = EXCLUDED.from_address,
                          to_address = EXCLUDED.to_address,
                          raw = EXCLUDED.raw,
                          indexed_at = NOW()
                    "#,
                )
                .bind(&self.network)
                .bind(tx_hash)
                .bind(number as i64)
                .bind(from)
                .bind(to)
                .bind(tx)
                .execute(&self.db)
                .await
                .map_err(|e| AppError::Internal(format!("transaction upsert failed: {e}")))?;
            }
        }

        info!(network = %self.network, block = number, txs = tx_count, "indexed block");
        Ok(IndexedBlock {
            number,
            hash,
            tx_count,
            indexed_at: Utc::now(),
        })
    }
}

pub fn network_from_env() -> Network {
    Network::from_env_default()
}
