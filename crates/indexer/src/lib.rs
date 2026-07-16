//! Block indexer for Tempo. Polls RPC and emits normalized events.

use chrono::Utc;
use serde::Serialize;
use tempoforge_blockchain::TempoClient;
use tempoforge_common::AppResult;
use tracing::info;

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
    cursor: u64,
}

impl Indexer {
    pub fn new(client: TempoClient, start_block: u64) -> Self {
        Self {
            client,
            cursor: start_block,
        }
    }

    pub fn cursor(&self) -> u64 {
        self.cursor
    }

    pub async fn tick(&mut self) -> AppResult<Option<IndexedBlock>> {
        let tip = self.client.block_number().await?;
        if self.cursor > tip {
            return Ok(None);
        }

        let block = self.client.get_block_by_number(self.cursor, false).await?;
        let tx_count = block.transactions.len();
        let indexed = IndexedBlock {
            number: self.cursor,
            hash: block.hash,
            tx_count,
            indexed_at: Utc::now(),
        };
        info!(block = indexed.number, txs = tx_count, "indexed block");
        self.cursor += 1;
        Ok(Some(indexed))
    }
}
