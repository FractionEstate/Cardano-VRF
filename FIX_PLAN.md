# VRF Implementation Fix Plan

## Current Status
✅ Package compiles (0 errors, 8 warnings)
❌ VRF verification fails - challenge mismatch
❌ Mathematical properties appear to fail in VRF context

## Research Findings

### Critical Differences from Reference Implementation

1. **Basepoint Multiplication**
   - Reference: `&k * ED25519_BASEPOINT_TABLE` (curve25519-dalek v3 style)
   - Ours: `EdwardsPoint::mul_base(&k)` (curve25519-dalek v4 style)
   - Impact: May produce different results due to internal implementation changes

2. **Verification Method**
   - Reference uses `vartime_multiscalar_mul` for batch operations
   - We use separate scalar multiplication and subtraction
   - Impact: Different reduction/rounding behavior

3. **Point vs Scalar Operation Order**
   - Reference: `h_point * k` (EdwardsPoint * Scalar)
   - Ours: Sometimes `k * h_point` or `h_point * k`
   - Impact: In some curve25519-dalek versions, order matters

## Action Plan

### Phase 1: Use Batch Scalar Multiplication (HIGHEST PRIORITY)

**Goal:** Match the reference's use of `vartime_multiscalar_mul`

**Files to modify:**
1. `src/cardano_compat/verify.rs`
2. `src/draft13.rs` (already partially done)

**Changes:**

#### verify.rs:
```rust
// OLD (current):
let s_b = EdwardsPoint::mul_base(&s);
let c_y = c * y_point;
let k_b = s_b - c_y;

let s_h = h_point * s;
let c_gamma = c * gamma;
let k_h = s_h - c_gamma;

// NEW (use batch multiplication):
use curve25519_dalek::traits::VartimeMultiscalarMul;

let neg_c = -c;
let k_b = EdwardsPoint::vartime_multiscalar_mul(
    &[s, neg_c],
    &[&constants::ED25519_BASEPOINT_POINT, &y_point],
);

let k_h = EdwardsPoint::vartime_multiscalar_mul(
    &[s, neg_c],
    &[&h_point, &gamma],
);
```

**Rationale:** The reference implementation uses this method which internally handles scalar arithmetic more carefully.

### Phase 2: Verify Basepoint Multiplication Consistency

**Goal:** Ensure basepoint multiplication matches reference

**Investigation needed:**
1. Test if `EdwardsPoint::mul_base(&k)` ≡ `&k * ED25519_BASEPOINT_TABLE` in v4.1
2. If not, use the explicit table multiplication

**Test code:**
```rust
#[test]
fn test_basepoint_mul_equivalence() {
    use curve25519_dalek::constants::ED25519_BASEPOINT_TABLE;
    let k = Scalar::from(12345u64);

    let method1 = EdwardsPoint::mul_base(&k);
    let method2 = &k * ED25519_BASEPOINT_TABLE;

    assert_eq!(method1.compress(), method2.compress());
}
```

### Phase 3: Point Operation Order Consistency

**Goal:** Ensure EdwardsPoint * Scalar order is consistent

**Files to audit:**
- `src/cardano_compat/prove.rs`
- `src/cardano_compat/verify.rs`
- `src/draft03.rs`
- `src/draft13.rs`

**Standard:** Always use `point * scalar` (not `scalar * point`)

### Phase 4: Import Cleanup

**Goal:** Match reference's import style

**Current:**
```rust
use curve25519_dalek::{constants, scalar::Scalar, EdwardsPoint};
```

**Should be:**
```rust
use curve25519_dalek::{
    constants::ED25519_BASEPOINT_POINT,
    edwards::EdwardsPoint,
    scalar::Scalar,
    traits::VartimeMultiscalarMul,
};
```

## Implementation Steps

### Step 1: Fix verify.rs

```rust
// File: src/cardano_compat/verify.rs

// Add to imports:
use curve25519_dalek::traits::VartimeMultiscalarMul;
use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;

// Replace verification equations (around line 100-110):
pub fn cardano_vrf_verify(
    public_key: &[u8; 32],
    proof: &[u8; 80],
    message: &[u8],
) -> VrfResult<[u8; 64]> {
    // ... existing code up to scalar parsing ...

    // Parse challenge and response
    let mut c_bytes = [0u8; 32];
    c_bytes[0..16].copy_from_slice(&c_bytes_short);
    let c = Scalar::from_bytes_mod_order(c_bytes);
    let s = Scalar::from_bytes_mod_order(s_bytes);

    // CRITICAL FIX: Use batch scalar multiplication
    let neg_c = -c; // Negate c for batch computation

    // Compute k*B = s*B - c*Y using batch multiplication
    let k_b = EdwardsPoint::vartime_multiscalar_mul(
        &[s, neg_c],
        &[&ED25519_BASEPOINT_POINT, &y_point],
    );

    // Compute k*H = s*H - c*Gamma using batch multiplication
    let k_h = EdwardsPoint::vartime_multiscalar_mul(
        &[s, neg_c],
        &[&h_point, &gamma],
    );

    // ... rest of verification ...
}
```

### Step 2: Verify prove.rs consistency

Ensure prove.rs uses consistent operations:
```rust
// Check that we're using:
let gamma = h_point * x;  // NOT x * h_point
let k_h = h_point * k;     // NOT k * h_point
```

### Step 3: Test with Debug Output

Add extensive debug output to compare with reference:
```rust
#[cfg(feature = "vrf-debug")]
{
    eprintln!("=== VRF VERIFY DEBUG ===");
    eprintln!("s scalar: {:?}", s.to_bytes());
    eprintln!("c scalar: {:?}", c.to_bytes());
    eprintln!("neg_c scalar: {:?}", neg_c.to_bytes());
    eprintln!("y_point: {:?}", y_point.compress().to_bytes());
    eprintln!("h_point: {:?}", h_point.compress().to_bytes());
    eprintln!("gamma: {:?}", gamma.compress().to_bytes());
    eprintln!("k_b computed: {:?}", k_b.compress().to_bytes());
    eprintln!("k_h computed: {:?}", k_h.compress().to_bytes());
    eprintln!("======================");
}
```

### Step 4: Run Tests

```bash
# Run with debug output
CARDANO_VRF_DEBUG=1 cargo test --features vrf-debug -- --nocapture test_basic_prove_verify

# Run all tests
cargo test

# Run specific failing test
cargo test test_verify_official_vector
```

## Expected Outcomes

### Success Criteria
✅ All test vectors pass
✅ Prove/verify roundtrip succeeds
✅ Challenge values match expected
✅ No mathematical property failures

### If Still Failing
1. Check curve25519-dalek version compatibility
2. Consider downgrading to v3.x as reference uses
3. Examine scalar reduction behavior
4. Compare against reference implementation's test output byte-by-byte

## Timeline

1. **Immediate (30 min):** Implement vartime_multiscalar_mul in verify.rs
2. **Short term (1 hour):** Test and debug with official vectors
3. **Medium term (2 hours):** If still failing, investigate curve25519-dalek version differences
4. **Long term (4 hours):** If needed, implement custom field operations matching reference

## Notes

- The reference emphasizes "byte-for-byte" matching with libsodium
- They use custom FieldElement operations in some places
- The VRF_PARITY_COMPLETE.md shows they achieved exact parity with all test vectors
- Key insight: It's not just about the math, but HOW the library implements it

## References

- FractionEstate/cardano-base-rust: cardano-vrf-pure implementation
- Reference prove.rs uses `&k * ED25519_BASEPOINT_TABLE`
- Reference verify.rs uses `vartime_multiscalar_mul` for Draft-13
- VRF_PARITY_COMPLETE.md documents their exact parity achievement
