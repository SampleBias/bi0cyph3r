//! Solana client for biocypher-storage program.
//!
//! Builds and submits transactions for record_encode, record_decode, record_safety.
//! Uses Anchor instruction format: 8-byte discriminator + borsh args.

use crate::dna::EncodingMode;
use crate::error::{BioCypherError, Result};
use crate::models::SafetyStatus;
use sha2::{Digest, Sha256};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    system_program::ID as SYSTEM_PROGRAM_ID,
    transaction::Transaction,
};
use std::str::FromStr;
use std::sync::Arc;

/// Default program ID for biocypher-storage (from anchor keys list)
const DEFAULT_PROGRAM_ID: &str = "FtXEkJEXm8bJbEc9DHPwuV8W7C9PLdszt8vnzsDgk9Rj";

/// Compute Anchor instruction discriminator (first 8 bytes of sha256("global:name"))
fn instruction_discriminator(name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(b"global:");
    hasher.update(name.as_bytes());
    let result = hasher.finalize();
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&result[..8]);
    disc
}

fn encoding_mode_to_u8(mode: EncodingMode) -> u8 {
    match mode {
        EncodingMode::Basic => 0,
        EncodingMode::Nanopore => 1,
        EncodingMode::Secure => 2,
        EncodingMode::SplitKey => 3,
    }
}

fn safety_status_to_u8(status: SafetyStatus) -> u8 {
    match status {
        SafetyStatus::Safe => 0,
        SafetyStatus::Caution => 1,
        SafetyStatus::Unsafe => 2,
    }
}

/// Solana client for biocypher-storage attestation.
pub struct SolanaClient {
    rpc_url: String,
    keypair: Option<Keypair>,
    program_id: Pubkey,
}

impl SolanaClient {
    /// Create client from env vars. Returns None if Solana is disabled (no keypair).
    pub fn from_env() -> Option<Self> {
        let rpc_url =
            std::env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:8899".to_string());
        let keypair_path = std::env::var("SOLANA_KEYPAIR_PATH").unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            format!("{}/.config/solana/id.json", home)
        });
        let keypair = read_keypair_file(&keypair_path).ok()?;
        let program_id = Pubkey::from_str(
            std::env::var("BIOCYPHER_STORAGE_PROGRAM_ID")
                .as_deref()
                .unwrap_or(DEFAULT_PROGRAM_ID),
        )
        .ok()?;

        Some(Self {
            rpc_url,
            keypair: Some(keypair),
            program_id,
        })
    }

    /// Check if Solana is configured and available.
    pub fn is_available(&self) -> bool {
        self.keypair.is_some()
    }

    /// Wallet pubkey (server keypair) for status display.
    pub fn wallet_pubkey(&self) -> Option<Pubkey> {
        self.keypair.as_ref().map(|k| k.pubkey())
    }

    /// Program ID for attestation.
    pub fn program_id(&self) -> Pubkey {
        self.program_id
    }

    /// RPC URL in use.
    pub fn rpc_url(&self) -> &str {
        &self.rpc_url
    }

    fn payer(&self) -> Result<&Keypair> {
        self.keypair
            .as_ref()
            .ok_or_else(|| BioCypherError::Solana("Solana keypair not configured".into()))
    }

    fn encode_record_pda(&self, sequence_hash: &[u8; 32]) -> Result<Pubkey> {
        let (pda, _) = Pubkey::find_program_address(
            &[b"encode", self.payer()?.pubkey().as_ref(), sequence_hash],
            &self.program_id,
        );
        Ok(pda)
    }

    fn decode_record_pda(&self, sequence_hash: &[u8; 32]) -> Result<Pubkey> {
        let (pda, _) = Pubkey::find_program_address(
            &[b"decode", self.payer()?.pubkey().as_ref(), sequence_hash],
            &self.program_id,
        );
        Ok(pda)
    }

    fn safety_record_pda(&self, sequence_hash: &[u8; 32]) -> Result<Pubkey> {
        let (pda, _) = Pubkey::find_program_address(
            &[b"safety", self.payer()?.pubkey().as_ref(), sequence_hash],
            &self.program_id,
        );
        Ok(pda)
    }

    async fn send_transaction(&self, instruction: Instruction) -> Result<String> {
        let payer = self.payer()?;
        let client = Arc::new(RpcClient::new_with_commitment(
            self.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        ));

        let recent_blockhash = client
            .get_latest_blockhash()
            .await
            .map_err(|e| BioCypherError::Solana(e.to_string()))?;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&payer.pubkey()),
            &[payer],
            recent_blockhash,
        );

        let sig = client
            .send_and_confirm_transaction_with_spinner_and_config(
                &transaction,
                CommitmentConfig::confirmed(),
                solana_client::rpc_config::RpcSendTransactionConfig::default(),
            )
            .await
            .map_err(|e| BioCypherError::Solana(e.to_string()))?;

        Ok(sig.to_string())
    }

    /// Record encode attestation on-chain.
    pub async fn record_encode(
        &self,
        mode: EncodingMode,
        sequence_hash: [u8; 32],
    ) -> Result<String> {
        let payer = self.payer()?;
        let encode_record = self.encode_record_pda(&sequence_hash)?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut data = instruction_discriminator("record_encode").to_vec();
        data.push(encoding_mode_to_u8(mode));
        data.extend_from_slice(&sequence_hash);
        data.extend_from_slice(&timestamp.to_le_bytes());

        let ix = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(encode_record, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ],
            data,
        };

        self.send_transaction(ix).await
    }

    /// Record decode attestation on-chain.
    pub async fn record_decode(
        &self,
        mode: EncodingMode,
        sequence_hash: [u8; 32],
    ) -> Result<String> {
        let payer = self.payer()?;
        let decode_record = self.decode_record_pda(&sequence_hash)?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut data = instruction_discriminator("record_decode").to_vec();
        data.push(encoding_mode_to_u8(mode));
        data.extend_from_slice(&sequence_hash);
        data.extend_from_slice(&timestamp.to_le_bytes());

        let ix = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(decode_record, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ],
            data,
        };

        self.send_transaction(ix).await
    }

    /// Record safety attestation on-chain.
    pub async fn record_safety(
        &self,
        sequence_hash: [u8; 32],
        status: SafetyStatus,
    ) -> Result<String> {
        let payer = self.payer()?;
        let safety_record = self.safety_record_pda(&sequence_hash)?;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut data = instruction_discriminator("record_safety").to_vec();
        data.extend_from_slice(&sequence_hash);
        data.push(safety_status_to_u8(status));
        data.extend_from_slice(&timestamp.to_le_bytes());

        let ix = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(safety_record, false),
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
            ],
            data,
        };

        self.send_transaction(ix).await
    }
}

/// Hash a DNA sequence with SHA-256 for on-chain attestation.
pub fn hash_sequence(sequence: &str) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(sequence.as_bytes());
    let result = hasher.finalize();
    let mut arr = [0u8; 32];
    arr.copy_from_slice(&result);
    arr
}

/// Build unsigned attestation transaction for user wallet signing.
/// Does not require server keypair; uses RPC for blockhash only.
pub async fn build_attest_transaction(
    payer: Pubkey,
    operation: &str,
    sequence_hash: [u8; 32],
    mode: Option<EncodingMode>,
    status: Option<SafetyStatus>,
) -> Result<Vec<u8>> {
    let rpc_url =
        std::env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "http://127.0.0.1:8899".to_string());
    let program_id = Pubkey::from_str(
        std::env::var("BIOCYPHER_STORAGE_PROGRAM_ID")
            .as_deref()
            .unwrap_or(DEFAULT_PROGRAM_ID),
    )
    .map_err(|_| BioCypherError::Solana("Invalid program ID".into()))?;

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let ix = match operation {
        "encode" => {
            let mode =
                mode.ok_or_else(|| BioCypherError::Solana("mode required for encode".into()))?;
            let (encode_record, _) = Pubkey::find_program_address(
                &[b"encode", payer.as_ref(), &sequence_hash],
                &program_id,
            );
            let mut data = instruction_discriminator("record_encode").to_vec();
            data.push(encoding_mode_to_u8(mode));
            data.extend_from_slice(&sequence_hash);
            data.extend_from_slice(&timestamp.to_le_bytes());
            Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new(encode_record, false),
                    AccountMeta::new(payer, true),
                    AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                ],
                data,
            }
        }
        "decode" => {
            let mode =
                mode.ok_or_else(|| BioCypherError::Solana("mode required for decode".into()))?;
            let (decode_record, _) = Pubkey::find_program_address(
                &[b"decode", payer.as_ref(), &sequence_hash],
                &program_id,
            );
            let mut data = instruction_discriminator("record_decode").to_vec();
            data.push(encoding_mode_to_u8(mode));
            data.extend_from_slice(&sequence_hash);
            data.extend_from_slice(&timestamp.to_le_bytes());
            Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new(decode_record, false),
                    AccountMeta::new(payer, true),
                    AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                ],
                data,
            }
        }
        "safety" => {
            let status = status
                .ok_or_else(|| BioCypherError::Solana("status required for safety".into()))?;
            let (safety_record, _) = Pubkey::find_program_address(
                &[b"safety", payer.as_ref(), &sequence_hash],
                &program_id,
            );
            let mut data = instruction_discriminator("record_safety").to_vec();
            data.extend_from_slice(&sequence_hash);
            data.push(safety_status_to_u8(status));
            data.extend_from_slice(&timestamp.to_le_bytes());
            Instruction {
                program_id,
                accounts: vec![
                    AccountMeta::new(safety_record, false),
                    AccountMeta::new(payer, true),
                    AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                ],
                data,
            }
        }
        _ => {
            return Err(BioCypherError::Solana(format!("Unknown operation: {}", operation)).into())
        }
    };

    let client = Arc::new(RpcClient::new_with_commitment(
        rpc_url,
        CommitmentConfig::confirmed(),
    ));
    let blockhash = client
        .get_latest_blockhash()
        .await
        .map_err(|e| BioCypherError::Solana(e.to_string()))?;

    let mut transaction = Transaction::new_with_payer(&[ix], Some(&payer));
    transaction.message.recent_blockhash = blockhash;
    bincode::serialize(&transaction).map_err(|e| BioCypherError::Solana(e.to_string()))
}
