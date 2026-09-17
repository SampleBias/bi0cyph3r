//! Split Key DNA Cryptography Module
//!
//! Encrypts message with random key K split into K1 (user) and K2 (escrow).
//! Provider A (DNA manufacturer) synthesizes ciphertext-as-DNA; Provider B holds K2.
//! Neither can decrypt alone.

use crate::dna::basic::DNACrypto;
use crate::dna::markers;
use crate::dna::secure::SecureDNACrypto;
use crate::dna::traits::{DNACoder, SequenceStatistics, SequenceStats};
use crate::error::{DNACryptoError, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::RngCore;

/// Split Key DNA cryptography
pub struct SplitKeyDNACrypto;

impl DNACoder for SplitKeyDNACrypto {
    fn encode_message(_message: &str) -> Result<String> {
        Err(DNACryptoError::SplitKeyRequired.into())
    }

    fn decode_sequence(_sequence: &str) -> Result<String> {
        Err(DNACryptoError::SplitKeyRequired.into())
    }
}

impl SequenceStats for SplitKeyDNACrypto {
    fn get_sequence_stats(sequence: &str) -> SequenceStatistics {
        SequenceStatistics::new(sequence)
    }
}

impl SplitKeyDNACrypto {
    /// Encode message with split keys. Returns (dna_sequence, k1_base64, k2_base64).
    pub fn encode_with_split_keys(message: &str) -> Result<(String, String, String)> {
        if message.is_empty() {
            return Ok((String::new(), String::new(), String::new()));
        }

        // Generate random K
        let mut k = [0u8; SecureDNACrypto::KEY_SIZE];
        rand::thread_rng().fill_bytes(&mut k);

        // Generate random IV and salt
        let mut iv = [0u8; SecureDNACrypto::IV_SIZE];
        let mut salt = [0u8; SecureDNACrypto::SALT_SIZE];
        rand::thread_rng().fill_bytes(&mut iv);
        rand::thread_rng().fill_bytes(&mut salt);

        // Encrypt with K
        let ciphertext = SecureDNACrypto::encrypt_with_key(message.as_bytes(), &k, &iv)?;

        // Serialize to base64 (same format as Secure mode)
        let crypto_string = SecureDNACrypto::crypto_data_to_string(&ciphertext, &iv, &salt)?;

        // Encode to DNA
        let dna_sequence = DNACrypto::encode_message(&crypto_string)?;

        // Add markers
        let result = format!(
            "{}{}{}",
            markers::START_MARKER,
            dna_sequence,
            markers::STOP_MARKER
        );

        // Split K: K1 = random, K2 = K XOR K1
        let mut k1 = [0u8; SecureDNACrypto::KEY_SIZE];
        rand::thread_rng().fill_bytes(&mut k1);

        let mut k2 = [0u8; SecureDNACrypto::KEY_SIZE];
        for i in 0..SecureDNACrypto::KEY_SIZE {
            k2[i] = k[i] ^ k1[i];
        }

        Ok((result, BASE64.encode(k1), BASE64.encode(k2)))
    }

    /// Decode sequence with split keys K1 and K2.
    pub fn decode_with_split_keys(
        sequence: &str,
        k1_base64: &str,
        k2_base64: &str,
    ) -> Result<String> {
        if sequence.is_empty() {
            return Ok(String::new());
        }

        let k1 = BASE64
            .decode(k1_base64.as_bytes())
            .map_err(|e| DNACryptoError::DecryptionError(format!("Invalid K1 base64: {}", e)))?;
        let k2 = BASE64
            .decode(k2_base64.as_bytes())
            .map_err(|e| DNACryptoError::DecryptionError(format!("Invalid K2 base64: {}", e)))?;

        if k1.len() != SecureDNACrypto::KEY_SIZE || k2.len() != SecureDNACrypto::KEY_SIZE {
            return Err(DNACryptoError::DecryptionError(
                "K1 and K2 must be 32 bytes each".to_string(),
            )
            .into());
        }

        // Reconstruct K = K1 XOR K2
        let mut k = [0u8; SecureDNACrypto::KEY_SIZE];
        for i in 0..SecureDNACrypto::KEY_SIZE {
            k[i] = k1[i] ^ k2[i];
        }

        // Remove markers
        let core = SecureDNACrypto::remove_markers(sequence);

        // Decode DNA to base64
        let crypto_string = DNACrypto::decode_sequence(&core)?;
        if crypto_string.is_empty() {
            return Ok(String::new());
        }

        // Parse crypto data
        let (encrypted_data, iv, _salt) = SecureDNACrypto::string_to_crypto_data(&crypto_string)?;

        // Decrypt with K
        let iv_arr: [u8; SecureDNACrypto::IV_SIZE] = iv
            .try_into()
            .map_err(|_| DNACryptoError::DecryptionError("Invalid IV length".to_string()))?;

        SecureDNACrypto::decrypt_with_key(&encrypted_data, &k, &iv_arr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = "Secret message for split key";
        let (dna, k1, k2) = SplitKeyDNACrypto::encode_with_split_keys(original).unwrap();
        assert!(dna.starts_with(markers::START_MARKER));
        assert!(dna.ends_with(markers::STOP_MARKER));
        let decoded = SplitKeyDNACrypto::decode_with_split_keys(&dna, &k1, &k2).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_encode_decode_empty() {
        let (dna, k1, k2) = SplitKeyDNACrypto::encode_with_split_keys("").unwrap();
        assert_eq!(dna, "");
        assert_eq!(k1, "");
        assert_eq!(k2, "");
        let decoded = SplitKeyDNACrypto::decode_with_split_keys("", "", "").unwrap();
        assert_eq!(decoded, "");
    }

    #[test]
    fn test_wrong_k2_fails() {
        let original = "secret";
        let (dna, k1, _k2) = SplitKeyDNACrypto::encode_with_split_keys(original).unwrap();
        let wrong_k2 = BASE64.encode([0u8; 32]);
        let result = SplitKeyDNACrypto::decode_with_split_keys(&dna, &k1, &wrong_k2);
        assert!(result.is_err());
    }
}
