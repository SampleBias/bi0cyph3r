# BioCypher Rust + Solana — Evaluation & Build Plan

> Historical document: web/Flask/API instructions below describe the retired architecture. See the [current terminal-only README](README.md) for setup and usage.

**Date**: February 17, 2025  
**Status**: Ready to Execute  
**Architecture**: Hybrid (Rust backend + Solana smart programs)

---

## 1. Executive Summary

This project migrates **BioCypher** — a DNA cryptography system — from Python/Flask to a **Rust backend** with **Solana blockchain** integration. The design uses a hybrid model:

- **Rust backend (Actix-web)**: Fast off-chain DNA encoding/decoding and safety screening
- **Solana programs**: On-chain encoding, decoding, and safety verification for attestation

### Current State (as of Feb 2025 — Phase 1 Complete)

| Component | Status | Notes |
|-----------|--------|-------|
| **Rust Backend** | ✅ Phase 1 complete | All 3 DNA modes working |
| **DNA Basic Mode** | ✅ Complete | 20+ unit tests passing |
| **DNA Nanopore Mode** | ✅ Complete | Triplet encoding, error correction, markers |
| **DNA Secure Mode** | ✅ Complete | AES-256-CBC + PBKDF2, password-based |
| **Safety Screener** | ✅ Complete | Pathogen, natural occurrence, characteristics |
| **API Endpoints** | ✅ Complete | encode, decode, safety-screen (all modes) |
| **CORS** | ✅ Added | Allow any origin for dev |
| **Solana Programs** | ❌ Not started | Encoder, Decoder, Safety programs |
| **Solana Integration** | ❌ Not started | No `solana-client` in Cargo.toml |
| **Frontend** | ⏳ Unknown | React/TypeScript referenced in docs |

---

## 2. Codebase Evaluation

### 2.1 What Exists

**Rust backend** (`biocypher-rust-solana/backend/`):

- **`main.rs`**: Actix-web server, health check, routes for `/api/encode`, `/api/decode`, `/api/safety-screen`
- **`dna/basic.rs`**: Full Basic mode (text↔binary↔DNA, 00=A, 01=T, 10=C, 11=G)
- **`dna/nanopore.rs`**: Triplet table defined; encode/decode return errors
- **`dna/secure.rs`**: Constants defined; encode/decode return errors
- **`dna/traits.rs`**: `DNACoder`, `SequenceStats`, `SequenceStatistics`
- **`safety/screener.rs`**: Pathogen detection, natural occurrence, GC/homopolymer/ORF analysis
- **`api/encode.rs`, `decode.rs`, `safety.rs`**: Request validation, mode dispatch, response models
- **`models.rs`**: Encode/Decode/Safety request/response structs
- **`error.rs`**: `BioCypherError`, `DNACryptoError`, `SafetyScreenerError`, Solana placeholder

**Workspace** (`biocypher-rust-solana/Cargo.toml`):

- Members: `backend` only; Solana programs commented out
- Solana deps commented: `solana-client`, `solana-sdk`, `anchor-client`

**Reference**:

- `biocypher/PROTOCOL_SPECIFICATION.md`: Encoding modes, mappings, test vectors
- `biocypher/nanopore_dna_crypto.py`, `secure_nanopore_dna_crypto.py`: Python reference
- `tasks/todo.md`: 200+ tasks; `tasks/DETAILED_IMPLEMENTATION_PLAN.md`: Technical plan

### 2.2 Gaps & Issues

1. **Nanopore mode**: Not implemented; API returns 501 for nanopore encode/decode.
2. **Secure mode**: Not implemented; API returns 501 for secure encode/decode.
3. **Solana**: No programs, no client integration, no `store_on_chain` / `decode_on_chain` / `verify_on_chain`.
4. **Secure mode API**: `SecureDNACrypto` has no password parameter; API passes password but crypto layer does not.
5. **Safety test**: `test_clean_sequence` expects `"ATCGNATCG"` but `clean_sequence` filters to A,T,C,G only → output is `"ATCGATCG"`. Test likely incorrect or N should be allowed.
6. **CORS**: Not configured in `main.rs` (only security headers).
7. **Build**: Requires network for `cargo build` (crates.io).

---

## 3. Build Plan — Phased Execution

### Phase 1: Complete Rust Backend (Week 1–2)

**Goal**: All three DNA modes working off-chain; API and tests green.

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| 1.1 Implement Nanopore mode | P0 | 2–3 days | `PROTOCOL_SPECIFICATION.md`, Python ref |
| 1.2 Implement Secure mode (AES-256-GCM + password) | P0 | 2 days | `secure_nanopore_dna_crypto.py` |
| 1.3 Wire Secure password through API → `SecureDNACrypto` | P0 | 0.5 day | 1.2 |
| 1.4 Fix safety `clean_sequence` test (N handling) | P1 | 0.5 day | — |
| 1.5 Add CORS middleware | P1 | 0.5 day | — |
| 1.6 Integration tests for encode/decode/safety | P1 | 1 day | 1.1–1.3 |
| 1.7 OpenAPI/Swagger spec | P2 | 0.5 day | — |

**Deliverables**:

- Nanopore and Secure modes implemented and tested
- All API modes (basic, nanopore, secure) working
- `cargo test` passing

---

### Phase 2: Solana Smart Programs (Week 2–3)

**Goal**: Anchor programs for encode, decode, and safety on Solana.

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| 2.1 Add Anchor to workspace; create `programs/` layout | P0 | 0.5 day | Anchor, Solana CLI |
| 2.2 `biocypher-encoder` program | P0 | 2 days | Basic encoding logic |
| 2.3 `biocypher-decoder` program | P0 | 1.5 days | Basic decoding logic |
| 2.4 `biocypher-safety` program | P0 | 1.5 days | Safety screening logic |
| 2.5 Deploy to devnet; write program tests | P0 | 1 day | 2.2–2.4 |
| 2.6 (Optional) Nanopore/Secure on-chain | P2 | 2 days | 2.2–2.4, Phase 1 |

**Deliverables**:

- Three Anchor programs building and deployable
- `anchor test` passing
- Programs on devnet

---

### Phase 3: Backend–Solana Integration (Week 3–4)

**Goal**: Backend can optionally store/verify on Solana.

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| 3.1 Add `solana-client`, `solana-sdk` to backend | P0 | 0.5 day | — |
| 3.2 Implement `solana/` module (client wrapper) | P0 | 2 days | Phase 2 |
| 3.3 Encode: `store_on_chain` → create DNARecord | P0 | 1 day | 3.2 |
| 3.4 Decode: `decode_on_chain` → call decoder program | P0 | 1 day | 3.2 |
| 3.5 Safety: `verify_on_chain` → store SafetyReport | P0 | 1 day | 3.2 |
| 3.6 `GET /api/verify/:signature` | P1 | 0.5 day | 3.2 |
| 3.7 (Optional) Redis caching | P2 | 1 day | — |

**Deliverables**:

- `store_on_chain`, `decode_on_chain`, `verify_on_chain` working
- Transaction signatures returned in API responses
- Verify endpoint for on-chain records

---

### Phase 4: Frontend & UX (Week 4–5)

**Goal**: UI supports all modes and optional blockchain features.

| Task | Priority | Effort | Dependencies |
|------|----------|--------|--------------|
| 4.1 Audit existing React app (if any) | P0 | 0.5 day | — |
| 4.2 Connect to Rust API | P0 | 1 day | Phase 1 |
| 4.3 Mode selector (basic/nanopore/secure) | P0 | 0.5 day | — |
| 4.4 Password input for Secure mode | P0 | 0.5 day | — |
| 4.5 “Store/verify on chain” toggles | P1 | 1 day | Phase 3 |
| 4.6 Solana wallet integration (optional) | P2 | 1–2 days | — |
| 4.7 Transaction/explorer links | P1 | 0.5 day | Phase 3 |

**Deliverables**:

- UI for encode, decode, safety
- Optional on-chain storage and verification
- Links to Solana Explorer for transactions

---

### Phase 5: Testing & Hardening (Week 5–6)

| Task | Priority | Effort |
|------|----------|--------|
| 5.1 End-to-end tests (API + optional Solana) | P0 | 1–2 days |
| 5.2 Load/performance tests | P1 | 1 day |
| 5.3 Security review (auth, input validation, crypto) | P0 | 1 day |
| 5.4 Documentation (API, deployment, env vars) | P1 | 0.5 day |

---

### Phase 6: Deployment (Week 6–7)

| Task | Priority | Effort |
|------|----------|--------|
| 6.1 Docker image for backend | P0 | 0.5 day |
| 6.2 Deploy backend (Fly.io, Railway, or similar) | P0 | 1 day |
| 6.3 Deploy frontend (Vercel, Netlify) | P0 | 0.5 day |
| 6.4 Deploy Solana programs to mainnet (if ready) | P1 | 0.5 day |
| 6.5 Monitoring, logging, alerts | P1 | 0.5 day |

---

## 4. Recommended Execution Order

```
Week 1:  Phase 1.1–1.3 (Nanopore + Secure modes)
Week 2:  Phase 1.4–1.7 + Phase 2.1–2.2 (Tests, CORS, Anchor setup, Encoder)
Week 3:  Phase 2.3–2.5 (Decoder, Safety programs, devnet)
Week 4:  Phase 3.1–3.6 (Solana integration)
Week 5:  Phase 4 (Frontend)
Week 6:  Phase 5 (Testing)
Week 7:  Phase 6 (Deployment)
```

---

## 5. Technical Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Web framework | Actix-web | Already in use; high performance |
| Solana framework | Anchor | Standard for Solana programs |
| Secure mode cipher | AES-256-GCM | Matches spec; authenticated encryption |
| Key derivation | PBKDF2, 100k iterations | Matches spec; good balance |
| On-chain scope (initial) | Basic mode only | Simpler; add nanopore/secure later |

---

## 6. Risk & Mitigation

| Risk | Mitigation |
|------|------------|
| Solana compute limits | Keep on-chain logic minimal; complex work in Rust |
| Password in Secure mode | Never log; use `SecretString`-style handling |
| Python/Rust output mismatch | Port test vectors from Python; add cross-check tests |
| Anchor/Solana version drift | Pin versions; test on devnet before mainnet |

---

## 7. Success Criteria

- [ ] All three DNA modes (basic, nanopore, secure) encode/decode correctly
- [ ] Safety screening matches expected behavior
- [ ] Solana programs deploy and run on devnet
- [ ] `store_on_chain` / `decode_on_chain` / `verify_on_chain` work end-to-end
- [ ] API response time &lt; 50 ms (off-chain), &lt; 800 ms (on-chain)
- [ ] Test suite passes; no critical security issues

---

## 8. Quick Start (Developer)

```bash
# Prerequisites
rustup default stable
# Solana CLI + Anchor (for Phase 2+)

# Build & run backend
cd biocypher-rust-solana
cargo build
cargo run

# Test
cargo test

# API
curl -X POST http://127.0.0.1:8080/api/encode \
  -H "Content-Type: application/json" \
  -d '{"message":"Hello","mode":"basic"}'
```

---

**Next step**: Start with Phase 1.1 — implement Nanopore mode in `dna/nanopore.rs` using `PROTOCOL_SPECIFICATION.md` and `nanopore_dna_crypto.py` as references.
