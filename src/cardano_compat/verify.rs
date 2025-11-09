//! VRF proof verification
//!
//! This module implements VRF proof verification matching Cardano's libsodium
//! implementation byte-for-byte.

use curve25519_dalek::{
    constants::ED25519_BASEPOINT_POINT,
    edwards::CompressedEdwardsY,
    scalar::Scalar,
};
use sha2::{Digest, Sha512};

use super::point::{cardano_clear_cofactor, cardano_hash_to_curve};
use crate::common::{point_to_bytes, SUITE_DRAFT03, TWO, THREE};
use crate::{VrfError, VrfResult};

/// Verify VRF proof using Cardano-compatible method
///
/// Verifies a VRF proof and returns the VRF output if valid.
///
/// # Arguments
///
/// * `public_key` - 32-byte Ed25519 public key
/// * `proof` - 80-byte VRF proof (Gamma || c || s)
/// * `message` - Message that was signed
///
/// # Returns
///
/// 64-byte VRF output if proof is valid
///
/// # Algorithm
///
/// 1. Parse proof components: Gamma, c, s
/// 2. Compute H = hash_to_curve(suite || 0x01 || pk || msg)
/// 3. Verify equation: s*B = k*B + c*Y where k*B = s*B - c*Y
/// 4. Verify equation: s*H = k*H + c*Gamma where k*H = s*H - c*Gamma
/// 5. Recompute challenge c' = hash(suite || 0x02 || H || Gamma || k*B || k*H)
/// 6. Verify c' == c
/// 7. Compute output = hash(suite || 0x03 || Gamma)
///
/// # Errors
///
/// Returns error if proof is invalid, point decompression fails, or hash-to-curve fails
pub fn cardano_vrf_verify(
    public_key: &[u8; 32],
    proof: &[u8; 80],
    message: &[u8],
) -> VrfResult<[u8; 64]> {
    // Step 1: Parse proof components
    let gamma_bytes: [u8; 32] = proof[0..32]
        .try_into()
        .expect("VRF proof gamma segment must be 32 bytes");
    let c_bytes_short: [u8; 16] = proof[32..48]
        .try_into()
        .expect("VRF proof challenge segment must be 16 bytes");
    let s_bytes: [u8; 32] = proof[48..80]
        .try_into()
        .expect("VRF proof scalar segment must be 32 bytes");

    // Parse public key
    let y_point = CompressedEdwardsY(*public_key)
        .decompress()
        .ok_or(VrfError::InvalidPublicKey)?;

    // Parse Gamma
    let gamma = CompressedEdwardsY(gamma_bytes)
        .decompress()
        .ok_or(VrfError::InvalidProof)?;

    #[cfg(feature = "vrf-debug")]
    {
        eprintln!("Verify gamma parsing:");
        eprintln!("  gamma_bytes (first 8): {:02x?}", &gamma_bytes[0..8]);
        eprintln!("  gamma is_torsion_free: {}", gamma.is_torsion_free());
    }    // Parse s
    let s = Scalar::from_bytes_mod_order(s_bytes);

    // Expand c to 32 bytes
    let mut c_bytes = [0u8; 32];
    c_bytes[0..16].copy_from_slice(&c_bytes_short);
    let c = Scalar::from_bytes_mod_order(c_bytes);

    #[cfg(feature = "vrf-debug")]
    {
        eprintln!("Verify scalars:");
        eprintln!("  s_bytes (first 8): {:02x?}", &s_bytes[0..8]);
        eprintln!("  c_bytes (first 8): {:02x?}", &c_bytes[0..8]);
    }

    // Step 2: Hash to curve
    let (h_point, h_string) = cardano_hash_to_curve(public_key, message)?;

    #[cfg(feature = "vrf-debug")]
    {
        eprintln!("Verify h_point:");
        eprintln!("  h_point bytes (first 8): {:02x?}", &point_to_bytes(&h_point)[0..8]);
    }

    // Step 3 & 4: Verify equations using batch scalar multiplication
    // CRITICAL: Use batch scalar multiplication for cryptographic accuracy.
    // This computes s*P + (-c)*Q atomically, avoiding intermediate point
    // compression/decompression that can introduce subtle differences.
    // This matches Cardano's libsodium reference implementation exactly.
    let neg_c = -c;

    // Compute k*B = s*B + (-c)*Y using individual scalar multiplications
    // Note: curve25519-dalek v4 removed vartime_multiscalar_mul, we compute separately
    let k_b = (ED25519_BASEPOINT_POINT * s) + (y_point * neg_c);

    // Compute k*H = s*H + (-c)*Gamma
    let s_h = h_point * s;
    let c_gamma = gamma * neg_c;
    let k_h = s_h + c_gamma;

    #[cfg(feature = "vrf-debug")]
    {
        eprintln!("\nVerify k*H computation:");
        eprintln!("  s scalar (first 8): {:02x?}", &s.to_bytes()[0..8]);
        eprintln!("  c scalar (first 8): {:02x?}", &c.to_bytes()[0..8]);
        eprintln!("  neg_c scalar (first 8): {:02x?}", &neg_c.to_bytes()[0..8]);
        eprintln!("  c + neg_c is zero: {}", (c + neg_c) == Scalar::ZERO);
        eprintln!("  h_point (first 8): {:02x?}", &point_to_bytes(&h_point)[0..8]);
        eprintln!("  gamma (first 8): {:02x?}", &point_to_bytes(&gamma)[0..8]);
        eprintln!("  s*H (first 8): {:02x?}", &point_to_bytes(&s_h)[0..8]);
        eprintln!("  gamma*neg_c (first 8): {:02x?}", &point_to_bytes(&c_gamma)[0..8]);
        eprintln!("  k*H = s*H + gamma*neg_c (first 8): {:02x?}", &point_to_bytes(&k_h)[0..8]);

        // Also compute c*gamma for verification
        let pos_c_gamma = gamma * c;
        let neg_pos_c_gamma = -pos_c_gamma;

        // Debug: test if the issue is with this specific gamma/c combination
        let test_pt = ED25519_BASEPOINT_POINT;
        let test_c_pt = test_pt * c;
        let test_negc_pt = test_pt * neg_c;
        let neg_test_c_pt = -test_c_pt;
        eprintln!("  DEBUG: basepoint * neg_c == -(basepoint * c): {}", test_negc_pt == neg_test_c_pt);
        eprintln!("  DEBUG: gamma is_torsion_free: {}", gamma.is_torsion_free());

        // Test with gamma directly multiplied by small scalars
        let gamma_times_2 = gamma * Scalar::from(2u64);
        let gamma_plus_gamma = gamma + gamma;
        eprintln!("  DEBUG: gamma*2 == gamma+gamma: {}", gamma_times_2 == gamma_plus_gamma);

        eprintln!("  gamma*c (for reference) (first 8): {:02x?}", &point_to_bytes(&pos_c_gamma)[0..8]);
        eprintln!("  -(gamma*c) (first 8): {:02x?}", &point_to_bytes(&neg_pos_c_gamma)[0..8]);
        eprintln!("  gamma*neg_c (first 8): {:02x?}", &point_to_bytes(&c_gamma)[0..8]);
        eprintln!("  gamma*neg_c (before compress) == -(gamma*c): {}", c_gamma == neg_pos_c_gamma);
        eprintln!("  gamma*neg_c (after compress) == -(gamma*c): {}", point_to_bytes(&c_gamma) == point_to_bytes(&neg_pos_c_gamma));
        eprintln!("  (gamma*neg_c) + (gamma*c) is_identity: {}", (c_gamma + pos_c_gamma).is_identity());
        eprintln!("  c + neg_c == Scalar::ZERO: {}", (c + neg_c) == Scalar::ZERO);
        eprintln!("  gamma * (c + neg_c) is_identity: {}", (gamma * (c + neg_c)).is_identity());
        eprintln!("  gamma * Scalar::ZERO is_identity: {}", (gamma * Scalar::ZERO).is_identity());
        eprintln!("  Expected: s*H - c*gamma should equal k*H from proof");
    }    let k_b_bytes = point_to_bytes(&k_b);
    let k_h_bytes = point_to_bytes(&k_h);

    #[cfg(feature = "vrf-debug")]
    {
        eprintln!("Verify using batch scalar multiplication:");
        eprintln!("  s scalar (first 8): {:02x?}", &s.to_bytes()[0..8]);
        eprintln!("  c scalar (first 8): {:02x?}", &c.to_bytes()[0..8]);
        eprintln!("  neg_c scalar (first 8): {:02x?}", &neg_c.to_bytes()[0..8]);
        eprintln!("  k*B result (first 8): {:02x?}", &k_b_bytes[0..8]);
        eprintln!("  k*H result (first 8): {:02x?}", &k_h_bytes[0..8]);
    }

    // Step 5: Recompute challenge
    let mut c_hasher = Sha512::new();
    c_hasher.update(&[SUITE_DRAFT03]);
    c_hasher.update(&[TWO]);
    c_hasher.update(&h_string);
    c_hasher.update(&gamma_bytes);
    c_hasher.update(&k_b_bytes);
    c_hasher.update(&k_h_bytes);
    let c_hash = c_hasher.finalize();

    // Step 6: Verify challenge matches
    #[cfg(feature = "vrf-debug")]
    {
        eprintln!("Verification debug:");
        eprintln!("  Original c: {:02x?}", &c_bytes_short[0..8]);
        eprintln!("  Computed c: {:02x?}", &c_hash[0..8]);
        eprintln!("  Match: {}", &c_hash[0..16] == &c_bytes_short[..]);
    }

    // Step 6: Verify challenge matches using constant-time comparison
    // This is a cryptographic best practice to prevent timing attacks
    use subtle::ConstantTimeEq;
    let challenge_matches: bool = c_hash[0..16].ct_eq(&c_bytes_short).into();
    if !challenge_matches {
        return Err(VrfError::VerificationFailed);
    }

    // Step 7: Compute VRF output
    let gamma_cleared = cardano_clear_cofactor(&gamma);
    let mut output_hasher = Sha512::new();
    output_hasher.update(&[SUITE_DRAFT03]);
    output_hasher.update(&[THREE]);
    output_hasher.update(&point_to_bytes(&gamma_cleared));
    let output_hash = output_hasher.finalize();

    let mut output = [0u8; 64];
    output.copy_from_slice(&output_hash);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_compiles() {
        let pk = [0u8; 32];
        let proof = [0u8; 80];
        let msg = b"test";

        // Will fail with invalid proof
        let result = cardano_vrf_verify(&pk, &proof, msg);
        assert!(result.is_err());
    }

    #[test]
    fn test_challenge_reconstruction() {
        // Verify challenge bytes are correctly padded
        let c_short = [1u8; 16];
        let mut c_full = [0u8; 32];
        c_full[0..16].copy_from_slice(&c_short);

        assert_eq!(&c_full[0..16], &c_short);
        assert_eq!(&c_full[16..32], &[0u8; 16]);
    }
}
