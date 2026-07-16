use crate::types::{Block, ForkSchedule, Network, TransactionReceipt};
use serde::Deserialize;
use serde_json::{Value, json};
use tempoforge_common::{AppError, AppResult};
use tracing::debug;

#[derive(Clone)]
pub struct TempoClient {
    http: reqwest::Client,
    rpc_url: String,
    network: Network,
}

impl TempoClient {
    pub fn new(network: Network) -> Self {
        Self {
            http: reqwest::Client::new(),
            rpc_url: network.rpc_url(),
            network,
        }
    }

    pub fn from_env() -> Self {
        Self::new(Network::from_env_default())
    }

    pub fn network(&self) -> Network {
        self.network
    }

    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    pub async fn chain_id(&self) -> AppResult<u64> {
        let hex: String = self.rpc("eth_chainId", json!([])).await?;
        parse_hex_u64(&hex)
    }

    pub async fn block_number(&self) -> AppResult<u64> {
        let hex: String = self.rpc("eth_blockNumber", json!([])).await?;
        parse_hex_u64(&hex)
    }

    pub async fn get_block_by_number(&self, number: u64, full_txs: bool) -> AppResult<Block> {
        let num = format!("0x{number:x}");
        self.rpc("eth_getBlockByNumber", json!([num, full_txs]))
            .await
    }

    pub async fn get_transaction_receipt(&self, hash: &str) -> AppResult<Option<TransactionReceipt>> {
        let value: Option<TransactionReceipt> = self
            .rpc("eth_getTransactionReceipt", json!([hash]))
            .await?;
        Ok(value)
    }

    pub async fn get_transaction(&self, hash: &str) -> AppResult<Option<Value>> {
        self.rpc("eth_getTransactionByHash", json!([hash])).await
    }

    pub async fn get_code(&self, address: &str) -> AppResult<String> {
        self.rpc("eth_getCode", json!([address, "latest"])).await
    }

    pub async fn get_logs(&self, filter: Value) -> AppResult<Vec<Value>> {
        self.rpc("eth_getLogs", json!([filter])).await
    }

    pub async fn call(&self, tx: Value) -> AppResult<String> {
        self.rpc("eth_call", json!([tx, "latest"])).await
    }

    pub async fn fork_schedule(&self) -> AppResult<ForkSchedule> {
        self.rpc("tempo_forkSchedule", json!([])).await
    }

    pub async fn fund_address(&self, address: &str) -> AppResult<Vec<String>> {
        if self.network != Network::Testnet {
            return Err(AppError::BadRequest(
                "tempo_fundAddress is only available on faucet-enabled testnet".into(),
            ));
        }
        self.rpc("tempo_fundAddress", json!([address])).await
    }

    /// Note: On Tempo, eth_getBalance returns a constant placeholder.
    pub async fn eth_get_balance_raw(&self, address: &str) -> AppResult<String> {
        self.rpc("eth_getBalance", json!([address, "latest"])).await
    }

    async fn rpc<T: for<'de> Deserialize<'de>>(&self, method: &str, params: Value) -> AppResult<T> {
        let body = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": method,
            "params": params,
        });

        debug!(method, url = %self.rpc_url, "tempo rpc call");

        let response = self
            .http
            .post(&self.rpc_url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::Upstream(format!("RPC transport error: {e}")))?;

        if !response.status().is_success() {
            return Err(AppError::Upstream(format!(
                "RPC HTTP {}",
                response.status()
            )));
        }

        let payload: RpcResponse<T> = response
            .json()
            .await
            .map_err(|e| AppError::Upstream(format!("RPC decode error: {e}")))?;

        if let Some(err) = payload.error {
            return Err(AppError::Upstream(format!(
                "RPC error {}: {}",
                err.code, err.message
            )));
        }

        payload
            .result
            .ok_or_else(|| AppError::Upstream("RPC response missing result".into()))
    }
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

fn parse_hex_u64(hex: &str) -> AppResult<u64> {
    let trimmed = hex.trim_start_matches("0x");
    u64::from_str_radix(trimmed, 16)
        .map_err(|e| AppError::Upstream(format!("invalid hex quantity {hex}: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex() {
        assert_eq!(parse_hex_u64("0xa").unwrap(), 10);
    }
}
