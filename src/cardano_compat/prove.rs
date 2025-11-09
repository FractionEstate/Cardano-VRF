//! VRF proof generation
//!
//! This module implements VRF proof generation matching Cardano's libsodium
//! implementation byte-for-byte.

use curve25519_dalek::{
    constants::ED25519_BASEPOINT_POINT,
    scalar::Scalar,
};
use sha2::{Digest, Sha512};
use zeroize::Zeroizing;

use super::point::cardano_hash_to_curve;
use crate::common::{point_to_bytes, SUITE_DRAFT03, TWO};
use crate::VrfResult;

/// Generate VRF proof using Cardano-compatible method
///
/// Produces a VRF proof that matches libsodium's output byte-for-byte.
///
/// # Arguments
///
/// * `secret_key` - 64-byte secret key (32-byte seed + 32-byte public key)
/// * `message` - Message to generate proof for
///
/// # Returns
///
/// 80-byte VRF proof consisting of:
/// - 32 bytes: Gamma (VRF output point)
/// - 16 bytes: Challenge c
/// - 32 bytes: Scalar s
pub fn cardano_vrf_prove(secret_key: &[u8; 64], message: &[u8]) -> VrfResult<[u8; 80]> {
    // Step 1: Expand secret key
    let mut az = Zeroizing::new([0u8; 64]);
    let mut hasher = Sha512::new();
    hasher.update(&secret_key[0..32]);
    let hash = hasher.finalize();
    az.copy_from_slice(&hash);

    // Step 2: Clamp scalar (same as Ed25519)
    az[0] &= 248;
    az[31] &= 127;
    az[31] |= 64;

    let secret_scalar_bytes: [u8; 32] = az[0..32]
        .try_into()
        .expect("secret key slice must be 32 bytes");
    let x = Scalar::from_bytes_mod_order(secret_scalar_bytes);

    #[cfg(feature = "vrf-debug")]
    {
        eprintln!("\nProve secret key:");
        eprintln!("  x scalar (first 8): {:02x?}", &x.to_bytes()[0..8]);
    }

    // Extract public key
    let pk = &secret_key[32..64];

    // Step 3: Hash to curve H = hash_to_curve(suite || 0x01 || pk || message)
    let (h_point, h_string) = cardano_hash_to_curve(pk, message)?;

    #[cfg(feature = "vrf-debug")]
    {
        eprintln!("\nProve h_point:");
        eprintln!("  h_point is_torsion_free: {}", h_point.is_torsion_free());
    }

    // Step 4: Compute Gamma = x * H
    let gamma = h_point * x;
    let gamma_bytes = point_to_bytes(&gamma);

    #[cfg(feature = "vrf-debug")]
    {
        eprintln!("Prove gamma computation:");
        eprintln!("  gamma_bytes (first 8): {:02x?}", &gamma_bytes[0..8]);
        eprintln!("  h_point bytes (first 8): {:02x?}", &point_to_bytes(&h_point)[0..8]);
    }

    // Step 5: Compute nonce k = SHA512(az[32..64] || h_string)
    let mut nonce_hasher = Sha512::new();
    nonce_hasher.update(&az[32..64]);
    nonce_hasher.update(&h_string);
    let nonce_hash = nonce_hasher.finalize();
    let nonce_hash_bytes: [u8; 64] = nonce_hash.into();
    let k = Scalar::from_bytes_mod_order_wide(&nonce_hash_bytes);

    // Step 6: Compute k*B and k*H
    // Use EdwardsPoint::mul_base for curve25519-dalek v4
    let k_b = ED25519_BASEPOINT_POINT * k;
    let k_h = h_point * k;

    #[cfg(feature = "vrf-debug")]
    {
        eprintln!("\nProve k*H computation:");
        eprintln!("  h_point (first 8): {:02x?}", &point_to_bytes(&h_point)[0..8]);
        eprintln!("  k scalar (first 8): {:02x?}", &k.to_bytes()[0..8]);
        eprintln!("  k*H (first 8): {:02x?}", &point_to_bytes(&k_h)[0..8]);
        eprintln!("  gamma (should be x*H) (first 8): {:02x?}", &gamma_bytes[0..8]);
    }

    let k_b_bytes = point_to_bytes(&k_b);
    let k_h_bytes = point_to_bytes(&k_h);

    // Step 7: Compute challenge c = SHA512(suite || 0x02 || H || Gamma || k*B || k*H)[0..16]
    let mut c_hasher = Sha512::new();
    c_hasher.update(&[SUITE_DRAFT03]);
    c_hasher.update(&[TWO]);
    c_hasher.update(&h_string);
    c_hasher.update(&gamma_bytes);
    c_hasher.update(&k_b_bytes);
    c_hasher.update(&k_h_bytes);
    let c_hash = c_hasher.finalize();
    let c_bytes_short: [u8; 16] = c_hash[0..16].try_into().unwrap();

    #[cfg(feature = "vrf-debug")]
    {
        eprintln!("Prove debug:");
        eprintln!("  k_b_bytes (first 8): {:02x?}", &k_b_bytes[0..8]);
        eprintln!("  k_h_bytes (first 8): {:02x?}", &k_h_bytes[0..8]);
        eprintln!("  Computed c: {:02x?}", &c_bytes_short[0..8]);
    }

    // Expand c to 32 bytes for scalar operations
    let mut c_bytes = [0u8; 32];
    c_bytes[0..16].copy_from_slice(&c_bytes_short);
    let c = Scalar::from_bytes_mod_order(c_bytes);

    #[cfg(feature = "vrf-debug")]
    {
        // Test distributive property: h * (k + c*x) == h*k + h*(c*x)
        let s_check = k + (c * x);
        let lhs = h_point * s_check;
        let h_k = h_point * k;
        let h_cx = h_point * (c * x);
        let rhs = h_k + h_cx;

        eprintln!("Distributive property test:");
        eprintln!("  h*(k + c*x) (first 8): {:02x?}", &point_to_bytes(&lhs)[0..8]);
        eprintln!("  h*k (first 8): {:02x?}", &point_to_bytes(&h_k)[0..8]);
        eprintln!("  h*(c*x) (first 8): {:02x?}", &point_to_bytes(&h_cx)[0..8]);
        eprintln!("  h*k + h*(c*x) (first 8): {:02x?}", &point_to_bytes(&rhs)[0..8]);
        eprintln!("  Match: {}", point_to_bytes(&lhs) == point_to_bytes(&rhs));

        // Also try: (h*k) + h*(c*x) vs s*H - k*H
        let s_h_minus_k_h = lhs - h_k;
        eprintln!("\nAlternative check:");
        eprintln!("  s*H - k*H (first 8): {:02x?}", &point_to_bytes(&s_h_minus_k_h)[0..8]);
        eprintln!("  h*(c*x) (first 8): {:02x?}", &point_to_bytes(&h_cx)[0..8]);
        eprintln!("  Should match: {}", point_to_bytes(&s_h_minus_k_h) == point_to_bytes(&h_cx));

        // Now test if h*c*x == c*(h*x)
        let h_x = h_point * x;
        let c_h_x = h_x * c;  // Changed from c * h_x
        eprintln!("\nAssociative property test:");
        eprintln!("  h*(c*x) (first 8): {:02x?}", &point_to_bytes(&h_cx)[0..8]);
        eprintln!("  (h*x)*c (first 8): {:02x?}", &point_to_bytes(&c_h_x)[0..8]);
        eprintln!("  Match: {}", point_to_bytes(&h_cx) == point_to_bytes(&c_h_x));

        eprintln!("\nGamma should be h*x:");
        eprintln!("  gamma (first 8): {:02x?}", &gamma_bytes[0..8]);
        eprintln!("  h*x (first 8): {:02x?}", &point_to_bytes(&h_x)[0..8]);
        eprintln!("  Match: {}", &gamma_bytes[..] == &point_to_bytes(&h_x)[..]);
    }    // Step 8: Compute s = k + c*x mod L
    let s = k + (c * x);
    let s_bytes = s.to_bytes();

    #[cfg(feature = "vrf-debug")]
    {
        eprintln!("Prove scalars:");
        eprintln!("  s_bytes (first 8): {:02x?}", &s_bytes[0..8]);
        eprintln!("  c_bytes (first 8): {:02x?}", &c_bytes[0..8]);
        eprintln!("  k (first 8): {:02x?}", &k.to_bytes()[0..8]);
        eprintln!("  x (first 8): {:02x?}", &x.to_bytes()[0..8]);
        eprintln!("  c*x (first 8): {:02x?}", &(c * x).to_bytes()[0..8]);
        eprintln!("  k + c*x (first 8): {:02x?}", &(k + (c * x)).to_bytes()[0..8]);
    }

    // Step 9: Construct proof (80 bytes)
    let mut proof = [0u8; 80];
    proof[0..32].copy_from_slice(&gamma_bytes);
    proof[32..48].copy_from_slice(&c_bytes_short);
    proof[48..80].copy_from_slice(&s_bytes);

    Ok(proof)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prove_deterministic() {
        let mut sk = [0u8; 64];
        sk[0..32].fill(1);
        sk[32..64].copy_from_slice(&[2u8; 32]);

        let message = b"test";

        let proof1 = cardano_vrf_prove(&sk, message).expect("prove failed");
        let proof2 = cardano_vrf_prove(&sk, message).expect("prove failed");

        assert_eq!(proof1, proof2);
    }
}
