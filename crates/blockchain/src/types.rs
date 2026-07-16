use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Network {
    Mainnet,
    Testnet,
    Local,
}

impl Network {
    pub fn from_env_default() -> Self {
        match std::env::var("TEMPO_DEFAULT_NETWORK")
            .unwrap_or_else(|_| "testnet".into())
            .to_lowercase()
            .as_str()
        {
            "mainnet" => Self::Mainnet,
            "local" => Self::Local,
            _ => Self::Testnet,
        }
    }

    pub fn chain_id(self) -> u64 {
        match self {
            Self::Mainnet => env_u64("TEMPO_MAINNET_CHAIN_ID", 4242),
            Self::Testnet => env_u64("TEMPO_TESTNET_CHAIN_ID", 42431),
            Self::Local => env_u64("TEMPO_LOCAL_CHAIN_ID", 31337),
        }
    }

    pub fn rpc_url(self) -> String {
        match self {
            Self::Mainnet => std::env::var("TEMPO_MAINNET_RPC")
                .unwrap_or_else(|_| "https://rpc.tempo.xyz".into()),
            Self::Testnet => std::env::var("TEMPO_TESTNET_RPC")
                .unwrap_or_else(|_| "https://rpc.moderato.tempo.xyz".into()),
            Self::Local => std::env::var("TEMPO_LOCAL_RPC")
                .unwrap_or_else(|_| "http://127.0.0.1:8545".into()),
        }
    }
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[derive(Debug, Clone)]
pub struct TempoNetworks;

impl TempoNetworks {
    pub fn all() -> [Network; 3] {
        [Network::Mainnet, Network::Testnet, Network::Local]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    pub transaction_hash: String,
    pub block_number: Option<String>,
    pub status: Option<String>,
    pub gas_used: Option<String>,
    pub contract_address: Option<String>,
    pub logs: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Block {
    pub number: Option<String>,
    pub hash: Option<String>,
    pub timestamp: Option<String>,
    pub transactions: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkSchedule {
    pub schedule: Vec<ForkInfo>,
    pub active: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForkInfo {
    pub name: String,
    pub activation_time: u64,
    pub active: bool,
    #[serde(default)]
    pub fork_id: Option<String>,
}
