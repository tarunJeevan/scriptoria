// Encryption service for Scriptoria Phase 1
// ChaCha20-Poly1305 with Argon2id key derivation

use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, Params, PasswordHash, PasswordVerifier, Version,
};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Nonce,
};
// use rand::{rngs::OsRng, RngCore};
use rand_core::{OsRng, TryRngCore};
use zeroize::Zeroize;

use crate::models::EncryptedContent;

// ============================================================================
// CONSTANTS
// ============================================================================

/// Argon2id parameters (64MB memory, 3 iterations)
const ARGON2_MEMORY_KB: u32 = 65536; // 64MB
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_PARALLELISM: u32 = 4;

/// Master key size (256 bits for ChaCha20)
const MASTER_KEY_SIZE: usize = 32;

// ============================================================================
// ENCRYPTION SERVICE
// ============================================================================

pub struct EncryptionService {
    master_key: [u8; MASTER_KEY_SIZE],
}

impl EncryptionService {
    /// Create a new encryption service with a master key
    /// SECURITY: Master key must be zeroized after use
    pub fn new(master_key: [u8; MASTER_KEY_SIZE]) -> Self {
        Self { master_key }
    }

    /// Derive master key from user password using Argon2id
    pub fn derive_master_key(
        password: &str,
        salt: &[u8],
    ) -> Result<[u8; MASTER_KEY_SIZE], EncryptionError> {
        let params = Params::new(
            ARGON2_MEMORY_KB,
            ARGON2_ITERATIONS,
            ARGON2_PARALLELISM,
            Some(MASTER_KEY_SIZE),
        )
        .map_err(|e| EncryptionError::KeyDerivation(e.to_string()))?;

        let argon2 = Argon2::new(argon2::Algorithm::Argon2id, Version::V0x13, params);

        let mut key = [0u8; MASTER_KEY_SIZE];
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .map_err(|e| EncryptionError::KeyDerivation(e.to_string()))?;

        Ok(key)
    }

    /// Generate a cryptographically secure random salt
    pub fn generate_salt() -> Result<Vec<u8>, EncryptionError> {
        let mut salt = vec![0u8; 32];
        OsRng
            .try_fill_bytes(&mut salt)
            .map_err(|e| EncryptionError::Randomness(e.to_string()))?;
        Ok(salt)
    }

    /// Encrypt content using ChaCha20-Poly1305
    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedContent, EncryptionError> {
        let cipher = ChaCha20Poly1305::new(&self.master_key.into());

        // Generate random nonce (96 bits for ChaCha20)
        let mut nonce_bytes = [0u8; 12];
        OsRng
            .try_fill_bytes(&mut nonce_bytes)
            .map_err(|e| EncryptionError::Randomness(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt plaintext
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| EncryptionError::Encryption(e.to_string()))?;

        // Extract authentication tag (last 16 bytes)
        if ciphertext.len() < 16 {
            return Err(EncryptionError::Encryption("Ciphertext too short".into()));
        }

        let tag_start = ciphertext.len() - 16;
        let mut tag = [0u8; 16];
        tag.copy_from_slice(&ciphertext[tag_start..]);

        let ciphertext_without_tag = ciphertext[..tag_start].to_vec();

        Ok(EncryptedContent {
            ciphertext: ciphertext_without_tag,
            nonce: nonce_bytes,
            tag,
        })
    }

    /// Decrypt content using ChaCha20-Poly1305
    pub fn decrypt(&self, encrypted: &EncryptedContent) -> Result<String, EncryptionError> {
        let cipher = ChaCha20Poly1305::new(&self.master_key.into());
        let nonce = Nonce::from_slice(&encrypted.nonce);

        // Reconstruct ciphertext with tag
        let mut ciphertext_with_tag = encrypted.ciphertext.clone();
        ciphertext_with_tag.extend_from_slice(&encrypted.tag);

        // Decrypt
        let plaintext = cipher
            .decrypt(nonce, ciphertext_with_tag.as_slice())
            .map_err(|e| EncryptionError::Decryption(e.to_string()))?;

        String::from_utf8(plaintext)
            .map_err(|e| EncryptionError::Decryption(format!("Invalid UTF-8: {}", e)))
    }

    /// Encrypt binary data (for file attachments)
    pub fn encrypt_binary(&self, data: &[u8]) -> Result<EncryptedContent, EncryptionError> {
        let cipher = ChaCha20Poly1305::new(&self.master_key.into());

        let mut nonce_bytes = [0u8; 12];
        OsRng
            .try_fill_bytes(&mut nonce_bytes)
            .map_err(|e| EncryptionError::Randomness(e.to_string()))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| EncryptionError::Encryption(e.to_string()))?;

        let tag_start = ciphertext.len() - 16;
        let mut tag = [0u8; 16];
        tag.copy_from_slice(&ciphertext[tag_start..]);

        let ciphertext_without_tag = ciphertext[..tag_start].to_vec();

        Ok(EncryptedContent {
            ciphertext: ciphertext_without_tag,
            nonce: nonce_bytes,
            tag,
        })
    }

    /// Decrypt binary data
    pub fn decrypt_binary(&self, encrypted: &EncryptedContent) -> Result<Vec<u8>, EncryptionError> {
        let cipher = ChaCha20Poly1305::new(&self.master_key.into());
        let nonce = Nonce::from_slice(&encrypted.nonce);

        let mut ciphertext_with_tag = encrypted.ciphertext.clone();
        ciphertext_with_tag.extend_from_slice(&encrypted.tag);

        cipher
            .decrypt(nonce, ciphertext_with_tag.as_slice())
            .map_err(|e| EncryptionError::Decryption(e.to_string()))
    }

    /// Derive a file-specific key from master key + file hash
    pub fn derive_file_key(
        &self,
        file_hash: &[u8],
    ) -> Result<[u8; MASTER_KEY_SIZE], EncryptionError> {
        use blake3::Hasher;

        let mut hasher = Hasher::new();
        hasher.update(&self.master_key);
        hasher.update(file_hash);

        let hash = hasher.finalize();
        let mut key = [0u8; MASTER_KEY_SIZE];
        key.copy_from_slice(&hash.as_bytes()[..MASTER_KEY_SIZE]);

        Ok(key)
    }
}

impl Drop for EncryptionService {
    fn drop(&mut self) {
        // Zeroize master key on drop
        self.master_key.zeroize();
    }
}

// ============================================================================
// KEY MANAGER (System Keyring Integration)
// ============================================================================

pub struct KeyManager;

impl KeyManager {
    /// Store master key salt in system keyring
    /// Returns true if stored successfully
    pub fn store_salt(salt: &[u8]) -> Result<(), EncryptionError> {
        let entry = keyring::Entry::new("scriptoria", "master_salt")
            .map_err(|e| EncryptionError::KeyStorage(e.to_string()))?;

        let salt_hex = hex::encode(salt);
        entry
            .set_password(&salt_hex)
            .map_err(|e| EncryptionError::KeyStorage(e.to_string()))?;

        Ok(())
    }

    /// Retrieve master key salt from system keyring
    pub fn retrieve_salt() -> Result<Vec<u8>, EncryptionError> {
        let entry = keyring::Entry::new("scriptoria", "master_salt")
            .map_err(|e| EncryptionError::KeyStorage(e.to_string()))?;

        let salt_hex = entry
            .get_password()
            .map_err(|e| EncryptionError::KeyStorage(e.to_string()))?;

        hex::decode(salt_hex)
            .map_err(|e| EncryptionError::KeyStorage(format!("Invalid salt format: {}", e)))
    }

    /// Check if salt exists in keyring
    pub fn has_salt() -> bool {
        keyring::Entry::new("scriptoria", "master_salt")
            .ok()
            .and_then(|e| e.get_password().ok())
            .is_some()
    }

    /// Delete salt from keyring (for reset/uninstall)
    pub fn delete_salt() -> Result<(), EncryptionError> {
        let entry = keyring::Entry::new("scriptoria", "master_salt")
            .map_err(|e| EncryptionError::KeyStorage(e.to_string()))?;

        entry
            .delete_credential()
            .map_err(|e| EncryptionError::KeyStorage(e.to_string()))?;

        Ok(())
    }
}

// ============================================================================
// PASSWORD VALIDATION
// ============================================================================

pub struct PasswordValidator;

impl PasswordValidator {
    /// Validate password strength
    /// Returns error message if invalid, Ok(()) if valid
    pub fn validate(password: &str) -> Result<(), EncryptionError> {
        if password.len() < 12 {
            return Err(EncryptionError::PasswordValidation(
                "Password must be at least 12 characters".into(),
            ));
        }

        if password.len() > 128 {
            return Err(EncryptionError::PasswordValidation(
                "Password must be at most 128 characters".into(),
            ));
        }

        let has_upper = password.chars().any(|c| c.is_uppercase());
        let has_lower = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_numeric());
        let has_special = password.chars().any(|c| !c.is_alphanumeric());

        let strength = [has_upper, has_lower, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();

        if strength < 3 {
            return Err(
            EncryptionError::PasswordValidation(
            "Password must contain at least 3 of: uppercase, lowercase, digit, special character"
                .into(),
            ));
        }

        Ok(())
    }
}

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum EncryptionError {
    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),

    #[error("Encryption failed: {0}")]
    Encryption(String),

    #[error("Decryption failed: {0}")]
    Decryption(String),

    #[error("Key storage failed: {0}")]
    KeyStorage(String),

    #[error("Invalid key format: {0}")]
    InvalidKey(String),

    #[error("Randomness generation failed: {0}")]
    Randomness(String),

    #[error("Password validation failed: {0}")]
    PasswordValidation(String),
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_roundtrip() {
        let master_key = [0u8; 32]; // Test key
        let service = EncryptionService::new(master_key);

        let plaintext = "Hello, Scriptoria!";
        let encrypted = service.encrypt(plaintext).unwrap();
        let decrypted = service.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_binary_encryption_roundtrip() {
        let master_key = [1u8; 32];
        let service = EncryptionService::new(master_key);

        let data = vec![0, 1, 2, 3, 4, 5, 255, 254, 253];
        let encrypted = service.encrypt_binary(&data).unwrap();
        let decrypted = service.decrypt_binary(&encrypted).unwrap();

        assert_eq!(data, decrypted);
    }

    fn gen_salt_with_retry(max_attempts: usize) -> Vec<u8> {
        for attempt in 1..=max_attempts {
            match EncryptionService::generate_salt() {
                Ok(salt) => return salt,
                Err(e) => {
                    eprintln!("generate_salt attempt {attempt}/{max_attempts} failed: {e}");
                    // Pause for 50 milliseconds before trying again
                    if attempt < max_attempts {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }
            }
        }
        panic!("Failed to generate salt after {max_attempts} attempts");
    }

    #[test]
    fn test_key_derivation() {
        let password = "test_password_12345";
        let salt = gen_salt_with_retry(3);

        let key1 = EncryptionService::derive_master_key(password, &salt).unwrap();
        let key2 = EncryptionService::derive_master_key(password, &salt).unwrap();

        assert_eq!(key1, key2); // Same password + salt = same key

        let different_salt = gen_salt_with_retry(3);
        let key3 = EncryptionService::derive_master_key(password, &different_salt).unwrap();

        assert_ne!(key1, key3); // Different salt = different key
    }

    #[test]
    fn test_password_validation() {
        // Password too short
        assert!(PasswordValidator::validate("Short1!").is_err());

        // Passwords missing complexity
        assert!(PasswordValidator::validate("NoDigitsOrSpecial").is_err());
        assert!(PasswordValidator::validate("nouppercaseorspecial123").is_err());

        // Valid passwords
        assert!(PasswordValidator::validate("Valid_Password_123").is_ok());
        assert!(PasswordValidator::validate("Another$Strong1Pass").is_ok());
        assert!(PasswordValidator::validate("Complex!Pass123Word").is_ok());
    }

    #[test]
    fn test_encrypted_content_blob_serialization() {
        let content = EncryptedContent {
            ciphertext: vec![1, 2, 3, 4, 5],
            nonce: [0; 12],
            tag: [255; 16],
        };

        let blob = content.to_blob();
        let reconstructed = EncryptedContent::from_blob(&blob).unwrap();

        assert_eq!(content.ciphertext, reconstructed.ciphertext);
        assert_eq!(content.nonce, reconstructed.nonce);
        assert_eq!(content.tag, reconstructed.tag);
    }

    #[test]
    fn test_file_key_derivation() {
        let master_key = [42u8; 32];
        let service = EncryptionService::new(master_key);

        let file_hash1 = b"file_hash_1";
        let file_hash2 = b"file_hash_2";

        let key1 = service.derive_file_key(file_hash1).unwrap();
        let key2 = service.derive_file_key(file_hash2).unwrap();
        let key1_again = service.derive_file_key(file_hash1).unwrap();

        assert_ne!(key1, key2); // Different files = different keys
        assert_eq!(key1, key1_again); // Same file = same key
    }

    #[test]
    fn test_different_keys_produce_different_ciphertext() {
        let key1 = [0u8; 32];
        let key2 = [1u8; 32];

        let service1 = EncryptionService::new(key1);
        let service2 = EncryptionService::new(key2);

        let plaintext = "Some plaintext";
        let encrypted1 = service1.encrypt(plaintext).unwrap();
        let encrypted2 = service2.encrypt(plaintext).unwrap();

        // Different keys should produce different ciphertext
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);

        // Each service should decrypt its own ciphertext
        assert_eq!(service1.decrypt(&encrypted1).unwrap(), plaintext);
        assert_eq!(service2.decrypt(&encrypted2).unwrap(), plaintext);

        // Cross-decryption should fail
        assert!(service1.decrypt(&encrypted2).is_err());
        assert!(service2.decrypt(&encrypted1).is_err());
    }

    #[test]
    fn test_salt_generation_uniqueness() {
        let salt1 = gen_salt_with_retry(3);
        let salt2 = gen_salt_with_retry(3);
        let salt3 = gen_salt_with_retry(3);

        // All salts should be unique
        assert_ne!(salt1, salt2);
        assert_ne!(salt2, salt3);
        assert_ne!(salt1, salt3);

        // All salts should be 32 bytes
        assert_eq!(salt1.len(), 32);
        assert_eq!(salt2.len(), 32);
        assert_eq!(salt3.len(), 32);
    }
}
