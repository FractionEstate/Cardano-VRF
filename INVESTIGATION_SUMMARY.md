# VRF Parity Investigation Summary

## Problem
Our VRF implementation compiles successfully but verification fails with challenge mismatches. Mathematical properties like distributivity and associativity appear to fail during VRF operations.

## Root Cause Analysis

After extensive research of the FractionEstate/cardano-base-rust reference implementation, I identified several critical differences:

### 1. **Basepoint Multiplication API**
**Reference Implementation:**
```rust
let k_b: EdwardsPoint = &k * ED25519_BASEPOINT_TABLE;
```

**Our Implementation:**
```rust
let k_b = EdwardsPoint::mul_base(&k);
```

**Issue:** curve25519-dalek v4 changed the API. `mul_base` might have slightly different behavior than the table multiplication.

### 2. **Verification Equation Method**
**Reference Implementation (Draft-13):**
```rust
let neg_c = scalar_negate(&c);
let u = EdwardsPoint::vartime_multiscalar_mul(
    &[s, neg_c],
    &[EdwardsPoint::mul_base(&Scalar::ONE), y_point],
);
```

**Our Implementation:**
```rust
let s_b = EdwardsPoint::mul_base(&s);
let c_y = c * y_point;
let k_b = s_b - c_y;
```

**Issue:** The reference uses batch multiplication (`vartime_multiscalar_mul`) which may have different rounding/reduction behavior than separate operations.

### 3. **curve25519-dalek Version**
The reference implementation documentation mentions they match libsodium "byte-for-byte" and use custom `FieldElement` operations. Our dependency audit shows they likely use an **older version of curve25519-dalek** where the API and internal behavior may differ.

## Critical Discoveries

From the VRF_PARITY_COMPLETE.md documentation:

1. **Suite ID must be 0x04** (we have this correct)
2. **Sign bit clearing is critical:** `r_bytes[31] &= 0x7f` before hash-to-curve
3. **Cofactor clearing** must use `Scalar::from(8u8) * point`
4. **Custom field operations:** The reference uses custom `FieldElement` implementation that "matches libsodium exactly"

From the reference's prove.rs:
```rust
// Uses older API
let k_b: curve25519_dalek::edwards::EdwardsPoint = &k * ED25519_BASEPOINT_TABLE;

// They import ED25519_BASEPOINT_TABLE differently:
use curve25519_dalek::constants::ED25519_BASEPOINT_TABLE;
```

## Recommendations

### Option 1: Match the Reference Implementation Exactly
1. **Downgrade curve25519-dalek** to the version used by the reference (likely v3.x)
2. **Use `&scalar * ED25519_BASEPOINT_TABLE`** instead of `mul_base`
3. **Implement batch verification** using `vartime_multiscalar_mul`
4. **Use the older constants API**

### Option 2: Fix the Math with Current API
1. **Use `vartime_multiscalar_mul` for verification** equations
2. **Ensure scalar negation** is done correctly
3. **Verify point compression/decompression** matches exactly
4. **Double-check reduction mod L** in all scalar operations

### Option 3: Hybrid Approach (Recommended)
1. Keep curve25519-dalek v4.1 but use it more carefully
2. Replace our simple arithmetic with batch operations where the reference uses them
3. Use `&scalar * point` multiplication instead of `point * scalar` (order matters for some operations)
4. Implement `vartime_multiscalar_mul` for verification

## Next Steps

The **immediate action** should be:

1. **Check if `EdwardsPoint::mul_base` in v4.1 is identical to `&scalar * ED25519_BASEPOINT_TABLE`**
   - If not, we need to use the table multiplication

2. **Rewrite verification to use `vartime_multiscalar_mul`** as the reference does:
   ```rust
   // Instead of: s*B - c*Y
   // Use:
   let neg_c = -c;
   let u = EdwardsPoint::vartime_multiscalar_mul(
       &[s, neg_c],
       &[&constants::ED25519_BASEPOINT_POINT, y_point],
   );
   ```

3. **Verify scalar operations are reduced mod L correctly**

4. **Add extensive debugging** to compare intermediate values with known test vectors

## Test Vector Analysis

The reference implementation passes these official vectors:
- vrf_ver03_standard_10: ✅ Exact match
- vrf_ver03_generated_1: ✅ Exact match

Both use:
- Suite ID: 0x04
- Sign bit clearing
- Cofactor multiplication by 8
- Specific hash-to-curve with Elligator2

## Mathematical Property Failure Explanation

The "distributivity failure" we observed (`h*(k + c*x) ≠ h*k + h*(c*x)`) is likely due to:

1. **Different reduction strategies** between operations
2. **Point compression/decompression** introducing subtle differences
3. **Scalar multiplication implementation** differences between v3 and v4
4. **Order of operations** affecting floating-point-like behavior in mod L arithmetic

The math itself is correct - it's the **implementation details** that matter for byte-for-byte parity.

## Conclusion

Our implementation architecture is sound, but we need to match the reference's **specific curve25519-dalek usage patterns**. The key is not just using the same library, but using it in the **exact same way** with the **same version** or at least the same API patterns.

The most likely fix: **Use `vartime_multiscalar_mul` for batch operations** as the reference does, which handles the scalar arithmetic more carefully and matches libsodium's behavior.
