//! Encryption utilities for secure credential storage
//!
//! Uses AES-256-GCM for authenticated encryption of API credentials.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::Rng;

/// Encryption key length (256 bits = 32 bytes)
const KEY_LENGTH: usize = 32;
/// Nonce length for AES-GCM (96 bits = 12 bytes)
const NONCE_LENGTH: usize = 12;

/// Encrypt data using AES-256-GCM
/// 
/// The output format is: nonce (12 bytes) || ciphertext
pub fn encrypt(plaintext: &[u8], key: &[u8; KEY_LENGTH]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;
    
    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_LENGTH];
    rand::thread_rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;
    
    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(NONCE_LENGTH + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend(ciphertext);
    
    Ok(result)
}

/// Decrypt data encrypted with AES-256-GCM
pub fn decrypt(encrypted: &[u8], key: &[u8; KEY_LENGTH]) -> Result<Vec<u8>, String> {
    if encrypted.len() < NONCE_LENGTH {
        return Err("Encrypted data too short".to_string());
    }
    
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|e| format!("Failed to create cipher: {}", e))?;
    
    // Split nonce and ciphertext
    let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_LENGTH);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    // Decrypt
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {}", e))
}

/// Encrypt a JSON-serializable value
pub fn encrypt_json<T: serde::Serialize>(value: &T, key: &[u8; KEY_LENGTH]) -> Result<Vec<u8>, String> {
    let json = serde_json::to_vec(value)
        .map_err(|e| format!("JSON serialization failed: {}", e))?;
    encrypt(&json, key)
}

/// Decrypt and deserialize JSON
pub fn decrypt_json<T: serde::de::DeserializeOwned>(encrypted: &[u8], key: &[u8; KEY_LENGTH]) -> Result<T, String> {
    let plaintext = decrypt(encrypted, key)?;
    serde_json::from_slice(&plaintext)
        .map_err(|e| format!("JSON deserialization failed: {}", e))
}

/// Generate a new random encryption key
pub fn generate_key() -> [u8; KEY_LENGTH] {
    let mut key = [0u8; KEY_LENGTH];
    rand::thread_rng().fill(&mut key);
    key
}

/// Generate a random webhook secret (64 hex characters = 32 bytes)
pub fn generate_webhook_secret() -> String {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill(&mut bytes);
    hex::encode(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = generate_key();
        let plaintext = b"Hello, World!";
        
        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_encrypt_json_roundtrip() {
        let key = generate_key();
        let value = serde_json::json!({
            "username": "test",
            "password": "secret123"
        });
        
        let encrypted = encrypt_json(&value, &key).unwrap();
        let decrypted: serde_json::Value = decrypt_json(&encrypted, &key).unwrap();
        
        assert_eq!(value, decrypted);
    }
}
