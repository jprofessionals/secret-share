use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::password_hash::{PasswordHash, SaltString};
use argon2::{Argon2, PasswordHasher};
use base64::{engine::general_purpose, Engine as _};
use rand::Rng;

use crate::error::AppError;

const NONCE_SIZE: usize = 12;

/// Derive encryption key from passphrase using Argon2
fn derive_key(passphrase: &str, salt: &[u8]) -> Result<[u8; 32], AppError> {
    let argon2 = Argon2::default();
    let salt_string = SaltString::encode_b64(salt)
        .map_err(|_| AppError::CryptoError("Failed to encode salt".to_string()))?;

    let password_hash = argon2
        .hash_password(passphrase.as_bytes(), &salt_string)
        .map_err(|e| AppError::CryptoError(format!("Key derivation failed: {}", e)))?;

    let hash_string = password_hash.to_string();
    let hash = PasswordHash::new(&hash_string)
        .map_err(|e| AppError::CryptoError(format!("Hash parsing failed: {}", e)))?;

    let hash_bytes = hash
        .hash
        .ok_or(AppError::CryptoError("No hash found".to_string()))?;

    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_bytes.as_bytes()[..32]);

    Ok(key)
}

/// Encrypt secret data with passphrase
pub fn encrypt_secret(plaintext: &str, passphrase: &str) -> Result<String, AppError> {
    // Generate random salt and nonce
    let mut rng = rand::rng();
    let salt: [u8; 16] = rng.random();
    let nonce_bytes: [u8; NONCE_SIZE] = rng.random();

    // Derive key from passphrase
    let key = derive_key(passphrase, &salt)?;

    // Create cipher
    let cipher = Aes256Gcm::new(&key.into());
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| AppError::CryptoError(format!("Encryption failed: {}", e)))?;

    // Combine salt + nonce + ciphertext
    let mut result = Vec::new();
    result.extend_from_slice(&salt);
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    // Encode as base64
    Ok(general_purpose::STANDARD.encode(result))
}

/// Decrypt secret data with passphrase
pub fn decrypt_secret(encrypted: &str, passphrase: &str) -> Result<String, AppError> {
    // Decode from base64
    let data = general_purpose::STANDARD
        .decode(encrypted)
        .map_err(|e| AppError::CryptoError(format!("Base64 decode failed: {}", e)))?;

    if data.len() < 16 + NONCE_SIZE {
        return Err(AppError::CryptoError("Invalid encrypted data".to_string()));
    }

    // Extract salt, nonce, and ciphertext
    let salt = &data[..16];
    let nonce_bytes = &data[16..16 + NONCE_SIZE];
    let ciphertext = &data[16 + NONCE_SIZE..];

    // Derive key from passphrase
    let key = derive_key(passphrase, salt)?;

    // Create cipher
    let cipher = Aes256Gcm::new(&key.into());
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| AppError::InvalidPassphrase)?;

    String::from_utf8(plaintext)
        .map_err(|e| AppError::CryptoError(format!("UTF-8 decode failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let secret = "This is a secret message";
        let passphrase = "correct-horse-battery";

        let encrypted = encrypt_secret(secret, passphrase).unwrap();
        let decrypted = decrypt_secret(&encrypted, passphrase).unwrap();

        assert_eq!(secret, decrypted);
    }

    #[test]
    fn test_wrong_passphrase() {
        let secret = "This is a secret message";
        let passphrase = "correct-horse-battery";
        let wrong_passphrase = "wrong-horse-battery";

        let encrypted = encrypt_secret(secret, passphrase).unwrap();
        let result = decrypt_secret(&encrypted, wrong_passphrase);

        assert!(result.is_err());
    }

    #[test]
    fn test_encrypt_decrypt_empty_string() {
        let secret = "";
        let passphrase = "correct-horse-battery";

        let encrypted = encrypt_secret(secret, passphrase).unwrap();
        let decrypted = decrypt_secret(&encrypted, passphrase).unwrap();

        assert_eq!(secret, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_unicode() {
        let secret = "こんにちは世界 🔐 Ñoño émojis 中文 العربية";
        let passphrase = "correct-horse-battery";

        let encrypted = encrypt_secret(secret, passphrase).unwrap();
        let decrypted = decrypt_secret(&encrypted, passphrase).unwrap();

        assert_eq!(secret, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_large_data() {
        // Create a large secret (100KB)
        let secret = "A".repeat(100_000);
        let passphrase = "correct-horse-battery";

        let encrypted = encrypt_secret(&secret, passphrase).unwrap();
        let decrypted = decrypt_secret(&encrypted, passphrase).unwrap();

        assert_eq!(secret, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_special_characters() {
        let secret = r#"{"password": "p@$$w0rd!", "key": "abc\n123\t456"}"#;
        let passphrase = "correct-horse-battery";

        let encrypted = encrypt_secret(secret, passphrase).unwrap();
        let decrypted = decrypt_secret(&encrypted, passphrase).unwrap();

        assert_eq!(secret, decrypted);
    }

    #[test]
    fn test_different_passphrases_produce_different_ciphertexts() {
        let secret = "Same secret";
        let passphrase1 = "first-passphrase-here";
        let passphrase2 = "second-passphrase-here";

        let encrypted1 = encrypt_secret(secret, passphrase1).unwrap();
        let encrypted2 = encrypt_secret(secret, passphrase2).unwrap();

        // Different passphrases should produce different ciphertexts
        assert_ne!(encrypted1, encrypted2);
    }

    #[test]
    fn test_same_passphrase_produces_different_ciphertexts() {
        // Due to random salt/nonce, encrypting twice should produce different results
        let secret = "Same secret";
        let passphrase = "same-passphrase-here";

        let encrypted1 = encrypt_secret(secret, passphrase).unwrap();
        let encrypted2 = encrypt_secret(secret, passphrase).unwrap();

        // Should be different due to random salt and nonce
        assert_ne!(encrypted1, encrypted2);

        // But both should decrypt correctly
        let decrypted1 = decrypt_secret(&encrypted1, passphrase).unwrap();
        let decrypted2 = decrypt_secret(&encrypted2, passphrase).unwrap();

        assert_eq!(secret, decrypted1);
        assert_eq!(secret, decrypted2);
    }

    #[test]
    fn test_invalid_base64_data() {
        let result = decrypt_secret("not-valid-base64!!!", "passphrase");
        assert!(result.is_err());
    }

    #[test]
    fn test_truncated_encrypted_data() {
        // Data shorter than salt + nonce (28 bytes minimum)
        let short_data = general_purpose::STANDARD.encode(&[0u8; 10]);
        let result = decrypt_secret(&short_data, "passphrase");
        assert!(result.is_err());
    }
}
