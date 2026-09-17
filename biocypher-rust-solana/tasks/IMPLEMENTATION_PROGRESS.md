# BioCypher Rust + Solana Migration - Task Tracking

> Historical document: web/Flask/API instructions below describe the retired architecture. See the [current terminal-only README](../../README.md) for setup and usage.

## Phase 1: Rust Backend Foundation (Week 1-2)

### Project Setup
- [x] Initialize Cargo workspace structure
- [x] Create backend/Cargo.toml with all dependencies
- [x] Set up project directory structure (src/dna, src/safety, src/solana, src/api, src/models)
- [x] Configure logging with tracing
- [x] Set up error handling with thiserror
- [x] Create config management module
- [x] Set up basic Actix-web server with health check endpoint

### DNA Crypto Implementation
- [x] Port Basic DNA Crypto from Python to Rust (dna/basic.rs)
  - [x] Implement text_to_binary()
  - [x] Implement binary_to_dna()
  - [x] Implement dna_to_binary()
  - [x] Implement binary_to_text()
  - [x] Implement encode_message()
  - [x] Implement decode_sequence()
  - [x] Implement get_sequence_stats()
  - [x] Write unit tests (10 test cases from Python)
  - [ ] Fix test failures (5 tests failing)

- [ ] Port Nanopore DNA Crypto from Python to Rust (dna/nanopore.rs)
  - [ ] Implement text_to_binary with parity
  - [ ] Implement triplet encoding (8 triplets)
  - [ ] Implement error correction (triple redundancy)
  - [ ] Implement homopolymer detection
  - [ ] Implement GC content balancing
  - [ ] Implement nanopore markers (ATCGATCG/CGATATCG)
  - [ ] Implement encode_message() with nanopore mode
  - [ ] Implement decode_sequence() with error correction
  - [ ] Implement get_nanopore_stats()
  - [ ] Write unit tests (17 test cases from Python)

- [ ] Port Secure DNA Crypto from Python to Rust (dna/secure.rs)
  - [ ] Implement AES-256-GCM encryption
  - [ ] Implement PBKDF2 key derivation (100,000 iterations)
  - [ ] Implement password strength validation
  - [ ] Implement secure encoding (AES + DNA)
  - [ ] Implement secure decoding (DNA + AES)
  - [ ] Implement password generation (cryptographically secure)
  - [ ] Write unit tests (15 test cases from Python)

### API Implementation
- [x] Create API data structures (models.rs)
  - [x] EncodeRequest struct
  - [x] EncodeResponse struct
  - [x] DecodeRequest struct
  - [x] DecodeResponse struct
  - [x] SafetyScreenRequest struct
  - [x] SafetyScreenResponse struct
  - [x] Error response types

- [x] Implement encode endpoint (POST /api/encode)
  - [x] Support basic mode
  - [ ] Support nanopore mode
  - [ ] Support secure mode with password
  - [x] Input validation and sanitization
  - [x] Error handling

- [x] Implement decode endpoint (POST /api/decode)
  - [x] Support basic mode
  - [ ] Support nanopore mode with error correction
  - [ ] Support secure mode with password
  - [x] Input validation
  - [x] Error handling

- [x] Implement safety screen endpoint (POST /api/safety-screen)
  - [x] Pathogen detection
  - [x] Natural occurrence check
  - [x] Sequence characteristics
  - [x] Risk assessment

### Testing
- [ ] Write integration tests for API endpoints
- [ ] Test error handling paths
- [ ] Performance benchmarking for crypto operations
- [ ] Validate output matches Python implementation exactly

### Documentation
- [ ] Document API endpoints with examples
- [ ] Create README for backend project
- [ ] Document crypto algorithms
- [ ] Set up Swagger/OpenAPI spec

---

## Phase 2: Solana Smart Programs (Week 2-3)
- [ ] Not started

---

## Phase 3: Backend-Solana Integration (Week 3-4)
- [ ] Not started

---

## Phase 4: Frontend Updates (Week 4-5)
- [ ] Not started

---

## Phase 5: Testing & Optimization (Week 5-6)
- [ ] Not started

---

## Phase 6: Deployment (Week 6-7)
- [ ] Not started

---

## Current Progress

**Completed**: Basic DNA crypto (mostly working), API structure, Safety screener

**In Progress**: Fixing Basic DNA crypto test failures

**Next Steps**:
1. Fix test failures in Basic DNA crypto
2. Implement Nanopore DNA crypto
3. Implement Secure DNA crypto
4. Update API to support all modes

---

## Issues Found

1. **Basic DNA Crypto Tests Failing** (5 tests):
   - test_encode_simple: Expecting "TAAA" but getting "TAAT"
   - test_decode_simple: Related to encode
   - test_invalid_bases_skipped: Not skipping invalid bases correctly
   - test_newline_characters: Not preserving newlines
   - test_clean_sequence: Not handling 'N' correctly

2. **Build Warnings**:
   - Unused imports in dna/mod.rs (cleanup needed)

---

**Last Updated**: 2025-01-20
