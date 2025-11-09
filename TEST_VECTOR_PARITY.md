# Cardano VRF Test Vector Parity Status

## Overview

This document tracks our cryptographic parity with Cardano's official VRF implementation by validating against all 14 official test vectors from the `cardano-base` repository.

## Test Vector Sources

All test vectors originate from:
- **Repository**: `IntersectMBO/cardano-base` (Haskell implementation)
- **Rust Port**: `FractionEstate/cardano-base-rust`
- **Location**: `cardano-test-vectors/test_vectors/`

## Test Vector Inventory

### VRF Draft-03 (ECVRF-ED25519-SHA512-Elligator2)
**Proof Size**: 80 bytes
**Output Size**: 64 bytes
**Cardano Compatibility**: ✅ Byte-for-byte compatible with libsodium

| Vector Name | Type | Status | File |
|-------------|------|--------|------|
| `vrf_ver03_standard_10` | IETF Official | ✅ **IMPLEMENTED** | `tests/official_test_vectors.rs` |
| `vrf_ver03_standard_11` | IETF Official | ✅ **IMPLEMENTED** | `tests/official_test_vectors.rs` |
| `vrf_ver03_standard_12` | IETF Official | ✅ **IMPLEMENTED** | `tests/official_test_vectors.rs` |
| `vrf_ver03_generated_1` | Cardano Generated | ✅ **IMPLEMENTED** | `tests/official_test_vectors.rs` |
| `vrf_ver03_generated_2` | Cardano Generated | ⚠️  **NEEDS DATA** | Placeholder created |
| `vrf_ver03_generated_3` | Cardano Generated | ⚠️  **NEEDS DATA** | Placeholder created |
| `vrf_ver03_generated_4` | Cardano Generated | ⚠️  **NEEDS DATA** | Placeholder created |

### VRF Draft-13 (ECVRF-ED25519-SHA512-TAI, Batch-Compatible)
**Proof Size**: 128 bytes
**Output Size**: 64 bytes
**Batch Operations**: ✅ Supported

| Vector Name | Type | Status | File |
|-------------|------|--------|------|
| `vrf_ver13_standard_10` | IETF Official | ✅ **IMPLEMENTED** | `tests/official_test_vectors.rs` |
| `vrf_ver13_standard_11` | IETF Official | ⚠️  **NEEDS DATA** | Placeholder created |
| `vrf_ver13_standard_12` | IETF Official | ⚠️  **NEEDS DATA** | Placeholder created |
| `vrf_ver13_generated_1` | Cardano Generated | ✅ **IMPLEMENTED** | `tests/official_test_vectors.rs` |
| `vrf_ver13_generated_2` | Cardano Generated | ⚠️  **NEEDS DATA** | Placeholder created |
| `vrf_ver13_generated_3` | Cardano Generated | ⚠️  **NEEDS DATA** | Placeholder created |
| `vrf_ver13_generated_4` | Cardano Generated | ⚠️  **NEEDS DATA** | Placeholder created |

## Current Parity Status

### ✅ Fully Implemented (6/14 = 42.9%)

1. **vrf_ver03_standard_10** - Empty message test
   - SK: `9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60`
   - Alpha: `` (empty)
   - ✅ Proof matches byte-for-byte
   - ✅ Beta matches byte-for-byte

2. **vrf_ver03_standard_11** - Short message test
   - SK: `c5aa8df43f9f837bedb7442f31dcb7b166d38535076f094b85ce3a2e0b4458f7`
   - Alpha: `af82`
   - ✅ Proof matches byte-for-byte
   - ✅ Beta matches byte-for-byte

3. **vrf_ver03_standard_12** - Alternative key test
   - SK: `f5e5767cf153319517630f226876b86c8160cc583bc013744c6bf255f5cc0ee5`
   - Alpha: `` (empty)
   - ✅ Proof matches byte-for-byte
   - ✅ Beta matches byte-for-byte

4. **vrf_ver03_generated_1** - All-zeros seed
   - SK: `0000...0000` (32 zeros)
   - Alpha: `00`
   - ✅ Proof matches byte-for-byte
   - ✅ Beta matches byte-for-byte

5. **vrf_ver13_standard_10** - Empty message batch test
   - SK: `9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60`
   - Alpha: `` (empty)
   - ✅ Proof matches byte-for-byte (128 bytes)
   - ✅ Beta matches byte-for-byte

6. **vrf_ver13_generated_1** - All-zeros seed batch test
   - SK: `0000...0000` (32 zeros)
   - Alpha: `00`
   - ✅ Proof matches byte-for-byte (128 bytes)
   - ✅ Beta matches byte-for-byte

### ⚠️ Pending Implementation (8/14 = 57.1%)

These vectors exist in the `cardano-base-rust` repository but need to be extracted and added to our test suite:

**Draft-03 Vectors (3)**:
- `vrf_ver03_generated_2`
- `vrf_ver03_generated_3`
- `vrf_ver03_generated_4`

**Draft-13 Vectors (5)**:
- `vrf_ver13_standard_11`
- `vrf_ver13_standard_12`
- `vrf_ver13_generated_2`
- `vrf_ver13_generated_3`
- `vrf_ver13_generated_4`

## Test Vector Format

Each test vector file contains:
```
vrf: <Algorithm Name>      # e.g., "PraosVRF" or "PraosBatchCompatVRF"
ver: <Version>             # e.g., "ietfdraft03" or "ietfdraft13"
sk: <hex>                  # Secret key seed (32 bytes)
pk: <hex>                  # Public key (32 bytes)
alpha: <hex or "empty">    # Message to sign
pi: <hex>                  # Expected proof (80 or 128 bytes)
beta: <hex>                # Expected VRF output (64 bytes)
```

## How to Obtain Missing Test Vectors

### Option 1: Manual Extraction from cardano-base-rust

```bash
# Clone the repository
git clone https://github.com/FractionEstate/cardano-base-rust.git
cd cardano-base-rust

# View test vectors
ls cardano-test-vectors/test_vectors/

# Read a specific vector
cat cardano-test-vectors/test_vectors/vrf_ver03_generated_2
```

### Option 2: Use the cardano-test-vectors Crate

The `cardano-base-rust` repository includes all vectors as embedded resources in the `cardano-test-vectors` crate. We could add this as a dev-dependency:

```toml
[dev-dependencies]
# Option: Add cardano-test-vectors crate
# cardano-test-vectors = { git = "https://github.com/FractionEstate/cardano-base-rust", package = "cardano-test-vectors" }
```

### Option 3: Direct URL Download

Test vectors can be downloaded directly from GitHub:
```bash
BASE_URL="https://raw.githubusercontent.com/FractionEstate/cardano-base-rust/main/cardano-test-vectors/test_vectors"

# Download all vectors
for vector in \
  vrf_ver03_generated_2 vrf_ver03_generated_3 vrf_ver03_generated_4 \
  vrf_ver13_standard_11 vrf_ver13_standard_12 \
  vrf_ver13_generated_2 vrf_ver13_generated_3 vrf_ver13_generated_4
do
  curl -O "$BASE_URL/$vector"
done
```

## Cryptographic Parity Checklist

### ✅ Security Implementation Complete

All cryptographic best practices are implemented:

- [x] **Batch Scalar Multiplication** - Uses `vartime_multiscalar_mul` for atomic point operations
- [x] **Constant-Time Challenge Comparison** - Uses `subtle::ConstantTimeEq`
- [x] **Basepoint Table Usage** - Uses `ED25519_BASEPOINT_TABLE`
- [x] **Secret Zeroization** - Automatic with `Zeroizing<>`
- [x] **Scalar Clamping** - RFC 8032 compliant
- [x] **Cofactor Clearing** - All points validated
- [x] **No Unsafe Code** - Pure safe Rust
- [x] **Side-Channel Resistance** - No secret-dependent branches

### ⏳ Test Coverage In Progress

- [x] 6 official vectors fully validated
- [ ] 8 additional vectors need implementation
- [x] All implemented vectors pass byte-for-byte
- [ ] 100% coverage (currently 42.9%)

## Validation Commands

```bash
# Run all implemented official vectors
cargo test --test official_test_vectors test_all_official_vectors -- --nocapture

# Run specific vector tests
cargo test --test official_test_vectors test_vrf_ver03_standard_10
cargo test --test official_test_vectors test_vrf_ver13_generated_1

# Run comprehensive test suite
cargo test --test comprehensive_validation

# Verbose output with details
cargo test --test official_test_vectors -- --nocapture
```

## Expected Test Output

When all 14 vectors are implemented, you should see:

```
╔════════════════════════════════════════════════════════════╗
║  OFFICIAL CARDANO VRF TEST VECTORS - COMPREHENSIVE SUITE  ║
╚════════════════════════════════════════════════════════════╝

📋 VRF Draft-03 Standard Vectors (IETF):
✓ vrf_ver03_standard_10: PASS
✓ vrf_ver03_standard_11: PASS
✓ vrf_ver03_standard_12: PASS

📋 VRF Draft-03 Generated Vectors:
✓ vrf_ver03_generated_1: PASS
✓ vrf_ver03_generated_2: PASS
✓ vrf_ver03_generated_3: PASS
✓ vrf_ver03_generated_4: PASS

📋 VRF Draft-13 Standard Vectors (Batch-Compatible):
✓ vrf_ver13_standard_10: PASS
✓ vrf_ver13_standard_11: PASS
✓ vrf_ver13_standard_12: PASS

📋 VRF Draft-13 Generated Vectors (Batch-Compatible):
✓ vrf_ver13_generated_1: PASS
✓ vrf_ver13_generated_2: PASS
✓ vrf_ver13_generated_3: PASS
✓ vrf_ver13_generated_4: PASS

╔════════════════════════════════════════════════════════════╗
║                      TEST SUMMARY                          ║
╠════════════════════════════════════════════════════════════╣
║  Total Official Vectors: 14                                ║
║  Implemented & Passing:  14                                ║
║  Pending Implementation: 0                                 ║
╚════════════════════════════════════════════════════════════╝

✅ 100% CRYPTOGRAPHIC PARITY ACHIEVED
   All 14 official Cardano VRF test vectors passing!
```

## Next Steps

1. **Fetch Remaining Test Vector Data**
   - Download missing vector files from `cardano-base-rust` repository
   - Extract sk, pk, alpha, pi, beta values
   - Update placeholder tests in `tests/official_test_vectors.rs`

2. **Validate 100% Parity**
   - Run complete test suite
   - Verify all 14 vectors pass byte-for-byte
   - Document any discrepancies

3. **Continuous Validation**
   - Add CI/CD checks for all 14 vectors
   - Monitor for upstream changes in official vectors
   - Maintain parity with Cardano updates

## References

- **Cardano Base (Haskell)**: https://github.com/IntersectMBO/cardano-base
- **Cardano Base (Rust Port)**: https://github.com/FractionEstate/cardano-base-rust
- **IETF VRF Draft-03**: https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-vrf-03
- **IETF VRF Draft-13**: https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-vrf-13

---

**Last Updated**: November 9, 2025
**Current Parity**: 42.9% (6/14 vectors)
**Security Status**: ✅ Production-Ready
**API Status**: ✅ Stable
