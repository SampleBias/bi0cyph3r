# BioCypher Protocol Specification v1.0

> Historical design reference, not the current implementation contract. Some pseudocode, examples, and validation rules predate the Rust port. See the [current README](../README.md) and Rust tests for supported behavior, including UTF-8, strict input validation, and Split Key mode. Original attribution is preserved in [LEGACY_LICENSE](LEGACY_LICENSE).

**Document Status**: Formal Protocol Specification  
**Version**: 1.0  
**Date**: 2025-01-XX  
**Author**: BioCypher Project

---

## Table of Contents

1. [Overview](#overview)
2. [Protocol Architecture](#protocol-architecture)
3. [Encoding Modes](#encoding-modes)
4. [Binary-to-DNA Mapping](#binary-to-dna-mapping)
5. [Basic Mode Specification](#basic-mode-specification)
6. [Nanopore Mode Specification](#nanopore-mode-specification)
7. [Secure Mode Specification](#secure-mode-specification)
8. [Sequence Format](#sequence-format)
9. [Error Handling](#error-handling)
10. [Validation Rules](#validation-rules)
11. [Test Vectors](#test-vectors)
12. [Conformance Requirements](#conformance-requirements)

---

## Overview

The BioCypher Protocol is a DNA-based encoding system for storing and transmitting digital data. It supports three encoding modes optimized for different use cases: basic encoding, nanopore-optimized encoding, and secure encrypted encoding.

### Protocol Goals

- **Interoperability**: Any compliant implementation can encode/decode BioCypher sequences
- **Reliability**: Error correction and validation ensure data integrity
- **Security**: Optional encryption for sensitive data
- **Platform Optimization**: Nanopore mode optimized for sequencing constraints

### Protocol Version

This specification defines **BioCypher Protocol Version 1.0**.

---

## Protocol Architecture

```
┌─────────────────────────────────────────┐
│         Input: Text Message              │
└─────────────────┬───────────────────────┘
                  │
        ┌─────────┴─────────┐
        │  Select Mode      │
        └─────────┬─────────┘
                  │
    ┌─────────────┼─────────────┐
    │             │             │
┌───▼───┐   ┌────▼────┐   ┌───▼────┐
│ Basic │   │Nanopore │   │ Secure │
└───┬───┘   └────┬────┘   └───┬────┘
    │            │            │
    └────────────┼────────────┘
                 │
        ┌────────▼────────┐
        │  DNA Sequence   │
        └─────────────────┘
```

---

## Encoding Modes

BioCypher supports three encoding modes:

| Mode ID | Name | Description | Markers | Error Correction |
|---------|------|-------------|---------|------------------|
| `basic` | Basic Mode | Simple binary-to-DNA mapping | None | None |
| `nanopore` | Nanopore Mode | Optimized for nanopore sequencing | Yes | Yes |
| `secure` | Secure Mode | AES-256-CBC encrypted + DNA encoding | Yes | None |

---

## Binary-to-DNA Mapping

### Standard Mapping Table

The fundamental mapping used in Basic and Secure modes:

| Binary Pair | DNA Base | Description |
|-------------|----------|-------------|
| `00` | `A` | Adenine |
| `01` | `T` | Thymine |
| `10` | `C` | Cytosine |
| `11` | `G` | Guanine |

### Encoding Rules

1. Binary string is processed in pairs (2 bits → 1 base)
2. If binary length is odd, pad with `0` before final pair
3. Invalid DNA bases (not A, T, C, G) are skipped during decoding

### Decoding Rules

1. Each DNA base maps to 2-bit binary pair
2. Binary string is processed in 8-bit chunks (bytes)
3. Each byte represents one ASCII character

---

## Basic Mode Specification

### Overview

Basic mode provides the simplest encoding: direct binary-to-DNA conversion with no error correction or markers.

### Encoding Algorithm

```
Input: Text message M
Output: DNA sequence D

1. Convert M to binary B:
   For each character c in M:
     ascii_val = ord(c)
     binary_byte = format(ascii_val, '08b')
     B += binary_byte

2. Convert B to DNA D:
   For each pair (b1, b2) in B:
     dna_base = DNA_ENCODE[b1 + b2]
     D += dna_base

3. Return D
```

### Decoding Algorithm

```
Input: DNA sequence D
Output: Text message M

1. Convert D to binary B:
   For each base b in D:
     if b is valid (A, T, C, G):
       binary_pair = DNA_DECODE[b]
       B += binary_pair

2. Convert B to text M:
   For each 8-bit chunk in B:
     ascii_val = int(chunk, 2)
     if 32 <= ascii_val <= 126:  # Printable ASCII
       M += chr(ascii_val)

3. Return M
```

### Sequence Format

- **No markers**: Basic mode sequences have no start/stop markers
- **No padding**: Sequences are raw DNA without delimiters
- **Length**: Variable, depends on input message length

### Example

```
Input:  "Hi"
Binary: 01001000 01101001
DNA:    TAAA TATA TAAA TATA
        (01=T, 00=A, 10=C, 00=A, 01=T, 01=T, 00=A, 10=C, 00=A, 01=T, 00=A, 10=C, 00=A, 01=T, 00=A, 10=C)
```

---

## Nanopore Mode Specification

### Overview

Nanopore mode optimizes sequences for nanopore sequencing by:
- Avoiding homopolymer runs
- Balancing GC content
- Adding error correction
- Using triplet encoding

### Encoding Algorithm

```
Input: Text message M, error_correction flag E
Output: DNA sequence D

1. Convert M to binary with parity B:
   For each character c in M:
     ascii_val = ord(c)
     binary_byte = format(ascii_val, '08b')
     parity = (binary_byte.count('1') % 2)
     binary_byte += str(parity)  # 9 bits total
     B += binary_byte

2. Apply error correction if E is true:
   For each bit b in B:
     B_corrected += b + b + b  # Triple redundancy

3. Convert to DNA using triplet encoding:
   Pad B to multiple of 3
   For each triplet (b1, b2, b3) in B:
     dna_triplet = NANOPORE_ENCODE[b1 + b2 + b3]
     D += dna_triplet

4. Check for optimization needs:
   if has_homopolymers(D) or not is_gc_balanced(D):
     padding = generate_padding(D)
     delimiter = "TACGTA"
     D = padding + delimiter + D + delimiter + padding

5. Add markers:
   D = "ATCGATCG" + D + "CGATATCG"

6. Return D
```

### Triplet Encoding Table

| Binary Triplet | DNA Triplet | Notes |
|----------------|------------|-------|
| `000` | `ATC` | No homopolymers |
| `001` | `ATG` | No homopolymers |
| `010` | `ACT` | No homopolymers |
| `011` | `ACG` | No homopolymers |
| `100` | `TAG` | No homopolymers |
| `101` | `TAC` | No homopolymers |
| `110` | `TCG` | No homopolymers |
| `111` | `TCA` | No homopolymers |

### Decoding Algorithm

```
Input: DNA sequence D, error_correction flag E
Output: Text message M

1. Remove markers:
   if D starts with "ATCGATCG":
     D = D[8:]
   if D ends with "CGATATCG":
     D = D[:-8]

2. Remove padding if present:
   delimiter = "TACGTA"
   first_delim = D.find(delimiter)
   last_delim = D.rfind(delimiter)
   if first_delim != -1 and last_delim != -1 and first_delim != last_delim:
     D = D[first_delim + 6 : last_delim]

3. Convert DNA to binary:
   For each triplet in D:
     binary_triplet = NANOPORE_DECODE[triplet]
     B += binary_triplet

4. Apply error correction if E is true:
   For each group of 3 bits in B:
     majority = most_common_bit(group)
     B_corrected += majority

5. Convert binary to text with parity check:
   For each 9-bit chunk in B:
     data_bits = chunk[0:8]
     parity_bit = chunk[8]
     if parity_bit == (data_bits.count('1') % 2):
       ascii_val = int(data_bits, 2)
       if 0 <= ascii_val <= 255:
         M += chr(ascii_val)

6. Return M
```

### Sequence Format

- **Start Marker**: `ATCGATCG` (8 bases)
- **Stop Marker**: `CGATATCG` (8 bases)
- **Padding Delimiter**: `TACGTA` (6 bases, if padding present)
- **Core Sequence**: Triplet-encoded data

### Padding Generation

If padding is needed:
```
padding_pattern = "ATCATGACTACG"
padding_length = max(6, sequence_length / 10)
padding = repeat(padding_pattern, padding_length)
```

### GC Content Optimization

- **Optimal Range**: 40-60% GC content
- **If outside range**: Padding is added to balance GC content
- **Validation**: GC content must be between 20-80% (wider tolerance)

### Homopolymer Avoidance

Banned patterns (detected via regex):
- `AA+` (2+ consecutive A)
- `TT+` (2+ consecutive T)
- `CC+` (2+ consecutive C)
- `GG+` (2+ consecutive G)
- `ATATAT` (repetitive alternating)
- `GCGCGC` (repetitive GC)

---

## Secure Mode Specification

### Overview

Secure mode combines AES-256-CBC encryption with DNA encoding for secure data storage.

### Cryptographic Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Encryption Algorithm | AES-256-CBC | Advanced Encryption Standard |
| Key Size | 256 bits (32 bytes) | 256-bit keys |
| Block Size | 128 bits (16 bytes) | AES block size |
| IV Size | 128 bits (16 bytes) | Initialization Vector |
| Salt Size | 128 bits (16 bytes) | PBKDF2 salt |
| Key Derivation | PBKDF2-HMAC-SHA256 | Password-based key derivation |
| PBKDF2 Iterations | 100,000 | Key stretching iterations |
| Padding Scheme | PKCS7 | Block padding |

### Encoding Algorithm

```
Input: Text message M, password P
Output: DNA sequence D

1. Encrypt message:
   salt = generate_random_bytes(16)
   iv = generate_random_bytes(16)
   key = PBKDF2(P, salt, iterations=100000, length=32)
   encrypted = AES256_CBC_encrypt(M, key, iv)

2. Serialize crypto data:
   crypto_data = length(salt) + salt + length(iv) + iv + length(encrypted) + encrypted
   crypto_string = base64_encode(crypto_data)

3. Encode to DNA using Basic Mode:
   D = BasicMode.encode(crypto_string)

4. Add markers:
   D = "ATCGATCG" + D + "CGATATCG"

5. Return D
```

### Crypto Data Serialization Format

```
Offset | Length | Field
-------|--------|------
0-1    | 2      | Salt length (big-endian uint16)
2-17   | 16     | Salt bytes
18-19  | 2      | IV length (big-endian uint16)
20-35  | 16     | IV bytes
36-39  | 4      | Encrypted data length (big-endian uint32)
40+    | N      | Encrypted data bytes
```

### Decoding Algorithm

```
Input: DNA sequence D, password P
Output: Text message M

1. Remove markers:
   D = D[8:-8]  # Remove "ATCGATCG" and "CGATATCG"

2. Decode DNA using Basic Mode:
   crypto_string = BasicMode.decode(D)

3. Deserialize crypto data:
   crypto_data = base64_decode(crypto_string)
   salt = extract_salt(crypto_data)
   iv = extract_iv(crypto_data)
   encrypted = extract_encrypted_data(crypto_data)

4. Decrypt:
   key = PBKDF2(P, salt, iterations=100000, length=32)
   M = AES256_CBC_decrypt(encrypted, key, iv)

5. Return M
```

### Sequence Format

- **Start Marker**: `ATCGATCG` (8 bases)
- **Stop Marker**: `CGATATCG` (8 bases)
- **Core Sequence**: Base64-encoded crypto data in DNA form

### Password Requirements

Minimum password strength:
- Length: ≥ 8 characters (recommended ≥ 12)
- Character variety: uppercase, lowercase, digits, special characters
- Validation: Implementations should validate password strength

---

## Sequence Format

### Basic Mode Format

```
[Payload DNA Sequence]
```

- No markers
- No delimiters
- Raw DNA sequence

### Nanopore Mode Format

```
ATCGATCG [Optional Padding] TACGTA [Core Sequence] TACGTA [Optional Padding] CGATATCG
```

Where:
- `ATCGATCG` = Start marker (always present)
- `TACGTA` = Padding delimiter (only if padding present)
- `CGATATCG` = Stop marker (always present)

### Secure Mode Format

```
ATCGATCG [Core Sequence] CGATATCG
```

Where:
- `ATCGATCG` = Start marker (always present)
- `CGATATCG` = Stop marker (always present)

---

## Error Handling

### Error Codes

| Error Code | Name | Description |
|------------|------|-------------|
| `INVALID_SEQUENCE` | Invalid DNA sequence | Sequence contains invalid bases |
| `MISSING_MARKERS` | Missing markers | Required markers not found |
| `DECODE_FAILED` | Decoding failed | Unable to decode sequence |
| `ENCRYPTION_ERROR` | Encryption error | Cryptographic operation failed |
| `PASSWORD_REQUIRED` | Password required | Secure mode requires password |
| `PASSWORD_WEAK` | Weak password | Password doesn't meet strength requirements |
| `CHECKSUM_MISMATCH` | Checksum mismatch | Data integrity check failed |

### Error Handling Rules

1. **Invalid DNA Bases**: Skip invalid bases during decoding, log warning
2. **Missing Markers**: Return error if markers are required but missing
3. **Decoding Failures**: Return empty string or raise exception (implementation-dependent)
4. **Cryptographic Errors**: Never expose sensitive information in error messages

---

## Validation Rules

### Sequence Validation

1. **DNA Base Validation**:
   - Only A, T, C, G are valid
   - Case-insensitive (convert to uppercase)
   - Whitespace should be removed before validation

2. **Marker Validation** (Nanopore/Secure modes):
   - Must start with `ATCGATCG` if markers required
   - Must end with `CGATATCG` if markers required
   - Markers must be exact (no variations)

3. **Length Validation**:
   - Minimum sequence length: 0 (empty sequences allowed)
   - Maximum sequence length: Implementation-dependent (typically 10,000+ bases)

### Nanopore-Specific Validation

1. **GC Content**: Must be between 20-80% (wider tolerance than optimal 40-60%)
2. **Homopolymer Check**: Should not contain banned patterns (warning, not error)
3. **Triplet Alignment**: Sequence length should be multiple of 3 (after marker removal)

### Secure Mode Validation

1. **Password Strength**: Must meet minimum requirements
2. **Crypto Data Format**: Must be valid base64 after DNA decoding
3. **Serialization Format**: Must match expected structure

---

## Test Vectors

### Basic Mode Test Vector 1

```
Input:  "A"
Binary: 01000001
DNA:    TAAA
        (01=T, 00=A, 00=A, 00=A)
```

### Basic Mode Test Vector 2

```
Input:  "Hi"
Binary: 01001000 01101001
DNA:    TAAA TATA TAAA TATA
```

### Nanopore Mode Test Vector 1

```
Input:  "Hello"
Mode:   nanopore
Error Correction: true
Output: ATCGATCG [triplet-encoded data with error correction] CGATATCG
```

### Secure Mode Test Vector 1

```
Input:  "Secret"
Password: "TestPass123!"
Mode:   secure
Output: ATCGATCG [encrypted base64 in DNA] CGATATCG
```

**Note**: Exact output depends on random salt/IV generation and is non-deterministic.

---

## Conformance Requirements

### Level 1: Basic Conformance

A conforming implementation MUST:
- Support Basic Mode encoding/decoding
- Implement standard binary-to-DNA mapping (00→A, 01→T, 10→C, 11→G)
- Handle ASCII text input (32-126 range)
- Skip invalid DNA bases during decoding

### Level 2: Nanopore Conformance

A conforming implementation MUST:
- Meet Level 1 requirements
- Support Nanopore Mode encoding/decoding
- Implement triplet encoding table
- Support error correction (triple redundancy)
- Handle nanopore markers (ATCGATCG/CGATATCG)
- Support padding removal

### Level 3: Secure Conformance

A conforming implementation MUST:
- Meet Level 1 requirements
- Support Secure Mode encoding/decoding
- Implement AES-256-CBC encryption
- Implement PBKDF2 key derivation (100,000 iterations)
- Support password validation
- Handle secure mode markers

### Level 4: Full Conformance

A conforming implementation MUST:
- Meet all Level 1, 2, and 3 requirements
- Support all three encoding modes
- Implement comprehensive error handling
- Support sequence validation
- Provide statistics and analysis features

---

## Implementation Notes

### Character Encoding

- **Input**: UTF-8 text (encoded as bytes, then ASCII)
- **Output**: ASCII characters (32-126 range)
- **Unicode**: Not directly supported (must be encoded first)

### Performance Considerations

- **Basic Mode**: Fastest, minimal overhead
- **Nanopore Mode**: Slower due to error correction and optimization
- **Secure Mode**: Slowest due to cryptographic operations

### Security Considerations

- **Secure Mode**: Uses industry-standard cryptography
- **Password Storage**: Never store passwords, derive keys on-the-fly
- **Error Messages**: Don't leak sensitive information
- **Random Number Generation**: Use cryptographically secure RNG

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-01-XX | Initial protocol specification |

---

## References

- AES Specification: FIPS 197
- PBKDF2 Specification: RFC 2898
- Base64 Encoding: RFC 4648

---

## License

This protocol specification is part of the BioCypher project. See LICENSE file for details.

---

**End of Specification**
