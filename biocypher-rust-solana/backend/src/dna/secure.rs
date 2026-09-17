//! Secure DNA Cryptography Module
//!
//! Combines AES-256-CBC encryption with DNA encoding
//! Ported from Python: biocypher/secure_nanopore_dna_crypto.py

use crate::dna::basic::DNACrypto;
use crate::dna::markers;
use crate::dna::traits::{DNACoder, SequenceStatistics, SequenceStats};
use crate::error::{DNACryptoError, Result};
use aes::Aes256;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use cbc::cipher::block_padding::Pkcs7;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use cbc::{Decryptor, Encryptor};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;

type Aes256CbcEnc = Encryptor<Aes256>;
type Aes256CbcDec = Decryptor<Aes256>;

/// Secure DNA cryptography with AES-256-CBC encryption
pub struct SecureDNACrypto;

impl DNACoder for SecureDNACrypto {
    fn encode_message(_message: &str) -> Result<String> {
        Err(DNACryptoError::PasswordRequired.into())
    }

    fn decode_sequence(_sequence: &str) -> Result<String> {
        Err(DNACryptoError::PasswordRequired.into())
    }
}

impl SequenceStats for SecureDNACrypto {
    fn get_sequence_stats(sequence: &str) -> SequenceStatistics {
        SequenceStatistics::new(sequence)
    }
}

impl SecureDNACrypto {
    /// Cryptographic constants
    pub const KEY_SIZE: usize = 32;
    pub const IV_SIZE: usize = 16;
    pub const SALT_SIZE: usize = 16;
    pub const PBKDF2_ITERATIONS: u32 = 100_000;

    /// Encode message with password (AES-256-CBC + Basic DNA encoding + markers)
    pub fn encode_with_password(message: &str, password: &str) -> Result<String> {
        if message.is_empty() {
            return Ok(String::new());
        }
        if password.is_empty() {
            return Err(DNACryptoError::PasswordRequired.into());
        }

        // Step 1: Encrypt with AES-256-CBC
        let (encrypted_data, iv, salt) = Self::encrypt_message(message, password)?;

        // Step 2: Serialize to base64 string (length-prefixed format)
        let crypto_string = Self::crypto_data_to_string(&encrypted_data, &iv, &salt)?;

        // Step 3: Encode to DNA using Basic mode
        let dna_sequence = DNACrypto::encode_message(&crypto_string)?;

        // Step 4: Add markers
        let result = format!(
            "{}{}{}",
            markers::START_MARKER,
            dna_sequence,
            markers::STOP_MARKER
        );

        Ok(result)
    }

    /// Decode sequence with password
    pub fn decode_with_password(sequence: &str, password: &str) -> Result<String> {
        if sequence.is_empty() {
            return Ok(String::new());
        }
        if password.is_empty() {
            return Err(DNACryptoError::PasswordRequired.into());
        }

        // Step 1: Remove markers
        let core = Self::remove_markers(sequence);

        // Step 2: Decode DNA using Basic mode
        let crypto_string = DNACrypto::decode_sequence(&core)?;
        if crypto_string.is_empty() {
            return Ok(String::new());
        }

        // Step 3: Parse crypto data
        let (encrypted_data, iv, salt) = Self::string_to_crypto_data(&crypto_string)?;

        // Step 4: Decrypt
        let plaintext = Self::decrypt_message(&encrypted_data, &iv, &salt, password)?;

        Ok(plaintext)
    }

    /// Derive key from password using PBKDF2
    fn derive_key(password: &str, salt: &[u8]) -> [u8; Self::KEY_SIZE] {
        let mut key = [0u8; Self::KEY_SIZE];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, Self::PBKDF2_ITERATIONS, &mut key);
        key
    }

    /// Encrypt message with password, returns (ciphertext, iv, salt)
    fn encrypt_message(
        plaintext: &str,
        password: &str,
    ) -> Result<(Vec<u8>, [u8; Self::IV_SIZE], [u8; Self::SALT_SIZE])> {
        let mut salt = [0u8; Self::SALT_SIZE];
        let mut iv = [0u8; Self::IV_SIZE];
        rand::thread_rng().fill_bytes(&mut salt);
        rand::thread_rng().fill_bytes(&mut iv);

        let key = Self::derive_key(password, &salt);
        let ciphertext = Self::encrypt_with_key(plaintext.as_bytes(), &key, &iv)?;
        Ok((ciphertext, iv, salt))
    }

    /// Encrypt plaintext with raw key and IV. Returns ciphertext.
    pub fn encrypt_with_key(
        plaintext: &[u8],
        key: &[u8; Self::KEY_SIZE],
        iv: &[u8; Self::IV_SIZE],
    ) -> Result<Vec<u8>> {
        let cipher = Aes256CbcEnc::new_from_slices(key, iv)
            .map_err(|e| DNACryptoError::EncryptionError(e.to_string()))?;
        Ok(cipher.encrypt_padded_vec_mut::<Pkcs7>(plaintext))
    }

    /// Decrypt message with password
    fn decrypt_message(
        ciphertext: &[u8],
        iv: &[u8],
        salt: &[u8],
        password: &str,
    ) -> Result<String> {
        let key = Self::derive_key(password, salt);
        Self::decrypt_with_key(ciphertext, &key, iv)
    }

    /// Decrypt ciphertext with raw key. Returns plaintext string.
    pub fn decrypt_with_key(
        ciphertext: &[u8],
        key: &[u8; Self::KEY_SIZE],
        iv: &[u8],
    ) -> Result<String> {
        let cipher = Aes256CbcDec::new_from_slices(key, iv)
            .map_err(|e| DNACryptoError::DecryptionError(e.to_string()))?;
        let decrypted = cipher
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
            .map_err(|e| DNACryptoError::DecryptionError(e.to_string()))?;

        String::from_utf8(decrypted)
            .map_err(|e| DNACryptoError::DecryptionError(e.to_string()).into())
    }

    /// Serialize crypto data to base64 string (public for split_key reuse)
    pub fn crypto_data_to_string(encrypted_data: &[u8], iv: &[u8], salt: &[u8]) -> Result<String> {
        let mut combined = Vec::new();
        combined.extend_from_slice(&(salt.len() as u16).to_be_bytes());
        combined.extend_from_slice(salt);
        combined.extend_from_slice(&(iv.len() as u16).to_be_bytes());
        combined.extend_from_slice(iv);
        combined.extend_from_slice(&(encrypted_data.len() as u32).to_be_bytes());
        combined.extend_from_slice(encrypted_data);
        Ok(BASE64.encode(&combined))
    }

    /// Parse base64 string to crypto data (public for split_key reuse)
    pub fn string_to_crypto_data(crypto_string: &str) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        let combined = BASE64
            .decode(crypto_string.as_bytes())
            .map_err(|e| DNACryptoError::DecryptionError(format!("Invalid base64: {}", e)))?;

        if combined.len() < 2 {
            return Err(DNACryptoError::DecryptionError("Invalid crypto data".to_string()).into());
        }

        let salt_len = u16::from_be_bytes([combined[0], combined[1]]) as usize;
        if combined.len() < 2 + salt_len {
            return Err(DNACryptoError::DecryptionError("Invalid salt length".to_string()).into());
        }
        let salt = combined[2..2 + salt_len].to_vec();

        let iv_start = 2 + salt_len;
        if combined.len() < iv_start + 2 {
            return Err(DNACryptoError::DecryptionError("Invalid IV length".to_string()).into());
        }
        let iv_len = u16::from_be_bytes([combined[iv_start], combined[iv_start + 1]]) as usize;
        let iv = combined[iv_start + 2..iv_start + 2 + iv_len].to_vec();

        let data_start = iv_start + 2 + iv_len;
        if combined.len() < data_start + 4 {
            return Err(DNACryptoError::DecryptionError("Invalid data length".to_string()).into());
        }
        let data_len = u32::from_be_bytes([
            combined[data_start],
            combined[data_start + 1],
            combined[data_start + 2],
            combined[data_start + 3],
        ]) as usize;
        let encrypted_data = combined[data_start + 4..data_start + 4 + data_len].to_vec();

        Ok((encrypted_data, iv, salt))
    }

    /// Remove start/stop markers (public for split_key reuse)
    pub fn remove_markers(sequence: &str) -> String {
        let mut seq = sequence.to_string();
        if seq.starts_with(markers::START_MARKER) {
            seq = seq[markers::START_MARKER.len()..].to_string();
        }
        if seq.ends_with(markers::STOP_MARKER) {
            seq = seq[..seq.len() - markers::STOP_MARKER.len()].to_string();
        }
        seq
    }

    /// Validate password strength (optional, for API use)
    pub fn validate_password_strength(password: &str) -> (bool, Vec<String>) {
        let mut issues = Vec::new();
        if password.len() < 8 {
            issues.push("Password should be at least 8 characters".to_string());
        }
        if !password.chars().any(|c| c.is_uppercase()) {
            issues.push("Password should contain uppercase letters".to_string());
        }
        if !password.chars().any(|c| c.is_lowercase()) {
            issues.push("Password should contain lowercase letters".to_string());
        }
        if !password.chars().any(|c| c.is_ascii_digit()) {
            issues.push("Password should contain numbers".to_string());
        }
        let valid = issues.is_empty();
        (valid, issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(SecureDNACrypto::KEY_SIZE, 32);
        assert_eq!(SecureDNACrypto::IV_SIZE, 16);
        assert_eq!(SecureDNACrypto::SALT_SIZE, 16);
        assert_eq!(SecureDNACrypto::PBKDF2_ITERATIONS, 100_000);
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = "Secret message";
        let password = "TestPass123!";
        let dna = SecureDNACrypto::encode_with_password(original, password).unwrap();
        assert!(dna.starts_with(markers::START_MARKER));
        assert!(dna.ends_with(markers::STOP_MARKER));
        let decoded = SecureDNACrypto::decode_with_password(&dna, password).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_encode_decode_empty() {
        assert_eq!(
            SecureDNACrypto::encode_with_password("", "pass").unwrap(),
            ""
        );
        assert_eq!(
            SecureDNACrypto::decode_with_password("", "pass").unwrap(),
            ""
        );
    }

    #[test]
    fn test_wrong_password_fails() {
        let dna = SecureDNACrypto::encode_with_password("secret", "password123").unwrap();
        let result = SecureDNACrypto::decode_with_password(&dna, "wrongpassword");
        assert!(result.is_err());
    }

    #[test]
    fn test_password_required() {
        let result = SecureDNACrypto::encode_with_password("msg", "");
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_password_strength() {
        let (valid, _) = SecureDNACrypto::validate_password_strength("TestPass123!");
        assert!(valid);

        let (invalid, issues) = SecureDNACrypto::validate_password_strength("weak");
        assert!(!invalid);
        assert!(!issues.is_empty());
    }
}
