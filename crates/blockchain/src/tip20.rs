//! TIP-20 helpers. Tempo uses TIP-20 stablecoins for balances and fees.

use crate::client::TempoClient;
use serde_json::json;
use tempoforge_common::AppResult;

/// ERC-20 / TIP-20 `balanceOf(address)` selector.
const BALANCE_OF_SELECTOR: &str = "0x70a08231";

pub async fn tip20_balance_of(
    client: &TempoClient,
    token: &str,
    holder: &str,
) -> AppResult<String> {
    let holder = holder.trim_start_matches("0x").to_lowercase();
    if holder.len() != 40 {
        return Err(tempoforge_common::AppError::Validation(
            "holder must be a 20-byte hex address".into(),
        ));
    }
    let data = format!("{BALANCE_OF_SELECTOR}{}{holder}", "0".repeat(24));
    client
        .call(json!({
            "to": token,
            "data": data,
        }))
        .await
}

pub fn explain_balance_semantics() -> &'static str {
    "Tempo has no native gas token. eth_getBalance returns a constant placeholder. \
     Query TIP-20 balanceOf for real balances and fee token balances."
}
