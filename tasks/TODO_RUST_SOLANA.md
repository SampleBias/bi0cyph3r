# BioCypher Rust + Solana Migration - Detailed Plan

> Historical document: web/Flask/API instructions below describe the retired architecture. See the [current terminal-only README](../README.md) for setup and usage.

## Option B: Hybrid Architecture (RECOMMENDED)

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                          UI / Frontend                               │
│                      (React/TypeScript)                              │
└────────────────────────────┬────────────────────────────────────────┘
                             │ HTTP/WebSocket
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Rust Backend Service                            │
│                      (Actix-web / Axum)                              │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  API Gateway Layer                                            │  │
│  │  - Authentication & Authorization                            │  │
│  │  - Rate Limiting                                              │  │
│  │  - Input Validation                                           │  │
│  │  - Request/Response Transformation                            │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Business Logic Layer                                        │  │
│  │  - DNA Encoding/Decoding (off-chain for speed)               │  │
│  │  - Safety Screening (off-chain for complex analysis)         │  │
│  │  - Session Management                                        │  │
│  │  - Caching Layer                                             │  │
│  └──────────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  Solana Integration Layer                                     │  │
│  │  - Solana RPC Client                                         │  │
│  │  - Transaction Builder                                       │  │
│  │  - Account Management                                         │  │
│  │  - Program Invocation                                         │  │
│  └──────────────────────────────────────────────────────────────┘  │
└────────────────────────────┬────────────────────────────────────────┘
                             │ Solana RPC
                             ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         Solana Blockchain                            │
│                                                                      │
│  ┌─────────────────────┐  ┌─────────────────────┐                  │
│  │  DNA Encoder        │  │  DNA Decoder        │                  │
│  │  Smart Program      │  │  Smart Program      │                  │
│  │                     │  │                     │                  │
│  │  - Basic Mode       │  │  - Basic Mode       │                  │
│  │  - Nanopore Mode    │  │  - Nanopore Mode    │                  │
│  │  - Secure Mode      │  │  - Secure Mode      │                  │
│  └─────────────────────┘  └─────────────────────┘                  │
│                                                                     │
│  ┌─────────────────────┐  ┌─────────────────────┐                  │
│  │  Safety Screener    │  │  DNA Storage        │                  │
│  │  Smart Program      │  │  Program            │                  │
│  │                     │  │                     │                  │
│  │  - Pathogen Detect  │  │  - Data Storage    │                  │
│  │  - Risk Assessment  │  │  - Retrieval       │                  │
│  │  - Attestation      │  │  - History         │                  │
│  └─────────────────────┘  └─────────────────────┘                  │
│                                                                     │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Data Accounts                            │   │
│  │  - Encoded DNA Records                                       │   │
│  │  - Safety Screening Reports                                   │   │
│  │  - User Data                                                  │   │
│  │  - Transaction History                                       │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

---

## YES, We Can Build This! Here's How:

### Phase 1: Rust Backend Foundation (Week 1-2)

#### 1.1 Project Setup
```toml
# Cargo.toml
[package]
name = "biocypher-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
# Web Framework
actix-web = "4.4"
actix-cors = "0.6"
tokio = { version = "1.35", features = ["full"] }

# Solana Integration
solana-client = "1.17"
solana-sdk = "1.17"
anchor-client = "0.29"
anchor-lang = "0.29"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Cryptography
aes-gcm = "0.10"
pbkdf2 = { version = "0.12", features = ["password-hash"] }
sha2 = "0.10"
rand = "0.8"

# Utilities
thiserror = "1.0"
anyhow = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"

# DNA Processing
regex = "1.10"
```

#### 1.2 Core DNA Crypto in Rust

```rust
// src/dna/crypto.rs
pub struct DNACrypto;

impl DNACrypto {
    const DNA_ENCODE: [(u8, &str); 4] = [
        (0b00, "A"),
        (0b01, "T"),
        (0b10, "C"),
        (0b11, "G"),
    ];
    
    pub fn encode_message(message: &str) -> Result<String, DNACryptoError> {
        let mut binary = String::new();
        for byte in message.as_bytes() {
            binary.push_str(&format!("{:08b}", byte));
        }
        
        let mut dna = String::new();
        for chunk in binary.as_bytes().chunks(2) {
            if chunk.len() == 2 {
                let pair = format!("{}{}", chunk[0] as char, chunk[1] as char);
                let bits = u8::from_str_radix(&pair, 2)?;
                let base = Self::DNA_ENCODE
                    .iter()
                    .find(|(b, _)| *b == bits)
                    .map(|(_, s)| *s)
                    .ok_or(DNACryptoError::InvalidBinaryPair)?;
                dna.push_str(base);
            }
        }
        Ok(dna)
    }
    
    pub fn decode_sequence(sequence: &str) -> Result<String, DNACryptoError> {
        let mut binary = String::new();
        for base in sequence.chars() {
            let bits = match base {
                'A' => "00",
                'T' => "01",
                'C' => "10",
                'G' => "11",
                _ => continue,
            };
            binary.push_str(bits);
        }
        
        let mut message = String::new();
        for chunk in binary.as_bytes().chunks(8) {
            if chunk.len() == 8 {
                let byte_str = std::str::from_utf8(chunk)?;
                let byte_val = u8::from_str_radix(byte_str, 2)?;
                if byte_val >= 32 && byte_val <= 126 {
                    message.push(byte_val as char);
                }
            }
        }
        Ok(message)
    }
}
```

#### 1.3 API Endpoints (Actix-web)

```rust
// src/api/routes.rs
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct EncodeRequest {
    message: String,
    mode: String,  // "basic", "nanopore", "secure"
    password: Option<String>,
}

#[derive(Serialize)]
struct EncodeResponse {
    dna_sequence: String,
    transaction_signature: Option<String>,  // If stored on-chain
    stats: SequenceStats,
}

pub async fn encode_message(
    req: web::Json<EncodeRequest>,
    solana_client: web::Data<SolanaClient>,
) -> impl Responder {
    match req.mode.as_str() {
        "basic" => {
            let dna = DNACrypto::encode_message(&req.message)?;
            // Optionally store on Solana
            let tx_sig = if req.store_on_chain {
                Some(solana_client.store_dna(&dna).await?)
            } else {
                None
            };
            Ok(HttpResponse::Ok().json(EncodeResponse {
                dna_sequence: dna,
                transaction_signature: tx_sig,
                stats: DNACrypto::get_stats(&dna),
            }))
        }
        "nanopore" => {
            let dna = NanoporeDNACrypto::encode(&req.message, true)?;
            Ok(HttpResponse::Ok().json(EncodeResponse {
                dna_sequence: dna,
                transaction_signature: None,
                stats: NanoporeDNACrypto::get_stats(&dna),
            }))
        }
        "secure" => {
            let password = req.password.ok_or(Error::PasswordRequired)?;
            let dna = SecureDNACrypto::encode(&req.message, &password)?;
            Ok(HttpResponse::Ok().json(EncodeResponse {
                dna_sequence: dna,
                transaction_signature: None,
                stats: DNACrypto::get_stats(&dna),
            }))
        }
        _ => Err(Error::InvalidMode),
    }
}
```

---

### Phase 2: Solana Smart Programs (Week 2-4)

#### 2.1 Project Structure (Anchor Framework)

```
programs/
├── biocypher-encoder/
│   ├── src/
│   │   ├── lib.rs           # Main program
│   │   ├── state.rs         # Account structures
│   │   ├── instructions.rs  # Instruction handlers
│   │   └── processor.rs    # Core logic
│   ├── Cargo.toml
│   └── Xargo.toml
├── biocypher-decoder/
│   └── [similar structure]
├── biocypher-safety/
│   └── [similar structure]
└── biocypher-storage/
    └── [similar structure]
```

#### 2.2 DNA Encoder Program

```rust
// programs/biocypher-encoder/src/lib.rs
use anchor_lang::prelude::*;

declare_id!("Bi0EnCoDeRxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

#[program]
pub mod biocypher_encoder {
    use super::*;

    /// Encode a message to DNA and store on-chain
    pub fn encode_basic(
        ctx: Context<EncodeBasic>,
        message: String,
    ) -> Result<()> {
        let dna_record = &mut ctx.accounts.dna_record;
        let encoder = DNAEncoder::new();

        // Encode message to DNA
        let dna_sequence = encoder.encode_basic(&message)?;

        // Store in account
        dna_record.owner = ctx.accounts.authority.key();
        dna_record.mode = EncodingMode::Basic;
        dna_record.message = message;
        dna_record.dna_sequence = dna_sequence.clone();
        dna_record.timestamp = Clock::get()?.unix_timestamp;

        msg!("Encoded message to DNA: {}", dna_sequence);
        Ok(())
    }

    /// Encode with nanopore optimization
    pub fn encode_nanopore(
        ctx: Context<EncodeNanopore>,
        message: String,
        use_error_correction: bool,
    ) -> Result<()> {
        let dna_record = &mut ctx.accounts.dna_record;
        let encoder = DNAEncoder::new();

        let dna_sequence = encoder.encode_nanopore(&message, use_error_correction)?;

        dna_record.owner = ctx.accounts.authority.key();
        dna_record.mode = EncodingMode::Nanopore;
        dna_record.message = message;
        dna_record.dna_sequence = dna_sequence.clone();
        dna_record.use_error_correction = use_error_correction;
        dna_record.timestamp = Clock::get()?.unix_timestamp;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct EncodeBasic<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + DNARecord::INIT_SPACE,
        seeds = [b"dna", authority.key().as_ref()],
        bump
    )]
    pub dna_record: Account<'info, DNARecord>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct DNARecord {
    pub owner: Pubkey,
    pub mode: EncodingMode,
    pub message: String,      // Original message (max 1000 chars)
    pub dna_sequence: String,  // Encoded DNA (max 10000 bases)
    pub use_error_correction: bool,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum EncodingMode {
    Basic,
    Nanopore,
    Secure,
}
```

#### 2.3 On-Chain DNA Encoding Logic

```rust
// programs/biocypher-encoder/src/processor.rs
use anchor_lang::prelude::*;

pub struct DNAEncoder;

impl DNAEncoder {
    pub fn encode_basic(&self, message: &str) -> Result<String> {
        let mut binary = String::new();

        // Convert message to binary
        for byte in message.as_bytes() {
            binary.push_str(&format!("{:08b}", byte));
        }

        // Convert binary to DNA
        let mut dna = String::new();
        let mut chars = binary.chars().collect::<Vec<_>>();

        for chunk in chars.chunks(2) {
            if chunk.len() == 2 {
                let bits = format!("{}{}", chunk[0], chunk[1]);
                let base = self.bits_to_base(&bits)?;
                dna.push(base);
            }
        }

        Ok(dna)
    }

    fn bits_to_base(&self, bits: &str) -> Result<char> {
        match bits {
            "00" => Ok('A'),
            "01" => Ok('T'),
            "10" => Ok('C'),
            "11" => Ok('G'),
            _ => err!(ErrorCode::InvalidBinary),
        }
    }

    pub fn encode_nanopore(&self, message: &str, use_ec: bool) -> Result<String> {
        // Step 1: Text to binary with parity
        let mut binary = String::new();
        for byte in message.as_bytes() {
            let bin = format!("{:08b}", byte);
            let parity = if bin.chars().filter(|c| *c == '1').count() % 2 == 0 {
                '0'
            } else {
                '1'
            };
            binary.push_str(&format!("{}{}", bin, parity));
        }

        // Step 2: Error correction (triple redundancy)
        if use_ec {
            let mut corrected = String::new();
            for bit in binary.chars() {
                corrected.push_str(&bit.to_string().repeat(3));
            }
            binary = corrected;
        }

        // Step 3: Convert to DNA using triplet encoding
        let mut dna = String::new();
        let mut chars = binary.chars().collect::<Vec<_>>();

        // Pad to multiple of 3
        while chars.len() % 3 != 0 {
            chars.push('0');
        }

        for chunk in chars.chunks(3) {
            if chunk.len() == 3 {
                let triplet = format!("{}{}{}", chunk[0], chunk[1], chunk[2]);
                let dna_triplet = self.triplet_to_dna(&triplet)?;
                dna.push_str(&dna_triplet);
            }
        }

        // Step 4: Add markers
        Ok(format!("ATCGATCG{}CGATATCG", dna))
    }

    fn triplet_to_dna(&self, triplet: &str) -> Result<String> {
        match triplet {
            "000" => Ok("ATC".to_string()),
            "001" => Ok("ATG".to_string()),
            "010" => Ok("ACT".to_string()),
            "011" => Ok("ACG".to_string()),
            "100" => Ok("TAG".to_string()),
            "101" => Ok("TAC".to_string()),
            "110" => Ok("TCG".to_string()),
            "111" => Ok("TCA".to_string()),
            _ => err!(ErrorCode::InvalidTriplet),
        }
    }
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid binary pair")]
    InvalidBinary,
    #[msg("Invalid triplet")]
    InvalidTriplet,
    #[msg("Message too long")]
    MessageTooLong,
}
```

#### 2.4 DNA Decoder Program

```rust
// programs/biocypher-decoder/src/lib.rs
use anchor_lang::prelude::*;

declare_id!("Bi0DeCoDeRxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

#[program]
pub mod biocypher_decoder {
    use super::*;

    pub fn decode_basic(
        ctx: Context<DecodeBasic>,
        dna_sequence: String,
    ) -> Result<()> {
        let decoder = DNADecoder::new();
        let message = decoder.decode_basic(&dna_sequence)?;

        let result = &mut ctx.accounts.decode_result;
        result.dna_sequence = dna_sequence;
        result.decoded_message = message;
        result.timestamp = Clock::get()?.unix_timestamp;

        Ok(())
    }

    pub fn decode_nanopore(
        ctx: Context<DecodeNanopore>,
        dna_sequence: String,
        use_error_correction: bool,
    ) -> Result<()> {
        let decoder = DNADecoder::new();
        let message = decoder.decode_nanopore(&dna_sequence, use_error_correction)?;

        let result = &mut ctx.accounts.decode_result;
        result.dna_sequence = dna_sequence;
        result.decoded_message = message;
        result.use_error_correction = use_error_correction;
        result.timestamp = Clock::get()?.unix_timestamp;

        Ok(())
    }
}
```

#### 2.5 Safety Screener Program

```rust
// programs/biocypher-safety/src/lib.rs
use anchor_lang::prelude::*;

declare_id!("Bi0S4FeTyxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

#[program]
pub mod biocypher_safety {
    use super::*;

    pub fn screen_sequence(
        ctx: Context<ScreenSequence>,
        dna_sequence: String,
    ) -> Result<()> {
        let screener = DNASafetyScreener::new();

        // Perform comprehensive screening
        let pathogen_check = screener.check_pathogens(&dna_sequence)?;
        let natural_check = screener.check_natural_occurrence(&dna_sequence)?;
        let characteristics = screener.analyze_characteristics(&dna_sequence)?;

        // Determine overall safety
        let safety_status = screener.determine_status(
            &pathogen_check,
            &natural_check,
            &characteristics,
        );

        let report = &mut ctx.accounts.safety_report;
        report.dna_sequence = dna_sequence;
        report.safety_status = safety_status;
        report.pathogen_risk = pathogen_check.risk;
        report.natural_occurrence = natural_check.found;
        report.gc_content = characteristics.gc_content;
        report.homopolymer_count = characteristics.homopolymer_count;
        report.timestamp = Clock::get()?.unix_timestamp;

        msg!("Safety screening complete: {:?}", safety_status);
        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct SafetyReport {
    pub dna_sequence: String,  // Max 10000 bases
    pub safety_status: SafetyStatus,
    pub pathogen_risk: bool,
    pub natural_occurrence: bool,
    pub gc_content: u8,
    pub homopolymer_count: u32,
    pub timestamp: i64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, InitSpace)]
pub enum SafetyStatus {
    Safe,
    Caution,
    Unsafe,
}
```

---

### Phase 3: Solana Integration in Rust Backend (Week 3-4)

#### 3.1 Solana Client Wrapper

```rust
// src/solana/client.rs
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use anchor_client::{Program, Client};
use anyhow::Result;

pub struct SolanaClient {
    rpc_client: RpcClient,
    program_id: Pubkey,
    payer: Keypair,
}

impl SolanaClient {
    pub fn new(rpc_url: &str, program_id: &str, payer_keypair: &[u8]) -> Result<Self> {
        let rpc_client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
        let program_id = Pubkey::from_str(program_id)?;
        let payer = Keypair::from_bytes(payer_keypair)?;

        Ok(Self {
            rpc_client,
            program_id,
            payer,
        })
    }

    pub async fn store_dna(
        &self,
        message: &str,
        mode: EncodingMode,
    ) -> Result<String> {
        // Create instruction
        let instruction = create_encode_instruction(
            self.program_id,
            self.payer.pubkey(),
            message.to_string(),
            mode,
        )?;

        // Build transaction
        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            recent_blockhash,
        );

        // Send transaction
        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    pub async fn retrieve_dna(&self, account_pubkey: &Pubkey) -> Result<DNARecord> {
        let account_data = self.rpc_client.get_account_data(account_pubkey)?;
        let record = DNARecord::try_from_slice(&account_data)?;
        Ok(record)
    }

    pub async fn screen_dna_safety(&self, dna_sequence: &str) -> Result<SafetyReport> {
        let instruction = create_screen_instruction(
            self.program_id,
            self.payer.pubkey(),
            dna_sequence.to_string(),
        )?;

        let recent_blockhash = self.rpc_client.get_latest_blockhash()?;
        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.payer.pubkey()),
            &[&self.payer],
            recent_blockhash,
        );

        let signature = self.rpc_client.send_and_confirm_transaction(&transaction)?;

        // Get the created report account
        let (report_pubkey, _bump) = Pubkey::find_program_address(
            &[b"safety", self.payer.pubkey().as_ref(), signature.as_ref()],
            &self.program_id,
        );

        let report = self.retrieve_safety_report(&report_pubkey).await?;
        Ok(report)
    }
}
```

#### 3.2 Integration with API Routes

```rust
// src/api/handlers.rs
use actix_web::{web, HttpResponse};
use crate::solana::SolanaClient;

pub async fn encode_with_blockchain(
    req: web::Json<EncodeRequest>,
    solana: web::Data<SolanaClient>,
) -> Result<HttpResponse, Error> {
    // Step 1: Encode off-chain (fast)
    let dna = match req.mode.as_str() {
        "basic" => DNACrypto::encode_message(&req.message)?,
        "nanopore" => NanoporeDNACrypto::encode(&req.message, true)?,
        "secure" => {
            let password = req.password.as_ref().ok_or(Error::PasswordRequired)?;
            SecureDNACrypto::encode(&req.message, password)?
        }
        _ => return Err(Error::InvalidMode),
    };

    // Step 2: Store on Solana (optional, based on request)
    let tx_signature = if req.store_on_chain.unwrap_or(false) {
        Some(solana.store_dna(&req.message, req.mode.clone().into()).await?)
    } else {
        None
    };

    // Step 3: Return result
    Ok(HttpResponse::Ok().json(EncodeResponse {
        dna_sequence: dna,
        transaction_signature: tx_signature,
        stats: get_stats(&dna, &req.mode),
    }))
}

pub async fn safety_screen_on_chain(
    req: web::Json<SafetyScreenRequest>,
    solana: web::Data<SolanaClient>,
) -> Result<HttpResponse, Error> {
    // Perform screening on Solana (immutable record)
    let report = solana.screen_dna_safety(&req.dna_sequence).await?;

    Ok(HttpResponse::Ok().json(SafetyScreenResponse {
        dna_sequence: report.dna_sequence,
        safety_status: report.safety_status,
        pathogen_risk: report.pathogen_risk,
        natural_occurrence: report.natural_occurrence,
        gc_content: report.gc_content,
        transaction_signature: report.signature,
    }))
}
```

---

### Phase 4: UI Integration (Week 4-5)

#### 4.1 Frontend API Client (TypeScript)

```typescript
// frontend/src/api/biocypher.ts

interface EncodeRequest {
  message: string;
  mode: 'basic' | 'nanopore' | 'secure';
  password?: string;
  store_on_chain?: boolean;
}

interface EncodeResponse {
  dna_sequence: string;
  transaction_signature?: string;
  stats: SequenceStats;
}

class BioCypherAPI {
  private baseURL: string;

  constructor(baseURL: string = 'http://localhost:8080') {
    this.baseURL = baseURL;
  }

  async encodeMessage(request: EncodeRequest): Promise<EncodeResponse> {
    const response = await fetch(`${this.baseURL}/api/encode`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(request),
    });

    if (!response.ok) throw new Error('Encoding failed');
    return response.json();
  }

  async encodeMessageOnChain(request: EncodeRequest): Promise<EncodeResponse> {
    request.store_on_chain = true;
    return this.encodeMessage(request);
  }

  async safetyScreenOnChain(dna_sequence: string) {
    const response = await fetch(`${this.baseURL}/api/safety-screen/on-chain`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ dna_sequence }),
    });

    if (!response.ok) throw new Error('Safety screening failed');
    return response.json();
  }

  async verifyOnChainTransaction(signature: string) {
    const response = await fetch(
      `${this.baseURL}/api/verify/${signature}`
    );
    return response.json();
  }
}
```

#### 4.2 React Component

```typescript
// frontend/src/components/EncodeForm.tsx

import React, { useState } from 'react';
import { BioCypherAPI } from '../api/biocypher';

export function EncodeForm() {
  const [message, setMessage] = useState('');
  const [mode, setMode] = useState<'basic' | 'nanopore' | 'secure'>('basic');
  const [password, setPassword] = useState('');
  const [storeOnChain, setStoreOnChain] = useState(false);
  const [result, setResult] = useState<any>(null);
  const [loading, setLoading] = useState(false);

  const api = new BioCypherAPI();

  const handleEncode = async () => {
    setLoading(true);
    try {
      const response = await api.encodeMessage({
        message,
        mode,
        password: mode === 'secure' ? password : undefined,
        store_on_chain: storeOnChain,
      });
      setResult(response);
    } catch (error) {
      console.error('Encoding failed:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="encode-form">
      <h2>Encode Message to DNA</h2>

      <textarea
        value={message}
        onChange={(e) => setMessage(e.target.value)}
        placeholder="Enter your message..."
        rows={5}
      />

      <select value={mode} onChange={(e) => setMode(e.target.value as any)}>
        <option value="basic">Basic Mode</option>
        <option value="nanopore">Nanopore Optimized</option>
        <option value="secure">Secure (Encrypted)</option>
      </select>

      {mode === 'secure' && (
        <input
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          placeholder="Enter encryption password"
        />
      )}

      <label>
        <input
          type="checkbox"
          checked={storeOnChain}
          onChange={(e) => setStoreOnChain(e.target.checked)}
        />
        Store on Solana Blockchain
      </label>

      <button onClick={handleEncode} disabled={loading}>
        {loading ? 'Encoding...' : 'Encode'}
      </button>

      {result && (
        <div className="result">
          <h3>DNA Sequence:</h3>
          <textarea value={result.dna_sequence} readOnly rows={10} />

          <div className="stats">
            <p>Length: {result.stats.length} bases</p>
            <p>GC Content: {result.stats.gc_content}%</p>
          </div>

          {result.transaction_signature && (
            <div className="blockchain-info">
              <h4>✓ Stored on Solana</h4>
              <p>Transaction: {result.transaction_signature}</p>
              <a
                href={`https://explorer.solana.com/tx/${result.transaction_signature}`}
                target="_blank"
                rel="noopener noreferrer"
              >
                View on Explorer
              </a>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
```

---

## Benefits of This Hybrid Approach

### ✅ Advantages

1. **Performance**
   - Rust backend handles fast encoding/decoding off-chain
   - Solana only used when blockchain benefits are needed
   - Caching reduces redundant operations

2. **Flexibility**
   - Client can choose between fast off-chain or verifiable on-chain
   - Gradual migration path
   - Can fall back to off-chain if Solana is congested

3. **User Experience**
   - Quick response times for most operations
   - Optional blockchain verification for important data
   - No mandatory wallet interactions for basic use

4. **Security & Trust**
   - Immutable records on Solana for critical operations
   - Cryptographic verification of data integrity
   - Decentralized attestation for safety screenings

5. **Cost Efficiency**
   - Only pay for on-chain operations when needed
   - Bulk operations can be batched
   - Free tier possible with off-chain operations

6. **Scalability**
   - Horizontal scaling of Rust backend
   - Solana handles high-throughput when needed
   - Can add more Solana programs as features grow

---

## Deployment Strategy

### Development Environment
```bash
# Start local Solana validator
solana-test-validator

# Start Rust backend
cargo run

# Start UI (from frontend directory)
npm start
```

### Production Architecture
```
Internet
    │
    ▼
┌─────────────────┐
│  CDN / LB       │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
┌───▼───┐ ┌──▼────┐
│ Rust  │ │ Rust  │
│ API 1 │ │ API 2 │  (Horizontal Scaling)
└───┬───┘ └──┬────┘
    │         │
    └────┬────┘
         │
         ▼
┌─────────────────┐
│  Redis Cache    │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
┌───▼──────────▼───┐
│  Solana RPC     │
│  (Mainnet/Devnet)│
└──────────────────┘
```

---

## Estimated Timeline

| Phase | Duration | Deliverables |
|-------|----------|--------------|
| 1. Rust Backend | 2 weeks | API server, DNA crypto, basic routes |
| 2. Solana Programs | 2 weeks | Encoder, decoder, safety programs |
| 3. Integration | 1 week | Solana client, unified API |
| 4. UI Updates | 1 week | React components, blockchain UI |
| 5. Testing | 1 week | Integration tests, end-to-end tests |
| **Total** | **7 weeks** | **Fully functional hybrid system** |

---

## Next Steps

1. **Set up Rust project structure** - Initialize Cargo workspace
2. **Implement DNA crypto in Rust** - Port Python algorithms
3. **Create Actix-web server** - Basic API endpoints
4. **Set up Anchor project** - Initialize Solana programs
5. **Implement Encoder program** - First smart program
6. **Integrate with backend** - Connect Rust to Solana
7. **Update UI** - Add blockchain features
8. **Deploy to devnet** - Test on Solana devnet
9. **Security audit** - Review all components
10. **Mainnet deployment** - Production launch

---

**YES, we can absolutely build this!** The hybrid approach gives us the best of both worlds - performance and flexibility from Rust, with the trust and immutability of Solana when needed. 

Ready to start implementation? Let's begin with Phase 1!
