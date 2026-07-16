//! Explorer services for blocks, transactions, addresses, and contracts.

use serde::Serialize;
use serde_json::Value;
use tempoforge_blockchain::TempoClient;
use tempoforge_common::{AppError, AppResult};
use tempoforge_blockchain::tip20::explain_balance_semantics;

#[derive(Debug, Clone, Serialize)]
pub struct AddressView {
    pub address: String,
    pub code_hash_present: bool,
    pub is_contract: bool,
    pub balance_note: String,
    pub tip20_balances: Vec<Tip20Balance>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Tip20Balance {
    pub token: String,
    pub raw: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransactionView {
    pub hash: String,
    pub transaction: Option<Value>,
    pub receipt: Option<Value>,
    pub decoded_events: Vec<DecodedEvent>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecodedEvent {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub summary: String,
}

#[derive(Clone)]
pub struct ExplorerService {
    client: TempoClient,
}

impl ExplorerService {
    pub fn new(client: TempoClient) -> Self {
        Self { client }
    }

    pub async fn get_transaction(&self, hash: &str) -> AppResult<TransactionView> {
        if !hash.starts_with("0x") || hash.len() != 66 {
            return Err(AppError::Validation(
                "transaction hash must be 32-byte hex with 0x prefix".into(),
            ));
        }

        let tx = self.client.get_transaction(hash).await?;
        let receipt = self.client.get_transaction_receipt(hash).await?;
        let receipt_value = receipt
            .as_ref()
            .map(|r| serde_json::to_value(r).unwrap_or(Value::Null));

        let decoded_events = receipt
            .as_ref()
            .map(|r| {
                r.logs
                    .iter()
                    .filter_map(decode_log)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        Ok(TransactionView {
            hash: hash.to_string(),
            transaction: tx,
            receipt: receipt_value,
            decoded_events,
        })
    }

    pub async fn get_address(&self, address: &str, tip20_tokens: &[String]) -> AppResult<AddressView> {
        validate_address(address)?;
        let code = self.client.get_code(address).await?;
        let is_contract = code != "0x" && code != "0x0";

        let mut tip20_balances = Vec::new();
        for token in tip20_tokens {
            if let Ok(raw) =
                tempoforge_blockchain::tip20::tip20_balance_of(&self.client, token, address).await
            {
                tip20_balances.push(Tip20Balance {
                    token: token.clone(),
                    raw,
                });
            }
        }

        Ok(AddressView {
            address: address.to_string(),
            code_hash_present: is_contract,
            is_contract,
            balance_note: explain_balance_semantics().to_string(),
            tip20_balances,
        })
    }

    pub async fn get_block(&self, number: u64) -> AppResult<Value> {
        let block = self.client.get_block_by_number(number, true).await?;
        Ok(serde_json::to_value(block).unwrap_or(Value::Null))
    }

    pub async fn latest_block_number(&self) -> AppResult<u64> {
        self.client.block_number().await
    }
}

fn validate_address(address: &str) -> AppResult<()> {
    let hex = address.trim_start_matches("0x");
    if hex.len() != 40 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(AppError::Validation("invalid address".into()));
    }
    Ok(())
}

fn decode_log(log: &Value) -> Option<DecodedEvent> {
    let address = log.get("address")?.as_str()?.to_string();
    let topics = log
        .get("topics")?
        .as_array()?
        .iter()
        .filter_map(|t| t.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();
    let data = log.get("data")?.as_str()?.to_string();
    let summary = match topics.first().map(String::as_str) {
        Some("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef") => {
            "ERC20/TIP-20 Transfer".to_string()
        }
        Some("0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925") => {
            "ERC20/TIP-20 Approval".to_string()
        }
        Some(topic) => format!("Event topic {}", &topic[..10.min(topic.len())]),
        None => "Anonymous log".into(),
    };

    Some(DecodedEvent {
        address,
        topics,
        data,
        summary,
    })
}
