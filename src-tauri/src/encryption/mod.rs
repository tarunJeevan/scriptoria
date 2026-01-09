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
            .map_err(|e| EncryptionError::Randomness(format!("RNG failed: {}", e)))?;
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

use std::fs;
use std::path::PathBuf;

pub struct KeyManager;

impl KeyManager {
    /// Get the fallback salt file path
    fn get_salt_file_path() -> PathBuf {
        let home = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| ".".to_string());

        PathBuf::from(home).join(".scriptoria").join("salt.enc")
    }

    /// Create a keyring entry with consistent parameters
    fn create_keyring_entry() -> Result<keyring::Entry, keyring::Error> {
        keyring::Entry::new("scriptoria", "master_salt")
    }

    /// Check if keyring is available on the platform
    pub fn is_keyring_available() -> bool {
        // Try to create test entry
        match Self::create_keyring_entry() {
            Ok(entry) => {
                // Try a simple set/get/delete operation
                let test_value = "keyring_test_value";

                match entry.set_password(test_value) {
                    Ok(_) => {
                        // Immediately try to retrieve it with the same entry
                        let same_entry_works =
                            matches!(entry.get_password(), Ok(v) if v == test_value);

                        // Try to retrieve it with a FRESH entry
                        let fresh_entry_result = Self::create_keyring_entry()
                            .and_then(|fresh_entry| fresh_entry.get_password());

                        let fresh_entry_works =
                            matches!(&fresh_entry_result, Ok(v) if v == test_value);

                        // Cleanup
                        let _ = entry.delete_credential();

                        // Only return true if same AND fresh entry retrieval works
                        same_entry_works && fresh_entry_works
                    }
                    Err(_) => false,
                }
            }
            Err(_) => false,
        }
    }

    /// Store master key salt in system keyring (with file fallback)
    pub fn store_salt(salt: &[u8]) -> Result<(), EncryptionError> {
        // Check if keyring is actually available before trying to use it
        if !Self::is_keyring_available() {
            eprintln!("Warning: Keyring not available. Using file fallback immediately.");
            return Self::store_salt_file(salt);
        }
        // Try keyring
        match Self::store_salt_keyring(salt) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!(
                    "Warning: Keyring storage failed ({}), using file fallback",
                    e
                );
                Self::store_salt_file(salt)
            }
        }
    }

    /// Store salt in system keyring
    /// Returns true if stored successfully
    fn store_salt_keyring(salt: &[u8]) -> Result<(), EncryptionError> {
        let entry = Self::create_keyring_entry().map_err(|e| {
            EncryptionError::KeyStorage(format!("Failed to create keyring entry: {}", e))
        })?;

        let salt_hex = hex::encode(salt);
        entry
            .set_password(&salt_hex)
            .map_err(|e| EncryptionError::KeyStorage(e.to_string()))?;

        // Immediate verification - create a FRESH entry to simulate cross session access
        let verify_entry = Self::create_keyring_entry().map_err(|e| {
            EncryptionError::KeyStorage(format!("Failed to create verification entry: {}", e))
        })?;

        match verify_entry.get_password() {
            Ok(retrieved) => {
                if retrieved != salt_hex {
                    return Err(EncryptionError::KeyStorage(
                        "Stored value doesn't match retrieved value".to_string(),
                    ));
                }
                Ok(())
            }
            Err(e) => Err(EncryptionError::KeyStorage(format!(
                "Stored but immediately retrieval with fresh entry failed: {}",
                e
            ))),
        }
    }

    /// Store salt in encrypted file (fallback)
    fn store_salt_file(salt: &[u8]) -> Result<(), EncryptionError> {
        let path = Self::get_salt_file_path();

        // Create directory if it doesn't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                EncryptionError::KeyStorage(format!("Failed to create directory: {}", e))
            })?;
        }

        // Store as hex-encoded string
        let salt_hex = hex::encode(salt);
        fs::write(&path, salt_hex).map_err(|e| {
            EncryptionError::KeyStorage(format!("Failed to write salt file: {}", e))
        })?;

        // Set restrictive permissions (Unix only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&path)
                .map_err(|e| {
                    EncryptionError::KeyStorage(format!("Failed to get file metadata: {}", e))
                })?
                .permissions();
            perms.set_mode(0o600); // Read/write for owner only
            fs::set_permissions(&path, perms).map_err(|e| {
                EncryptionError::KeyStorage(format!("Failed to set permissions: {}", e))
            })?;
        }

        Ok(())
    }

    /// Retrieve master key salt from system keyring (with file fallback)
    pub fn retrieve_salt() -> Result<Vec<u8>, EncryptionError> {
        // Check if keyring is actually available before trying to use it
        if !Self::is_keyring_available() {
            eprintln!("Warning: Keyring not available. Retrieving file fallback immediately.");
            return Self::retrieve_salt_file();
        }

        // Try keyring first
        match Self::retrieve_salt_keyring() {
            Ok(salt) => Ok(salt),
            Err(_) => {
                // Fall back to file
                Self::retrieve_salt_file()
            }
        }
    }

    /// Retrieve salt from system keyring
    fn retrieve_salt_keyring() -> Result<Vec<u8>, EncryptionError> {
        let entry = Self::create_keyring_entry().map_err(|e| {
            EncryptionError::KeyStorage(format!("Failed to create keyring entry: {}", e))
        })?;

        let salt_hex = entry
            .get_password()
            .map_err(|e| EncryptionError::KeyStorage(format!("Failed to get password: {}", e)))?;

        hex::decode(salt_hex)
            .map_err(|e| EncryptionError::KeyStorage(format!("Invalid salt format: {}", e)))
    }

    /// Retrieve salt from encrypted file (fallback)
    fn retrieve_salt_file() -> Result<Vec<u8>, EncryptionError> {
        let path = Self::get_salt_file_path();

        if !path.exists() {
            return Err(EncryptionError::KeyStorage(
                "Salt file not found".to_string(),
            ));
        }

        let salt_hex = fs::read_to_string(&path)
            .map_err(|e| EncryptionError::KeyStorage(format!("Failed to read salt file: {}", e)))?;

        hex::decode(salt_hex.trim())
            .map_err(|e| EncryptionError::KeyStorage(format!("Invalid salt format: {}", e)))
    }

    /// Check if salt exists (keyring or file)
    pub fn has_salt() -> bool {
        let has_keyring = Self::has_salt_keyring();
        let has_file = Self::has_salt_file();

        has_keyring || has_file
    }

    /// Check if salt exists in keyring
    fn has_salt_keyring() -> bool {
        match Self::create_keyring_entry() {
            Ok(entry) => entry.get_password().is_ok(),
            Err(_) => false,
        }
    }

    /// Check if salt exists in file
    fn has_salt_file() -> bool {
        let path = Self::get_salt_file_path();
        path.exists()
    }

    /// Delete salt from keyring and file (for reset/uninstall)
    pub fn delete_salt() -> Result<(), EncryptionError> {
        // Try to delete from both locations
        let keyring_result = Self::delete_salt_keyring();
        let file_result = Self::delete_salt_file();

        // Return success if either deletion succeeded
        if keyring_result.is_ok() || file_result.is_ok() {
            Ok(())
        } else {
            Err(EncryptionError::KeyStorage(
                "Failed to delete salt from both keyring and file".to_string(),
            ))
        }
    }

    /// Delete salt from keyring
    fn delete_salt_keyring() -> Result<(), EncryptionError> {
        let entry = Self::create_keyring_entry().map_err(|e| {
            EncryptionError::KeyStorage(format!("Failed to create keyring entry: {}", e))
        })?;

        entry.delete_credential().map_err(|e| {
            EncryptionError::KeyStorage(format!("Failed to delete credential: {}", e))
        })?;

        Ok(())
    }

    /// Delete salt from file
    fn delete_salt_file() -> Result<(), EncryptionError> {
        let path = Self::get_salt_file_path();

        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                EncryptionError::KeyStorage(format!("Failed to delete salt file: {}", e))
            })?;
        }

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
