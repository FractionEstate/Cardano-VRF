# Changelog

All notable changes to the `cardano-vrf` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-11-09

### Added
- Initial release of `cardano-vrf` crate
- Draft-03 VRF implementation (ECVRF-ED25519-SHA512-Elligator2) with 80-byte proofs
- Draft-13 VRF implementation (ECVRF-ED25519-SHA512-TAI) with 128-byte proofs (batch-compatible)
- Cardano-compatible VRF primitives in `cardano_compat` module
  - `prove.rs`: VRF proof generation
  - `verify.rs`: VRF proof verification
  - `point.rs`: Edwards point operations and cofactor clearing
- Pure Rust implementation with no FFI dependencies
- Memory-safe operations using `zeroize` for sensitive data
- Common utilities for both draft versions
- Feature-gated debug logging (`vrf-debug` feature)
- Comprehensive examples:
  - `basic_usage.rs`: Basic VRF prove and verify workflow
  - `keypair_generation.rs`: Different methods for generating keypairs
- No-std support (with `alloc`)
- MIT/Apache-2.0 dual licensing

### Features
- Byte-for-byte compatibility with Cardano's libsodium VRF implementation
- Constant-time cryptographic operations
- Support for both legacy (Draft-03) and modern (Draft-13) VRF variants
- Deterministic keypair generation from seeds
- Simple and ergonomic API

[0.1.0]: https://github.com/FractionEstate/Cardano-VRF/releases/tag/v0.1.0
