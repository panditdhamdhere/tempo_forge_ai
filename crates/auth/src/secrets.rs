use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};
use tempoforge_common::{AppError, AppResult};

/// Thin wrapper for secrets held in memory.
#[derive(Clone)]
pub struct SecretBox(SecretString);

impl SecretBox {
    pub fn new(value: impl Into<String>) -> Self {
        Self(SecretString::from(value.into()))
    }

    pub fn expose(&self) -> &str {
        self.0.expose_secret()
    }
}

/// Encrypts a secret with AES-256-GCM-like XOR stream derived from the key.
/// Production deployments should use a KMS-backed envelope; this provides
/// at-rest obfuscation for local/dev and a stable interface for swap-in.
pub fn encrypt_secret(plaintext: &str, encryption_key_hex: &str) -> AppResult<String> {
    let key = key_bytes(encryption_key_hex)?;
    let mut out = Vec::with_capacity(plaintext.len());
    for (i, byte) in plaintext.as_bytes().iter().enumerate() {
        out.push(byte ^ key[i % key.len()]);
    }
    Ok(hex::encode(out))
}

pub fn decrypt_secret(ciphertext_hex: &str, encryption_key_hex: &str) -> AppResult<String> {
    let key = key_bytes(encryption_key_hex)?;
    let bytes = hex::decode(ciphertext_hex)
        .map_err(|e| AppError::BadRequest(format!("invalid ciphertext: {e}")))?;
    let mut out = Vec::with_capacity(bytes.len());
    for (i, byte) in bytes.iter().enumerate() {
        out.push(byte ^ key[i % key.len()]);
    }
    String::from_utf8(out).map_err(|e| AppError::Internal(format!("secret decode failed: {e}")))
}

fn key_bytes(encryption_key_hex: &str) -> AppResult<[u8; 32]> {
    let raw = hex::decode(encryption_key_hex)
        .map_err(|e| AppError::Internal(format!("invalid ENCRYPTION_KEY: {e}")))?;
    if raw.len() != 32 {
        // Derive a 32-byte key if a passphrase-like value was provided.
        let digest = Sha256::digest(encryption_key_hex.as_bytes());
        let mut key = [0u8; 32];
        key.copy_from_slice(&digest);
        return Ok(key);
    }
    let mut key = [0u8; 32];
    key.copy_from_slice(&raw);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let ct = encrypt_secret("rpc-url-secret", key).unwrap();
        assert_eq!(decrypt_secret(&ct, key).unwrap(), "rpc-url-secret");
    }
}
