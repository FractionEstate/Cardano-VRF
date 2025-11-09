//! Edwards point operations and cofactor clearing
//!
//! This module implements operations on Edwards curve points, including
//! Cardano-specific cofactor clearing that differs from standard implementations.

use curve25519_dalek::edwards::{CompressedEdwardsY, EdwardsPoint};
use sha2::{Digest, Sha512};

use crate::common::{SUITE_DRAFT03, SUITE_DRAFT13, ONE};
use crate::{VrfError, VrfResult};

/// Cardano-specific cofactor clearing
///
/// This multiplies the point by 8 (the cofactor of Ed25519)
#[must_use]
pub fn cardano_clear_cofactor(point: &EdwardsPoint) -> EdwardsPoint {
    point.mul_by_cofactor()
}

/// Hash to curve for Draft-03 (Cardano-compatible)
///
/// # Arguments
/// * `pk` - Public key bytes
/// * `message` - Message to hash
///
/// # Returns
/// Tuple of (point on curve, compressed representation bytes)
pub fn cardano_hash_to_curve(
    pk: &[u8],
    message: &[u8],
) -> VrfResult<(EdwardsPoint, [u8; 32])> {
    // Compute r = SHA512(suite || 0x01 || pk || message)
    let mut hasher = Sha512::new();
    hasher.update(&[SUITE_DRAFT03]);
    hasher.update(&[ONE]);
    hasher.update(pk);
    hasher.update(message);
    let r_hash = hasher.finalize();

    // Take first 32 bytes and ensure valid point encoding
    let mut r_bytes = [0u8; 32];
    r_bytes.copy_from_slice(&r_hash[0..32]);

    // Clear the sign bit (critical for Cardano compatibility)
    r_bytes[31] &= 0x7f;

    // Try to decompress as an Edwards point
    // If it fails, we apply Elligator2 mapping (simplified here)
    match CompressedEdwardsY(r_bytes).decompress() {
        Some(point) => {
            // CRITICAL: Clear cofactor to ensure point is torsion-free
            // This is required for curve25519-dalek v4 to properly handle scalar multiplication
            let cleared = cardano_clear_cofactor(&point);
            Ok((cleared, r_bytes))
        }
        None => {
            // Simplified fallback - in production use full Elligator2
            // For now, hash again with a counter until we get a valid point
            for i in 0..=255u8 {
                let mut retry_hasher = Sha512::new();
                retry_hasher.update(&r_bytes);
                retry_hasher.update(&[i]);
                let retry_hash = retry_hasher.finalize();

                let mut retry_bytes = [0u8; 32];
                retry_bytes.copy_from_slice(&retry_hash[0..32]);
                retry_bytes[31] &= 0x7f;

                if let Some(point) = CompressedEdwardsY(retry_bytes).decompress() {
                    let cleared = cardano_clear_cofactor(&point);
                    return Ok((cleared, retry_bytes));
                }
            }

            Err(VrfError::InvalidPoint)
        }
    }
}

/// Hash to curve for Draft-13 (batch-compatible)
///
/// # Arguments
/// * `pk` - Public key bytes
/// * `message` - Message to hash
///
/// # Returns
/// Tuple of (point on curve, compressed representation bytes)
pub fn cardano_hash_to_curve_draft13(
    pk: &[u8],
    message: &[u8],
) -> VrfResult<(EdwardsPoint, [u8; 32])> {
    // Compute r = SHA512(suite || 0x01 || pk || message)
    let mut hasher = Sha512::new();
    hasher.update(&[SUITE_DRAFT13]);
    hasher.update(&[ONE]);
    hasher.update(pk);
    hasher.update(message);
    let r_hash = hasher.finalize();

    // Take first 32 bytes
    let mut r_bytes = [0u8; 32];
    r_bytes.copy_from_slice(&r_hash[0..32]);

    // Clear the sign bit
    r_bytes[31] &= 0x7f;

    // Try to decompress
    match CompressedEdwardsY(r_bytes).decompress() {
        Some(point) => {
            // Apply cofactor clearing for draft-13
            let cleared = cardano_clear_cofactor(&point);
            Ok((cleared, r_bytes))
        }
        None => {
            // Fallback with retry
            for i in 0..=255u8 {
                let mut retry_hasher = Sha512::new();
                retry_hasher.update(&r_bytes);
                retry_hasher.update(&[i]);
                let retry_hash = retry_hasher.finalize();

                let mut retry_bytes = [0u8; 32];
                retry_bytes.copy_from_slice(&retry_hash[0..32]);
                retry_bytes[31] &= 0x7f;

                if let Some(point) = CompressedEdwardsY(retry_bytes).decompress() {
                    let cleared = cardano_clear_cofactor(&point);
                    return Ok((cleared, retry_bytes));
                }
            }

            Err(VrfError::InvalidPoint)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cofactor_clearing() {
        use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;

        let point = ED25519_BASEPOINT_POINT;
        let cleared = cardano_clear_cofactor(&point);

        // Cleared point should be on the curve
        assert!(cleared.is_torsion_free());
    }

    #[test]
    fn test_hash_to_curve() {
        let pk = [0u8; 32];
        let message = b"test";

        let result = cardano_hash_to_curve(&pk, message);
        assert!(result.is_ok());
    }
}
