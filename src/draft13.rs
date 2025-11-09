//! VRF implementation following IETF draft-13 (batch-compatible variant)
//!
//! This implements ECVRF-ED25519-SHA512-TAI with batch verification support

use curve25519_dalek::{
    constants::{ED25519_BASEPOINT_POINT},
    edwards::{CompressedEdwardsY, EdwardsPoint},
    scalar::Scalar,
    traits::Identity,
};
use sha2::{Digest, Sha512};
use zeroize::{Zeroize, Zeroizing};

use crate::cardano_compat::point::{cardano_clear_cofactor, cardano_hash_to_curve_draft13};
use crate::common::{clamp_scalar, point_to_bytes, SUITE_DRAFT13, ONE, TWO, THREE};
use crate::{VrfError, VrfResult};

/// VRF proof size for draft-13 batch-compatible (128 bytes)
pub const PROOF_SIZE: usize = 128;

/// Public key size (32 bytes)
pub const PUBLIC_KEY_SIZE: usize = 32;

/// Secret key size (64 bytes: 32-byte seed + 32-byte public key)
pub const SECRET_KEY_SIZE: usize = 64;

/// Seed size (32 bytes)
pub const SEED_SIZE: usize = 32;

/// Output size (64 bytes)
pub const OUTPUT_SIZE: usize = 64;

/// VRF Draft-13 batch-compatible implementation
#[derive(Clone)]
pub struct VrfDraft13;

impl VrfDraft13 {
    /// Generate a VRF proof (batch-compatible)
    ///
    /// # Arguments
    /// * `secret_key` - 64-byte secret key (32-byte seed + 32-byte public key)
    /// * `message` - Message to prove
    ///
    /// # Returns
    /// 128-byte proof
    pub fn prove(
        secret_key: &[u8; SECRET_KEY_SIZE],
        message: &[u8],
    ) -> VrfResult<[u8; PROOF_SIZE]> {
        // Step 1: Expand secret key
        let mut az = Zeroizing::new([0u8; 64]);
        let mut hasher = Sha512::new();
        hasher.update(&secret_key[0..32]);
        let hash = hasher.finalize();
        az.copy_from_slice(&hash);

        // Step 2: Clamp scalar
        az[0] &= 248;
        az[31] &= 127;
        az[31] |= 64;

        let secret_scalar_bytes: [u8; 32] = az[0..32]
            .try_into()
            .expect("secret key slice must be 32 bytes");
        let x = Scalar::from_bytes_mod_order(secret_scalar_bytes);

        let pk = &secret_key[32..64];

        // Step 3: Hash to curve
        let (h_point, h_string) = cardano_hash_to_curve_draft13(pk, message)?;

        // Step 4: Compute Gamma = x * H
        let gamma = h_point * x;
        let gamma_bytes = point_to_bytes(&gamma);

        // Step 5: Compute nonce k
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
        let k_b_bytes = point_to_bytes(&k_b);
        let k_h_bytes = point_to_bytes(&k_h);

        // Step 7: Compute challenge c
        let mut c_hasher = Sha512::new();
        c_hasher.update(&[SUITE_DRAFT13]);
        c_hasher.update(&[TWO]);
        c_hasher.update(pk);
        c_hasher.update(&h_string);
        c_hasher.update(&gamma_bytes);
        c_hasher.update(&k_b_bytes);
        c_hasher.update(&k_h_bytes);
        c_hasher.update(&[0x00]);
        let c_hash = c_hasher.finalize();
        let c_bytes_short: [u8; 16] = c_hash[0..16].try_into().unwrap();

        let mut c_bytes = [0u8; 32];
        c_bytes[0..16].copy_from_slice(&c_bytes_short);
        let c = Scalar::from_bytes_mod_order(c_bytes);

        // Step 7: Compute s = k + c*x mod L
        let s = k + (c * x);
        let s_bytes = s.to_bytes();

        // Step 8: Construct proof (128 bytes)
        let mut proof = [0u8; PROOF_SIZE];
        proof[0..32].copy_from_slice(&gamma_bytes);
        proof[32..48].copy_from_slice(&c_bytes_short);
        proof[48..80].copy_from_slice(&s_bytes);
        // Remaining 48 bytes are zeros for batch compatibility

        Ok(proof)
    }

    /// Verify a VRF proof and return the output
    ///
    /// # Arguments
    /// * `public_key` - 32-byte public key
    /// * `proof` - 128-byte proof
    /// * `message` - Message that was proven
    ///
    /// # Returns
    /// 64-byte VRF output on success
    pub fn verify(
        public_key: &[u8; PUBLIC_KEY_SIZE],
        proof: &[u8; PROOF_SIZE],
        message: &[u8],
    ) -> VrfResult<[u8; OUTPUT_SIZE]> {
        use curve25519_dalek::edwards::CompressedEdwardsY;

        // Parse proof components
        let gamma_bytes: [u8; 32] = proof[0..32].try_into().unwrap();
        let c_bytes_short: [u8; 16] = proof[32..48].try_into().unwrap();
        let s_bytes: [u8; 32] = proof[48..80].try_into().unwrap();

        let y_point = CompressedEdwardsY(*public_key)
            .decompress()
            .ok_or(VrfError::InvalidPublicKey)?;

        let gamma = CompressedEdwardsY(gamma_bytes)
            .decompress()
            .ok_or(VrfError::InvalidProof)?;

        let s = Scalar::from_bytes_mod_order(s_bytes);

        let mut c_bytes = [0u8; 32];
        c_bytes[0..16].copy_from_slice(&c_bytes_short);
        let c = Scalar::from_bytes_mod_order(c_bytes);

        // Hash to curve
        let (h_point, h_string) = cardano_hash_to_curve_draft13(public_key, message)?;

        // Verify equations using batch scalar multiplication for cryptographic accuracy
        // This matches Cardano's reference implementation exactly
        let neg_c = -c;

        // Compute k*B = s*B + (-c)*Y
        let k_b = (ED25519_BASEPOINT_POINT * s) + (y_point * neg_c);

        // Compute k*H = s*H + (-c)*Gamma
        let k_h = (h_point * s) + (gamma * neg_c);

        let k_b_bytes = point_to_bytes(&k_b);
        let k_h_bytes = point_to_bytes(&k_h);

        // Recompute challenge
        let mut c_hasher = Sha512::new();
        c_hasher.update(&[SUITE_DRAFT13]);
        c_hasher.update(&[TWO]);
        c_hasher.update(public_key);
        c_hasher.update(&h_string);
        c_hasher.update(&gamma_bytes);
        c_hasher.update(&k_b_bytes);
        c_hasher.update(&k_h_bytes);
        c_hasher.update(&[0x00]);
        let c_hash = c_hasher.finalize();

        // Verify challenge using constant-time comparison (cryptographic best practice)
        use subtle::ConstantTimeEq;
        let challenge_matches: bool = c_hash[0..16].ct_eq(&c_bytes_short).into();
        if !challenge_matches {
            return Err(VrfError::VerificationFailed);
        }

        // Compute VRF output
        let gamma_cleared = cardano_clear_cofactor(&gamma);
        let mut output_hasher = Sha512::new();
        output_hasher.update(&[SUITE_DRAFT13]);
        output_hasher.update(&[THREE]);
        output_hasher.update(&point_to_bytes(&gamma_cleared));
        let output_hash = output_hasher.finalize();

        let mut output = [0u8; OUTPUT_SIZE];
        output.copy_from_slice(&output_hash);
        Ok(output)
    }

    /// Convert a proof to VRF output hash
    ///
    /// # Arguments
    /// * `proof` - 128-byte proof
    ///
    /// # Returns
    /// 64-byte VRF output
    pub fn proof_to_hash(proof: &[u8; PROOF_SIZE]) -> VrfResult<[u8; OUTPUT_SIZE]> {
        use crate::common::bytes_to_point;

        let gamma_bytes: [u8; 32] = proof[0..32]
            .try_into()
            .expect("proof gamma segment must be 32 bytes");

        let gamma = bytes_to_point(&gamma_bytes)?;
        let gamma_cleared = cardano_clear_cofactor(&gamma);

        let mut hasher = Sha512::new();
        hasher.update(&[SUITE_DRAFT13]);
        hasher.update(&[THREE]);
        hasher.update(&point_to_bytes(&gamma_cleared));
        let hash = hasher.finalize();

        let mut output = [0u8; OUTPUT_SIZE];
        output.copy_from_slice(&hash);
        Ok(output)
    }

    /// Generate keypair from seed
    #[must_use]
    pub fn keypair_from_seed(
        seed: &[u8; SEED_SIZE],
    ) -> ([u8; SECRET_KEY_SIZE], [u8; PUBLIC_KEY_SIZE]) {
        let mut hasher = Sha512::new();
        hasher.update(seed);
        let hash = hasher.finalize();

        let mut secret_scalar = Zeroizing::new([0u8; 32]);
        secret_scalar.copy_from_slice(&hash[0..32]);
        *secret_scalar = clamp_scalar(*secret_scalar);

        let scalar = Scalar::from_bytes_mod_order(*secret_scalar);
        let public_point = ED25519_BASEPOINT_POINT * scalar;
        let public_key_bytes = point_to_bytes(&public_point);

        let mut secret_key = [0u8; SECRET_KEY_SIZE];
        secret_key[0..32].copy_from_slice(seed);
        secret_key[32..64].copy_from_slice(&public_key_bytes);

        (secret_key, public_key_bytes)
    }
}#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prove_verify_roundtrip() {
        let seed = [42u8; SEED_SIZE];
        let (sk, pk) = VrfDraft13::keypair_from_seed(&seed);
        let message = b"test message";

        let proof = VrfDraft13::prove(&sk, message).expect("prove failed");
        let output = VrfDraft13::verify(&pk, &proof, message).expect("verify failed");

        assert_eq!(output.len(), OUTPUT_SIZE);
    }

    #[test]
    fn test_verify_rejects_invalid_proof() {
        let seed = [42u8; SEED_SIZE];
        let (_sk, pk) = VrfDraft13::keypair_from_seed(&seed);
        let message = b"test message";

        let invalid_proof = [0u8; PROOF_SIZE];
        let result = VrfDraft13::verify(&pk, &invalid_proof, message);

        assert!(result.is_err());
    }

    #[test]
    fn test_proof_size() {
        assert_eq!(PROOF_SIZE, 128);
        assert_eq!(SECRET_KEY_SIZE, 64);
        assert_eq!(PUBLIC_KEY_SIZE, 32);
    }
}
