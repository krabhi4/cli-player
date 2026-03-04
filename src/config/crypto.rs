use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Nonce};
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

const SALT: &[u8] = b"cli-music-player-salt-v1";
const ITERATIONS: u32 = 100_000;

fn derive_key() -> [u8; 32] {
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "default".to_string());
    let nodename = hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let machine_id = format!("{nodename}:{username}");

    let mut key = [0u8; 32];
    pbkdf2_hmac::<Sha256>(machine_id.as_bytes(), SALT, ITERATIONS, &mut key);
    key
}

/// Encrypt a password for config storage. Returns base64-encoded nonce+ciphertext.
pub fn encrypt_password(password: &str) -> String {
    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key).expect("valid key size");
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(&nonce, password.as_bytes())
        .expect("encryption should not fail");

    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &combined)
}

/// Decrypt a stored password. Returns empty string on failure.
pub fn decrypt_password(encrypted: &str) -> Result<String, String> {
    let combined = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encrypted)
        .map_err(|e| format!("base64 decode error: {e}"))?;

    if combined.len() < 12 {
        return Err("encrypted data too short".to_string());
    }

    let key = derive_key();
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| format!("cipher init error: {e}"))?;
    let nonce = Nonce::from_slice(&combined[..12]);
    let plaintext = cipher.decrypt(nonce, &combined[12..]).map_err(|_| {
        "Cannot decrypt password. This may happen if the password was \
             encrypted on a different machine. Please re-enter your password."
            .to_string()
    })?;

    String::from_utf8(plaintext).map_err(|e| format!("utf8 decode error: {e}"))
}
