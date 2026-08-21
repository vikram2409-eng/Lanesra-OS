//! Integration Hub (spec §20): secrets-at-rest and the one-way hashing an
//! issued inbound API key uses instead. Two distinct primitives, matching
//! the spec's own distinction:
//!
//! - **Reversible encryption** (AES-256-GCM) for anything Lanesra itself
//!   must present again later - a Connection's API key/bearer token/basic
//!   password, an OAuth2 client secret + refresh token, a webhook's HMAC
//!   signing secret. Stored as `integration_secrets` rows (ciphertext +
//!   nonce, base64), decrypted only for the moment an outbound call or a
//!   signature needs the real value, never returned by any read command.
//! - **One-way hashing** (SHA-256) for an *inbound* API client's issued
//!   secret (`api_client_service`) - Lanesra only ever needs to verify a
//!   presented key matches, never re-send it anywhere, so this follows the
//!   same convention `auth_service::hash_password` already uses for user
//!   passwords, not the reversible path above.
//!
//! # Master key resolution
//!
//! Deliberately **not** resolved from a hidden global/env lookup inside
//! this module - every encrypt/decrypt call here takes the resolved key
//! as an explicit `&[u8; 32]` parameter, the same way every other service
//! function in this crate takes `&Connection` explicitly rather than
//! reaching for a thread-local. `resolve_master_key` is the one function
//! that *does* look at the environment/filesystem, called exactly once at
//! process startup by each binary (`src-tauri/src/lib.rs`,
//! `server/src/main.rs`), which then thread the resolved key through their
//! own app state next to `conn`, exactly like `actor_user_id` already is.
//! This keeps every service function in this module trivially testable
//! with a throwaway in-memory key and immune to parallel-test env-var
//! races, while still giving the two real binaries a single, deterministic
//! resolution order matching the spec's own §20 guidance:
//!   1. `LANESRA_SECRET_MASTER_KEY` env var (base64, 32 bytes) - the
//!      spec's "environment variables ... for master-key bootstrap",
//!      recommended for Team Workspace/internet deployments.
//!   2. Else a key file at the caller-supplied path, generated on first
//!      use if missing - the spec's explicitly-permitted "practical
//!      fallback" for desktop ("encrypted application store fallback"),
//!      kept as a *separate file* from the SQLite database itself (never a
//!      DB column) so a copied `.sqlite3` alone can't decrypt anything.

use std::path::Path;

use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rand_core::RngCore;
use sha2::{Digest, Sha256};

use crate::domain::AppError;
use crate::domain::AppResult;

const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

/// Resolves the master key an operator's deployment should use, following
/// the order documented on this module. `key_file_path` is only consulted
/// (and only ever written to) when `LANESRA_SECRET_MASTER_KEY` isn't set.
pub fn resolve_master_key(key_file_path: &Path) -> AppResult<[u8; KEY_LEN]> {
    if let Ok(b64) = std::env::var("LANESRA_SECRET_MASTER_KEY") {
        let bytes = BASE64
            .decode(b64.trim())
            .map_err(|e| AppError::Validation(format!("LANESRA_SECRET_MASTER_KEY is not valid base64: {e}")))?;
        return to_key_array(&bytes);
    }
    if let Ok(existing) = std::fs::read(key_file_path) {
        return to_key_array(&existing);
    }
    let mut key = [0u8; KEY_LEN];
    OsRng.fill_bytes(&mut key);
    if let Some(parent) = key_file_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::Validation(format!("could not create secret key directory: {e}")))?;
    }
    std::fs::write(key_file_path, key).map_err(|e| AppError::Validation(format!("could not write secret key file: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = std::fs::metadata(key_file_path) {
            let mut perms = meta.permissions();
            perms.set_mode(0o600);
            let _ = std::fs::set_permissions(key_file_path, perms);
        }
    }
    Ok(key)
}

fn to_key_array(bytes: &[u8]) -> AppResult<[u8; KEY_LEN]> {
    bytes
        .try_into()
        .map_err(|_| AppError::Validation(format!("secret master key must be exactly {KEY_LEN} bytes, got {}", bytes.len())))
}

/// Encrypts `plaintext`, returning `(ciphertext_b64, nonce_b64)` -
/// `integration_secrets.ciphertext`/`.nonce`'s exact shape.
pub fn encrypt(master_key: &[u8; KEY_LEN], plaintext: &str) -> AppResult<(String, String)> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(master_key));
    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| AppError::Validation(format!("could not encrypt secret: {e}")))?;
    Ok((BASE64.encode(ciphertext), BASE64.encode(nonce_bytes)))
}

/// Reverses `encrypt` - the one place a stored secret's real value is ever
/// reconstructed, and only ever held in memory for the moment an outbound
/// call or signature needs it.
pub fn decrypt(master_key: &[u8; KEY_LEN], ciphertext_b64: &str, nonce_b64: &str) -> AppResult<String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(master_key));
    let ciphertext = BASE64.decode(ciphertext_b64).map_err(|e| AppError::Validation(format!("invalid stored secret: {e}")))?;
    let nonce_bytes = BASE64.decode(nonce_b64).map_err(|e| AppError::Validation(format!("invalid stored secret nonce: {e}")))?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| AppError::Validation("could not decrypt secret - wrong master key or corrupted data".into()))?;
    String::from_utf8(plaintext).map_err(|e| AppError::Validation(format!("decrypted secret was not valid UTF-8: {e}")))
}

/// One-way hash for an inbound API client's issued secret - see this
/// module's own doc comment for why this is hashed, not encrypted.
pub fn hash_api_secret(secret: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hex::encode(hasher.finalize())
}

/// A cryptographically random, URL-safe-ish secret of `byte_len` random
/// bytes, hex-encoded - used for both an issued API client secret and a
/// webhook/HMAC signing secret.
pub fn generate_random_secret(byte_len: usize) -> String {
    let mut bytes = vec![0u8; byte_len];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; KEY_LEN] {
        let mut k = [0u8; KEY_LEN];
        OsRng.fill_bytes(&mut k);
        k
    }

    #[test]
    fn encrypt_then_decrypt_round_trips() {
        let key = test_key();
        let (ciphertext, nonce) = encrypt(&key, "sk_live_super_secret_token").unwrap();
        assert_ne!(ciphertext, "sk_live_super_secret_token");
        let plaintext = decrypt(&key, &ciphertext, &nonce).unwrap();
        assert_eq!(plaintext, "sk_live_super_secret_token");
    }

    #[test]
    fn decrypting_with_the_wrong_key_fails() {
        let key = test_key();
        let wrong_key = test_key();
        let (ciphertext, nonce) = encrypt(&key, "hello").unwrap();
        assert!(decrypt(&wrong_key, &ciphertext, &nonce).is_err());
    }

    #[test]
    fn resolve_master_key_from_env_var_takes_priority() {
        let key = test_key();
        let b64 = BASE64.encode(key);
        std::env::set_var("LANESRA_SECRET_MASTER_KEY", &b64);
        let resolved = resolve_master_key(Path::new("/nonexistent/should/not/be/read")).unwrap();
        assert_eq!(resolved, key);
        std::env::remove_var("LANESRA_SECRET_MASTER_KEY");
    }

    #[test]
    fn resolve_master_key_creates_and_reuses_a_key_file() {
        std::env::remove_var("LANESRA_SECRET_MASTER_KEY");
        let dir = std::env::temp_dir().join(format!("lanesra-secret-test-{}", crate::domain::ids::new_uuid()));
        let path = dir.join("secret.key");
        let first = resolve_master_key(&path).unwrap();
        let second = resolve_master_key(&path).unwrap();
        assert_eq!(first, second, "a second resolution must reuse the same on-disk key, not generate a new one");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn hash_api_secret_is_deterministic_and_one_way() {
        let hash1 = hash_api_secret("my-api-secret");
        let hash2 = hash_api_secret("my-api-secret");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, "my-api-secret");
    }

    #[test]
    fn generate_random_secret_is_not_constant() {
        assert_ne!(generate_random_secret(16), generate_random_secret(16));
    }
}
