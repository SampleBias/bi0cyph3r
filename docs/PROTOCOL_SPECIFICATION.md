# Bi0cyph3r protocol

This document specifies the formats implemented by the terminal application. The current authenticated envelopes are version 2. The historical Nanopore and CBC formats are retained for compatibility. The protocol is open; implementations should use the vectors below and reject malformed data.

## Common conventions

Text is UTF-8. Bytes map to four DNA bases, most significant two bits first: `00=A`, `01=T`, `10=C`, `11=G`. For example, `A` (hex `41`) becomes `TAAT`, and `Hi` (hex `48 69`) becomes `TACATCCT`.

The application accepts raw DNA or one FASTA record, removes whitespace and normalizes case before decoding. Invalid bases and multiple FASTA records are rejected. Mode selection is explicit: sequences do not necessarily identify the historical mode. The TUI limits text to 4 KiB and imported input to 256 KiB.

## Basic

Encode UTF-8 bytes with the mapping above, without a header, encryption, or checksum. Decoding requires a multiple of four bases and valid UTF-8. Basic is an encoding, not encryption or integrity protection.

## Authenticated envelope version 2

Secure and Split Key begin with the literal DNA prefix `GCGTACGCGTACGCGT`. All following bytes are mapped directly to DNA using the common mapping. The binary header and ciphertext are:

| Mode | Header bytes | Remaining bytes |
| --- | --- | --- |
| Secure | `01` (one byte), 16-byte salt, 12-byte nonce | AES-256-GCM ciphertext followed by 16-byte tag |
| Split Key | `02` (one byte), 12-byte nonce | AES-256-GCM ciphertext followed by 16-byte tag |

AES-GCM additional authenticated data is the ASCII bytes of the literal DNA prefix followed by the complete binary header. Plaintext is the UTF-8 message. Nonces and salts are generated using the operating system random source. Decoders reject an incorrect prefix, mode, length, tag, credentials, or UTF-8 plaintext. Never reuse a nonce with the same key.

Secure derives a 32-byte key using Argon2id version 0x13 with memory cost 19,456 KiB, two iterations, one lane, and the 16-byte header salt. The password is encoded in UTF-8. These parameters are fixed for this envelope version.

Split Key creates an independent random 32-byte AES key and a random 32-byte share K1; K2 is key XOR K1. The two shares are exported as standard padded Base64 strings. Both are needed to recover the key. The TUI requires explicit key export; the key bundle contains both shares, so store them separately afterward.

### Version 2 test vectors

These deterministic vectors use plaintext `Hi` (`48 69`) and nonce `000102030405060708090a0b` in hexadecimal. Fixed values are for conformance tests only; real encoders must generate fresh random values. The vectors were generated independently using Python's `cryptography` implementation.

Secure: password `password`; salt `000102030405060708090a0b0c0d0e0f`:

```text
GCGTACGCGTACGCGTAAATAAAAAAATAAACAAAGAATAAATTAATCAATGAACAAACTAACCAACGAAGAAAGTAAGCAAGGAAAAAAATAAACAAAGAATAAATTAATCAATGAACAAACTAACCAACGCAGGCAAGCTTCTATACAAGCAGTGGGGTGCGAAAGAGGATGATTACTTACTTTCCTAGGAACAAGCCGGAG
```

Split Key: K1 is 32 zero bytes; K2 is `000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f`:

```text
GCGTACGCGTACGCGTAAACAAAAAAATAAACAAAGAATAAATTAATCAATGAACAAACTAACCAACGAAGGTCCGGACAGTGAGCCGGTCCTCTGCAACTTACTGGTAGCTTATAAGTTCCACAGCCTATCTGTAACAG
```

## Historical Nanopore format

Each UTF-8 byte is expanded to eight bits followed by even parity, then each bit is repeated three times. Every three bits map to a DNA triplet: `000=ATC`, `001=ATG`, `010=ACT`, `011=ACG`, `100=TAG`, `101=TAC`, `110=TCG`, `111=TCA`. The core is therefore 27 bases per input byte. The encoder adds `ATCGATCG` at the start and `CGATATCG` at the end.

If the core has consecutive equal bases or its GC content is outside 40–60%, the encoder surrounds it with the delimiter `TACGTA` and matching padding on both sides. Padding repeats `ATCATGACTACG` to `max(6, floor(core length / 10))` bases per side. This padding is heuristic and does not guarantee GC balance or the absence of homopolymers. The legacy decoder identifies padding using the first and last delimiter; a core that itself contains delimiters may be ambiguous. Corrupted or truncated data can be rejected; repetition and parity do not guarantee recovery from sequencing errors, particularly insertions and deletions.

## Legacy encrypted formats

Legacy Secure and Legacy Split use the same sequence layout: the prefix `ATCGATCG`, followed by Basic encoding of a standard padded Base64 string, followed by `CGATATCG`. The Base64 data contains, in order, a big-endian 16-bit salt length (16), 16-byte salt, a big-endian 16-bit IV length (16), 16-byte IV, a big-endian 32-bit ciphertext length, and AES-256-CBC ciphertext with PKCS#7 padding. Secure derives its key with PBKDF2-HMAC-SHA256, 100,000 iterations; Split reconstructs its key by XOR of two 32-byte shares and ignores the salt.

These formats do not authenticate ciphertext, IV, or metadata. They remain available as explicit decode-only modes; a correct-looking result is not proof of integrity. The original historical design notes and attribution are in the project history and [LEGACY_LICENSE](LEGACY_LICENSE).
