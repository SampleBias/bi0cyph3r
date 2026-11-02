//! Version 2 authenticated DNA envelopes.
use crate::error::{DNACryptoError, Result};
use aes_gcm::{
    aead::{Aead, Payload},
    Aes256Gcm, KeyInit, Nonce,
};
use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::{rngs::OsRng, RngCore};

const PREFIX: &str = "GCGTACGCGTACGCGT";
const PASSWORD_MODE: u8 = 1;
const SPLIT_MODE: u8 = 2;
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

fn fail(message: &str) -> crate::error::BioCypherError {
    DNACryptoError::DecodingFailed(message.into()).into()
}
fn random<const N: usize>() -> Result<[u8; N]> {
    let mut value = [0; N];
    OsRng
        .try_fill_bytes(&mut value)
        .map_err(|_| DNACryptoError::EncryptionError("Random number generation failed".into()))?;
    Ok(value)
}
fn password_key(password: &str, salt: &[u8]) -> Result<[u8; 32]> {
    if password.is_empty() {
        return Err(DNACryptoError::PasswordRequired.into());
    }
    let params = Params::new(19_456, 2, 1, Some(32))
        .map_err(|_| fail("Invalid key derivation parameters"))?;
    let mut key = [0; 32];
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|_| fail("Key derivation failed"))?;
    Ok(key)
}
fn dna(bytes: &[u8]) -> String {
    let alphabet = b"ATCG";
    let mut result = String::with_capacity(bytes.len() * 4);
    for &byte in bytes {
        for shift in [6, 4, 2, 0] {
            result.push(alphabet[((byte >> shift) & 3) as usize] as char);
        }
    }
    result
}
fn bytes(sequence: &str) -> Result<Vec<u8>> {
    if !sequence.len().is_multiple_of(4) {
        return Err(fail("Incomplete encrypted envelope"));
    }
    sequence
        .as_bytes()
        .chunks(4)
        .map(|chunk| {
            let mut byte = 0;
            for &base in chunk {
                byte = (byte << 2)
                    | match base {
                        b'A' => 0,
                        b'T' => 1,
                        b'C' => 2,
                        b'G' => 3,
                        _ => return Err(fail("Invalid DNA base")),
                    };
            }
            Ok(byte)
        })
        .collect()
}
fn seal(
    mode: u8,
    salt: &[u8],
    nonce: &[u8; NONCE_LEN],
    key: &[u8; 32],
    message: &str,
) -> Result<String> {
    let mut header = vec![mode];
    header.extend_from_slice(salt);
    header.extend_from_slice(nonce);
    let mut aad = PREFIX.as_bytes().to_vec();
    aad.extend_from_slice(&header);
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|_| fail("Invalid encryption key"))?;
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(nonce),
            Payload {
                msg: message.as_bytes(),
                aad: &aad,
            },
        )
        .map_err(|_| DNACryptoError::EncryptionError("Encryption failed".into()))?;
    header.extend_from_slice(&ciphertext);
    Ok(format!("{PREFIX}{}", dna(&header)))
}
fn open(
    sequence: &str,
    mode: u8,
    password: Option<&str>,
    split_key: Option<&[u8; 32]>,
) -> Result<String> {
    let data = bytes(
        sequence
            .strip_prefix(PREFIX)
            .ok_or_else(|| fail("Expected a version 2 encrypted sequence"))?,
    )?;
    let salt_len = if mode == PASSWORD_MODE { SALT_LEN } else { 0 };
    let header_len = 1 + salt_len + NONCE_LEN;
    if data.len() < header_len + TAG_LEN || data[0] != mode {
        return Err(fail("Invalid encrypted envelope or mode"));
    }
    let key = if let Some(password) = password {
        password_key(password, &data[1..1 + salt_len])?
    } else {
        *split_key.ok_or_else(|| fail("Missing split key"))?
    };
    let mut aad = PREFIX.as_bytes().to_vec();
    aad.extend_from_slice(&data[..header_len]);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|_| fail("Invalid encryption key"))?;
    let plaintext = cipher
        .decrypt(
            Nonce::from_slice(&data[1 + salt_len..header_len]),
            Payload {
                msg: &data[header_len..],
                aad: &aad,
            },
        )
        .map_err(|_| fail("Authentication failed: incorrect credentials or damaged data"))?;
    String::from_utf8(plaintext).map_err(|_| fail("Decoded text is not UTF-8"))
}
pub fn encode_password(message: &str, password: &str) -> Result<String> {
    let salt = random::<SALT_LEN>()?;
    let nonce = random::<NONCE_LEN>()?;
    seal(
        PASSWORD_MODE,
        &salt,
        &nonce,
        &password_key(password, &salt)?,
        message,
    )
}
pub fn decode_password(sequence: &str, password: &str) -> Result<String> {
    open(sequence, PASSWORD_MODE, Some(password), None)
}
pub fn encode_split(message: &str) -> Result<(String, String, String)> {
    let key = random::<32>()?;
    let k1 = random::<32>()?;
    let k2: [u8; 32] = std::array::from_fn(|i| key[i] ^ k1[i]);
    let sequence = seal(SPLIT_MODE, &[], &random::<NONCE_LEN>()?, &key, message)?;
    Ok((sequence, BASE64.encode(k1), BASE64.encode(k2)))
}
pub fn decode_split(sequence: &str, k1: &str, k2: &str) -> Result<String> {
    let a = BASE64.decode(k1).map_err(|_| fail("Invalid K1"))?;
    let b = BASE64.decode(k2).map_err(|_| fail("Invalid K2"))?;
    if a.len() != 32 || b.len() != 32 {
        return Err(fail("Split keys must each be 32 bytes"));
    }
    let key: [u8; 32] = std::array::from_fn(|i| a[i] ^ b[i]);
    open(sequence, SPLIT_MODE, None, Some(&key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_interoperability_vectors() {
        let secure = "GCGTACGCGTACGCGTAAATAAAAAAATAAACAAAGAATAAATTAATCAATGAACAAACTAACCAACGAAGAAAGTAAGCAAGGAAAAAAATAAACAAAGAATAAATTAATCAATGAACAAACTAACCAACGCAGGCAAGCTTCTATACAAGCAGTGGGGTGCGAAAGAGGATGATTACTTACTTTCCTAGGAACAAGCCGGAG";
        assert_eq!(decode_password(secure, "password").unwrap(), "Hi");
        let salt = std::array::from_fn::<_, 16, _>(|i| i as u8);
        let nonce = std::array::from_fn::<_, 12, _>(|i| i as u8);
        assert_eq!(
            seal(
                PASSWORD_MODE,
                &salt,
                &nonce,
                &password_key("password", &salt).unwrap(),
                "Hi"
            )
            .unwrap(),
            secure
        );
        let split = "GCGTACGCGTACGCGTAAACAAAAAAATAAACAAAGAATAAATTAATCAATGAACAAACTAACCAACGAAGGTCCGGACAGTGAGCCGGTCCTCTGCAACTTACTGGTAGCTTATAAGTTCCACAGCCTATCTGTAACAG";
        let k1 = BASE64.encode([0u8; 32]);
        let k2 = BASE64.encode(std::array::from_fn::<_, 32, _>(|i| i as u8));
        assert_eq!(decode_split(split, &k1, &k2).unwrap(), "Hi");
        assert_eq!(
            seal(
                SPLIT_MODE,
                &[],
                &nonce,
                &std::array::from_fn(|i| i as u8),
                "Hi"
            )
            .unwrap(),
            split
        );
    }

    #[test]
    fn password_envelope_authenticates_text_and_header() {
        let sequence = encode_password("Hello, DNA 🧬", "correct horse").unwrap();
        assert_eq!(
            decode_password(&sequence, "correct horse").unwrap(),
            "Hello, DNA 🧬"
        );
        assert!(decode_password(&sequence, "wrong password").is_err());
        let mut damaged = sequence.into_bytes();
        let last = damaged.len() - 1;
        damaged[last] = if damaged[last] == b'A' { b'T' } else { b'A' };
        assert!(decode_password(std::str::from_utf8(&damaged).unwrap(), "correct horse").is_err());
    }

    #[test]
    fn split_envelope_requires_both_keys_and_detects_corruption() {
        let (sequence, k1, k2) = encode_split("Hello").unwrap();
        assert_eq!(decode_split(&sequence, &k1, &k2).unwrap(), "Hello");
        assert!(decode_split(&sequence, &k1, &BASE64.encode([0u8; 32])).is_err());
        let mut damaged = sequence.into_bytes();
        let index = PREFIX.len() + 8;
        damaged[index] = if damaged[index] == b'A' { b'T' } else { b'A' };
        assert!(decode_split(std::str::from_utf8(&damaged).unwrap(), &k1, &k2).is_err());
        assert!(decode_split(PREFIX, &k1, &k2).is_err());
    }
}
