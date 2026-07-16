use blake3::Hasher;
use chrono::{DateTime, Utc};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use tempoforge_common::{ApiKeyId, OrgId, UserId};

const API_KEY_PREFIX: &str = "tf_live_";
const API_KEY_BYTES: usize = 32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyRecord {
    pub id: ApiKeyId,
    pub org_id: OrgId,
    pub created_by: UserId,
    pub name: String,
    pub key_prefix: String,
    pub key_hash: String,
    pub scopes: Vec<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub struct ApiKeyHasher;

impl ApiKeyHasher {
    pub fn hash(raw_key: &str) -> String {
        let mut hasher = Hasher::new();
        hasher.update(raw_key.as_bytes());
        hasher.finalize().to_hex().to_string()
    }

    pub fn verify(raw_key: &str, expected_hash: &str) -> bool {
        Self::hash(raw_key) == expected_hash
    }
}

/// Returns `(raw_key, prefix, hash)`. The raw key is shown once to the user.
pub fn generate_api_key() -> (String, String, String) {
    let mut bytes = [0u8; API_KEY_BYTES];
    rand::thread_rng().fill_bytes(&mut bytes);
    let secret = hex::encode(bytes);
    let raw = format!("{API_KEY_PREFIX}{secret}");
    let prefix = raw.chars().take(12).collect::<String>();
    let hash = ApiKeyHasher::hash(&raw);
    (raw, prefix, hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_hash() {
        let (raw, prefix, hash) = generate_api_key();
        assert!(raw.starts_with(API_KEY_PREFIX));
        assert_eq!(prefix.len(), 12);
        assert!(ApiKeyHasher::verify(&raw, &hash));
        assert!(!ApiKeyHasher::verify("tf_live_wrong", &hash));
    }
}
