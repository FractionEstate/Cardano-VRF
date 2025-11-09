//! VRF implementation following IETF draft-03 (ECVRF-ED25519-SHA512-Elligator2)
//!
//! This implements the VRF algorithm with 80-byte proofs.

use curve25519_dalek::{constants::ED25519_BASEPOINT_POINT, scalar::Scalar};
use sha2::{Digest, Sha512};
use zeroize::Zeroizing;

use crate::cardano_compat::{cardano_vrf_prove, cardano_vrf_verify};
use crate::common::{clamp_scalar, point_to_bytes};
use crate::VrfResult;

/// VRF proof size for draft-03 (80 bytes)
pub const PROOF_SIZE: usize = 80;

/// Public key size (32 bytes)
pub const PUBLIC_KEY_SIZE: usize = 32;

/// Secret key size (64 bytes: 32-byte seed + 32-byte public key)
pub const SECRET_KEY_SIZE: usize = 64;

/// Seed size (32 bytes)
pub const SEED_SIZE: usize = 32;

/// Output size (64 bytes)
pub const OUTPUT_SIZE: usize = 64;

/// VRF Draft-03 implementation
#[derive(Clone)]
pub struct VrfDraft03;

impl VrfDraft03 {
    /// Generate a VRF proof
    ///
    /// # Arguments
    /// * `secret_key` - 64-byte secret key (32-byte seed + 32-byte public key)
    /// * `message` - Message to prove
    ///
    /// # Returns
    /// 80-byte proof
    ///
    /// # Errors
    ///
    /// Returns `VrfError` if the proof generation fails.
    ///
    /// # Panics
    ///
    /// May panic if internal cryptographic operations fail (extremely unlikely).
    pub fn prove(
        secret_key: &[u8; SECRET_KEY_SIZE],
        message: &[u8],
    ) -> VrfResult<[u8; PROOF_SIZE]> {
        cardano_vrf_prove(secret_key, message)
    }

    /// Verify a VRF proof and return the output
    ///
    /// # Arguments
    /// * `public_key` - 32-byte public key
    /// * `proof` - 80-byte proof
    /// * `message` - Message that was proven
    ///
    /// # Returns
    /// 64-byte VRF output on success
    pub fn verify(
        public_key: &[u8; PUBLIC_KEY_SIZE],
        proof: &[u8; PROOF_SIZE],
        message: &[u8],
    ) -> VrfResult<[u8; OUTPUT_SIZE]> {
        cardano_vrf_verify(public_key, proof, message)
    }

    /// Convert a proof to VRF output hash
    ///
    /// # Arguments
    /// * `proof` - 80-byte proof
    ///
    /// # Returns
    /// 64-byte VRF output
    pub fn proof_to_hash(proof: &[u8; PROOF_SIZE]) -> VrfResult<[u8; OUTPUT_SIZE]> {
        use crate::cardano_compat::point::cardano_clear_cofactor;
        use crate::common::{bytes_to_point, SUITE_DRAFT03, THREE};

        let gamma_bytes: [u8; 32] = proof[0..32]
            .try_into()
            .expect("proof gamma segment must be 32 bytes");

        let gamma = bytes_to_point(&gamma_bytes)?;
        let gamma_cleared = cardano_clear_cofactor(&gamma);

        let mut hasher = Sha512::new();
        hasher.update([SUITE_DRAFT03]);
        hasher.update([THREE]);
        hasher.update(point_to_bytes(&gamma_cleared));
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
        let public_key = point_to_bytes(&public_point);

        let mut secret_key = [0u8; SECRET_KEY_SIZE];
        secret_key[0..32].copy_from_slice(seed);
        secret_key[32..64].copy_from_slice(&public_key);

        (secret_key, public_key)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prove_verify_roundtrip() {
        let seed = [42u8; SEED_SIZE];
        let (sk, pk) = VrfDraft03::keypair_from_seed(&seed);
        let message = b"test message";

        let proof = VrfDraft03::prove(&sk, message).expect("prove failed");
        let output = VrfDraft03::verify(&pk, &proof, message).expect("verify failed");

        assert_eq!(output.len(), OUTPUT_SIZE);
    }

    #[test]
    fn test_verify_rejects_invalid_proof() {
        let seed = [42u8; SEED_SIZE];
        let (_sk, pk) = VrfDraft03::keypair_from_seed(&seed);
        let message = b"test message";

        let invalid_proof = [0u8; PROOF_SIZE];
        let result = VrfDraft03::verify(&pk, &invalid_proof, message);

        assert!(result.is_err());
    }

    #[test]
    fn test_proof_to_hash_deterministic() {
        let seed = [42u8; SEED_SIZE];
        let (sk, _pk) = VrfDraft03::keypair_from_seed(&seed);
        let message = b"test message";

        let proof = VrfDraft03::prove(&sk, message).expect("prove failed");
        let hash1 = VrfDraft03::proof_to_hash(&proof).expect("hash failed");
        let hash2 = VrfDraft03::proof_to_hash(&proof).expect("hash failed");

        assert_eq!(hash1, hash2);
    }
}
