# BioCypher Rust + Solana Migration - Task List

> Historical document: web/Flask/API instructions below describe the retired architecture. See the [current terminal-only README](../README.md) for setup and usage.

## Overview
Migrate BioCypher from Python/Flask to Rust backend services with Solana blockchain integration using hybrid architecture.

## Project Status
- **Phase**: Planning Complete
- **Current Status**: Ready to begin Phase 1
- **Target Timeline**: 7 weeks
- **Architecture**: Hybrid (Rust backend + Solana programs)

---

## Phase 1: Rust Backend Foundation (Week 1-2)

### Project Setup
- [ ] Initialize Cargo workspace structure
- [ ] Create backend/Cargo.toml with all dependencies
- [ ] Set up project directory structure (src/dna, src/safety, src/solana, src/api)
- [ ] Configure logging with tracing
- [ ] Set up error handling with thiserror
- [ ] Create config management module
- [ ] Set up basic Actix-web server with health check endpoint

### DNA Crypto Implementation
- [ ] Port Basic DNA Crypto from Python to Rust (dna/basic.rs)
  - [ ] Implement text_to_binary()
  - [ ] Implement binary_to_dna()
  - [ ] Implement dna_to_binary()
  - [ ] Implement binary_to_text()
  - [ ] Implement encode_message()
  - [ ] Implement decode_sequence()
  - [ ] Implement get_sequence_stats()
  - [ ] Write unit tests (10 test cases from Python)

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
- [ ] Create API data structures (models.rs)
  - [ ] EncodeRequest struct
  - [ ] EncodeResponse struct
  - [ ] DecodeRequest struct
  - [ ] DecodeResponse struct
  - [ ] SafetyScreenRequest struct
  - [ ] SafetyScreenResponse struct
  - [ ] Error response types

- [ ] Implement encode endpoint (POST /api/encode)
  - [ ] Support basic mode
  - [ ] Support nanopore mode
  - [ ] Support secure mode with password
  - [ ] Input validation and sanitization
  - [ ] Error handling

- [ ] Implement decode endpoint (POST /api/decode)
  - [ ] Support basic mode
  - [ ] Support nanopore mode with error correction
  - [ ] Support secure mode with password
  - [ ] Input validation
  - [ ] Error handling

- [ ] Implement safety screen endpoint (POST /api/safety-screen)
  - [ ] Pathogen detection
  - [ ] Natural occurrence check
  - [ ] Sequence characteristics
  - [ ] Risk assessment

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

### Anchor Setup
- [ ] Initialize Anchor workspace
- [ ] Set up program structure (encoder, decoder, safety)
- [ ] Configure Anchor.toml for programs
- [ ] Set up local Solana validator for testing

### Encoder Program
- [ ] Create biocypher-encoder program structure
- [ ] Define DNARecord account structure
  - [ ] owner: Pubkey
  - [ ] mode: EncodingMode enum
  - [ ] message: String (max 1000 chars)
  - [ ] dna_sequence: String (max 10000 bases)
  - [ ] use_error_correction: bool
  - [ ] timestamp: i64
- [ ] Implement encode_basic instruction
  - [ ] Validate input message
  - [ ] Encode to DNA on-chain
  - [ ] Store in DNARecord account
  - [ ] PDA derivation from owner key
- [ ] Implement encode_nanopore instruction
  - [ ] Validate input
  - [ ] Triplet encoding on-chain
  - [ ] Add error correction
  - [ ] Add nanopore markers
  - [ ] Store in account
- [ ] Implement encode_secure instruction
  - [ ] Validate password
  - [ ] Encrypt message (AES-256)
  - [ ] Encode to DNA
  - [ ] Store encrypted data
- [ ] Implement error codes
- [ ] Write unit tests for all instructions

### Decoder Program
- [ ] Create biocypher-decoder program structure
- [ ] Define DecodeResult account structure
- [ ] Implement decode_basic instruction
  - [ ] Validate DNA sequence
  - [ ] Decode on-chain
  - [ ] Store result
- [ ] Implement decode_nanopore instruction
  - [ ] Remove markers
  - [ ] Triplet decoding
  - [ ] Error correction
  - [ ] Store result
- [ ] Implement decode_secure instruction
  - [ ] Validate DNA sequence
  - [ ] Decode DNA
  - [ ] Decrypt with password
  - [ ] Store result
- [ ] Write unit tests for all instructions

### Safety Screener Program
- [ ] Create biocypher-safety program structure
- [ ] Define SafetyReport account structure
  - [ ] dna_sequence: String
  - [ ] safety_status: SafetyStatus enum
  - [ ] pathogen_risk: bool
  - [ ] natural_occurrence: bool
  - [ ] gc_content: u8
  - [ ] homopolymer_count: u32
  - [ ] timestamp: i64
- [ ] Implement screen_sequence instruction
  - [ ] Pathogen detection logic
  - [ ] Natural occurrence check
  - [ ] GC content analysis
  - [ ] Homopolymer detection
  - [ ] Risk assessment
  - [ ] Generate status (Safe/Caution/Unsafe)
- [ ] Implement pathogen signature database (compressed)
- [ ] Write unit tests for all instructions

### Deployment
- [ ] Deploy encoder program to Solana devnet
- [ ] Deploy decoder program to Solana devnet
- [ ] Deploy safety program to Solana devnet
- [ ] Verify program deployments
- [ ] Create deployment script
- [ ] Document program IDs

### Testing
- [ ] Integration tests for all programs
- [ ] Test with local validator
- [ ] Test on devnet
- [ ] Performance testing (compute units)
- [ ] Account size optimization

---

## Phase 3: Backend-Solana Integration (Week 3-4)

### Solana Client Implementation
- [ ] Create SolanaClient wrapper (solana/client.rs)
  - [ ] Initialize RPC client with commitment
  - [ ] Load payer keypair from env
  - [ ] Implement transaction builder
  - [ ] Implement account management utilities
- [ ] Implement encoder interaction (solana/encoder.rs)
  - [ ] store_dna_basic() - Create and send transaction
  - [ ] store_dna_nanopore() - Nanopore encoding
  - [ ] store_dna_secure() - Encrypted storage
  - [ ] retrieve_dna() - Fetch DNA record from chain
- [ ] Implement decoder interaction (solana/decoder.rs)
  - [ ] decode_dna_basic() - On-chain decoding
  - [ ] decode_dna_nanopore() - Nanopore decoding
  - [ ] decode_dna_secure() - Decryption
- [ ] Implement safety interaction (solana/safety.rs)
  - [ ] screen_dna_on_chain() - On-chain screening
  - [ ] retrieve_safety_report() - Fetch report
  - [ ] verify_report() - Verify immutability

### API Updates
- [ ] Update encode endpoint to support on-chain storage
  - [ ] Add store_on_chain parameter
  - [ ] Conditionally call Solana client
  - [ ] Return transaction signature if stored
- [ ] Update decode endpoint to support on-chain decoding
  - [ ] Add decode_on_chain parameter
  - [ ] Call decoder program
  - [ ] Return on-chain result
- [ ] Update safety screen endpoint
  - [ ] Add verify_on_chain parameter
  - [ ] Call safety program
  - [ ] Return transaction signature
- [ ] Add verification endpoint (GET /api/verify/:signature)
  - [ ] Fetch transaction from Solana
  - [ ] Parse and return result
- [ ] Add transaction history endpoint (GET /api/history/:public_key)
  - [ ] Fetch user's DNA records
  - [ ] Fetch user's safety reports
  - [ ] Paginate results

### Caching Layer
- [ ] Set up Redis connection
- [ ] Implement cache for DNA encodings
  - [ ] Cache key: hash(message + mode)
  - [ ] TTL: 24 hours
- [ ] Implement cache for safety screenings
  - [ ] Cache key: hash(dna_sequence)
  - [ ] TTL: 7 days
- [ ] Implement cache invalidation
  - [ ] Invalidate on new data
  - [ ] Manual invalidation endpoint

### Configuration
- [ ] Add Solana RPC URL to config
- [ ] Add Solana program IDs to config
- [ ] Add payer keypair configuration
- [ ] Add Redis connection config
- [ ] Support devnet/mainnet switching

### Testing
- [ ] Integration tests for Solana interaction
- [ ] Test transaction building and sending
- [ ] Test error handling (network failures)
- [ ] Test caching behavior
- [ ] Test account retrieval

---

## Phase 4: Frontend Updates (Week 4-5)

### API Client
- [ ] Create TypeScript API client (src/api/biocypher.ts)
  - [ ] encodeMessage() - Basic off-chain encoding
  - [ ] encodeMessageOnChain() - On-chain storage
  - [ ] decodeMessage() - Basic off-chain decoding
  - [ ] decodeMessageOnChain() - On-chain decoding
  - [ ] safetyScreen() - Off-chain screening
  - [ ] safetyScreenOnChain() - On-chain verification
  - [ ] verifyTransaction() - Verify transaction on-chain
  - [ ] getTransactionHistory() - Fetch user's history

### Components
- [ ] Update EncodeForm component
  - [ ] Add mode selection (basic/nanopore/secure)
  - [ ] Add password input for secure mode
  - [ ] Add "Store on Blockchain" checkbox
  - [ ] Display DNA sequence result
  - [ ] Display transaction signature if on-chain
  - [ ] Add Solana explorer link
  - [ ] Display stats (GC content, length, etc.)
- [ ] Update DecodeForm component
  - [ ] Add mode selection
  - [ ] Add password input for secure mode
  - [ ] Add "Verify on Blockchain" option
  - [ ] Display decoded message
  - [ ] Display verification status
- [ ] Update SafetyScreen component
  - [ ] Add DNA sequence input
  - [ ] Add "Verify on Blockchain" option
  - [ ] Display safety status (Safe/Caution/Unsafe)
  - [ ] Display pathogen analysis
  - [ ] Display natural occurrence
  - [ ] Display recommendations
  - [ ] Add transaction verification
- [ ] Create TransactionViewer component
  - [ ] Display transaction details
  - [ ] Show Solana explorer link
  - [ ] Display timestamp
  - [ ] Display account addresses
- [ ] Create TransactionHistory component
  - [ ] List user's transactions
  - [ ] Filter by type (encode/decode/safety)
  - [ ] Paginate results
  - [ ] Click to view details

### UI/UX Improvements
- [ ] Add loading states for on-chain operations
- [ ] Add progress indicators
- [ ] Add error messages for Solana failures
- [ ] Add tooltips for blockchain features
- [ ] Add "What's this?" explanations
- [ ] Responsive design improvements

### Testing
- [ ] Component unit tests
- [ ] Integration tests with API
- [ ] E2E tests with Cypress/Playwright
- [ ] Browser compatibility testing

---

## Phase 5: Testing & Optimization (Week 5-6)

### Integration Testing
- [ ] End-to-end encode flow (off-chain)
- [ ] End-to-end encode flow (on-chain)
- [ ] End-to-end decode flow (off-chain)
- [ ] End-to-end decode flow (on-chain)
- [ ] End-to-end safety screening (off-chain)
- [ ] End-to-end safety screening (on-chain)
- [ ] Cross-mode encoding/decoding
- [ ] Error handling across all flows

### Performance Testing
- [ ] Load test Rust backend (1000 req/sec)
- [ ] Load test Solana interactions
- [ ] Measure cache hit rates
- [ ] Optimize slow endpoints
- [ ] Profile memory usage
- [ ] Optimize compute unit usage in Solana

### Security Audit
- [ ] Review Rust code for vulnerabilities
- [ ] Review Solana programs for security issues
- [ ] Test for common web vulnerabilities (OWASP)
- [ ] Validate crypto implementations
- [ ] Check for key leakage in logs
- [ ] Test input sanitization
- [ ] External security audit

### Documentation
- [ ] Complete API documentation
- [ ] Write architecture documentation
- [ ] Create deployment guide
- [ ] Write troubleshooting guide
- [ ] Document Solana program instructions
- [ ] Create video tutorials
- [ ] Write developer onboarding guide

---

## Phase 6: Deployment (Week 6-7)

### Rust Backend Deployment
- [ ] Containerize with Docker
- [ ] Set up CI/CD pipeline
- [ ] Deploy to staging environment
- [ ] Configure load balancer
- [ ] Set up monitoring (Prometheus/Grafana)
- [ ] Set up log aggregation (ELK/Loki)
- [ ] Configure TLS certificates
- [ ] Set up health checks
- [ ] Deploy to production
- [ ] Configure auto-scaling

### Solana Programs Deployment
- [ ] Finalize program IDs
- [ ] Deploy programs to mainnet
- [ ] Verify program source
- [ ] Set up program monitoring
- [ ] Create upgrade authority setup
- [ ] Document program addresses
- [ ] Set up alerts for program activity

### Frontend Deployment
- [ ] Build production bundle
- [ ] Deploy to CDN (Cloudflare/Vercel)
- [ ] Configure custom domain
- [ ] Set up SSL
- [ ] Configure caching headers
- [ ] Set up analytics
- [ ] Deploy to production

### Monitoring & Alerting
- [ ] Set up application monitoring
- [ ] Set up blockchain monitoring
- [ ] Set up error tracking (Sentry)
- [ ] Configure alert thresholds
- [ ] Create runbooks for incidents
- [ ] Set up status page
- [ ] Configure backup systems

### Data Migration (if needed)
- [ ] Export existing Python data
- [ ] Transform data for Rust backend
- [ ] Batch encode with Rust
- [ ] Store key records on Solana
- [ ] Verify data integrity
- [ ] Cutover traffic gradually

### Launch
- [ ] Pre-launch checklist
- [ ] Final smoke tests
- [ ] Monitor launch metrics
- [ ] Handle support requests
- [ ] Post-launch review

---

## Ongoing Tasks

### Maintenance
- [ ] Regular security updates
- [ ] Dependency updates
- [ ] Performance monitoring
- [ ] User feedback collection
- [ ] Bug fixes

### Enhancements
- [ ] Add wallet adapter integration (optional)
- [ ] Add batch encoding API
- [ ] Add advanced nanopore features
- [ ] Add export/import functionality
- [ ] Add collaboration features
- [ ] Add mobile app (future)

---

## Notes

### Completed
- [x] Architecture analysis complete
- [x] Technology stack selected
- [x] Detailed implementation plan created
- [x] Task breakdown complete

### In Progress
- [ ] Phase 1: Rust Backend Foundation

### Blocked
- None currently

### Risks
- Solana network congestion may affect on-chain operations
- Mitigation: Hybrid architecture allows off-chain fallback
