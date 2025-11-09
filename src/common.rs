//! Common cryptographic utilities shared across VRF implementations
//!
//! This module provides low-level cryptographic primitives used by both
//! Draft-03 and Draft-13 VRF implementations, including:
//! - Point and scalar conversions
//! - Ed25519 scalar clamping
//! - Cofactor clearing for curve points
//! - SHA-512 hashing utilities
//!
//! # Suite Identifiers
//!
//! VRF protocols use suite identifiers to domain-separate different variants:
//! - Draft-03: `0x04` (ECVRF-ED25519-SHA512-ELL2)
//! - Draft-13: `0x03` (ECVRF-ED25519-SHA512-TAI)

use curve25519_dalek::{edwards::{CompressedEdwardsY, EdwardsPoint}, scalar::Scalar};
use sha2::{Digest, Sha512};

use crate::{VrfError, VrfResult};

/// Suite identifier for Draft-03 VRF (ECVRF-ED25519-SHA512-ELL2)
///
/// This identifies the Elligator2-based hash-to-curve variant used in
/// Cardano's implementation following IETF draft-irtf-cfrg-vrf-03.
pub const SUITE_DRAFT03: u8 = 0x04;

/// Suite identifier for Draft-13 VRF (ECVRF-ED25519-SHA512-TAI)
///
/// This identifies the Try-And-Increment hash-to-curve variant that
/// supports batch verification, following IETF draft-irtf-cfrg-vrf-13.
pub const SUITE_DRAFT13: u8 = 0x03;

/// Domain separation byte: Used in hash-to-curve operations
///
/// Prepended to inputs for the initial hash-to-curve step to prevent
/// cross-protocol attacks.
pub const ONE: u8 = 0x01;

/// Domain separation byte: Used in challenge generation
///
/// Prepended to inputs when computing the Fiat-Shamir challenge scalar
/// during proof generation and verification.
pub const TWO: u8 = 0x02;

/// Domain separation byte: Used in VRF output computation
///
/// Prepended to inputs when deriving the final VRF output hash from the
/// Gamma point.
pub const THREE: u8 = 0x03;

/// Decompress bytes into an Edwards curve point
///
/// Attempts to decompress a 32-byte compressed Edwards Y coordinate
/// into a full Edwards point on the Ed25519 curve.
///
/// # Arguments
///
/// * `bytes` - Compressed Edwards Y coordinate (32 bytes)
///
/// # Returns
///
/// The decompressed Edwards point if valid
///
/// # Errors
///
/// Returns [`VrfError::InvalidPoint`] if:
/// - The bytes don't represent a valid curve point
/// - The Y coordinate is out of range
/// - The X coordinate cannot be recovered
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::common::bytes_to_point;
/// use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
///
/// let compressed = ED25519_BASEPOINT_POINT.compress().to_bytes();
/// let point = bytes_to_point(&compressed).unwrap();
/// assert_eq!(point, ED25519_BASEPOINT_POINT);
/// ```
pub fn bytes_to_point(bytes: &[u8; 32]) -> VrfResult<EdwardsPoint> {
    CompressedEdwardsY(*bytes)
        .decompress()
        .ok_or(VrfError::InvalidPoint)
}

/// Compress an Edwards curve point to bytes
///
/// Converts an Edwards point to its compressed Y coordinate representation.
/// This is the canonical encoding used in Ed25519 signatures and VRF proofs.
///
/// # Arguments
///
/// * `point` - The Edwards point to compress
///
/// # Returns
///
/// 32-byte compressed Edwards Y coordinate
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::common::point_to_bytes;
/// use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
///
/// let bytes = point_to_bytes(&ED25519_BASEPOINT_POINT);
/// assert_eq!(bytes.len(), 32);
/// ```
#[must_use]
pub fn point_to_bytes(point: &EdwardsPoint) -> [u8; 32] {
    point.compress().to_bytes()
}

/// Decode bytes as a scalar modulo the group order
///
/// Interprets 32 bytes as a scalar value, automatically reducing
/// modulo L (the order of the Ed25519 group). This is a constant-time
/// operation that always succeeds.
///
/// # Arguments
///
/// * `bytes` - 32-byte scalar representation (any value accepted)
///
/// # Returns
///
/// Scalar value reduced modulo L
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::common::bytes_to_scalar;
///
/// let bytes = [0xff; 32]; // Will be reduced mod L
/// let scalar = bytes_to_scalar(&bytes);
/// ```
pub fn bytes_to_scalar(bytes: &[u8; 32]) -> Scalar {
    Scalar::from_bytes_mod_order(*bytes)
}

/// Encode a scalar as 32 bytes
///
/// Converts a scalar to its canonical little-endian byte representation.
///
/// # Arguments
///
/// * `scalar` - The scalar to encode
///
/// # Returns
///
/// 32-byte little-endian scalar encoding
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::common::{bytes_to_scalar, scalar_to_bytes};
///
/// let original = [42u8; 32];
/// let scalar = bytes_to_scalar(&original);
/// let encoded = scalar_to_bytes(&scalar);
/// ```
#[must_use]
pub fn scalar_to_bytes(scalar: &Scalar) -> [u8; 32] {
    scalar.to_bytes()
}

/// Clamp scalar bytes for Ed25519 compatibility
///
/// Applies the standard Ed25519 scalar clamping operation:
/// - Clear the 3 least significant bits (ensure multiple of 8)
/// - Clear the most significant bit (ensure < 2^255)
/// - Set the second-most significant bit (ensure >= 2^254)
///
/// This ensures the scalar is in the range [2^254, 2^255) and is a
/// multiple of 8, which is required for Ed25519 secret keys.
///
/// # Arguments
///
/// * `bytes` - 32-byte scalar to clamp
///
/// # Returns
///
/// Clamped 32-byte scalar
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::common::clamp_scalar;
///
/// let unclamped = [0xff; 32];
/// let clamped = clamp_scalar(unclamped);
///
/// assert_eq!(clamped[0] & 0x07, 0);     // Low 3 bits clear
/// assert_eq!(clamped[31] & 0x80, 0);    // High bit clear
/// assert_eq!(clamped[31] & 0x40, 0x40); // Second-high bit set
/// ```
#[must_use]
pub fn clamp_scalar(mut bytes: [u8; 32]) -> [u8; 32] {
    bytes[0] &= 248;   // Clear bottom 3 bits
    bytes[31] &= 127;  // Clear top bit
    bytes[31] |= 64;   // Set second-top bit
    bytes
}

/// Clear the cofactor from an Edwards curve point
///
/// Multiplies the point by 8 (the cofactor of Ed25519) to ensure it's
/// in the prime-order subgroup. This is critical for security in VRF
/// operations as it prevents small-subgroup attacks.
///
/// # Background
///
/// Ed25519 has order 8 × L where L is prime. Points can exist in small
/// subgroups of order 1, 2, 4, or 8. Multiplying by 8 moves any point
/// into the prime-order subgroup of order L.
///
/// # Arguments
///
/// * `point` - The Edwards point to clear
///
/// # Returns
///
/// Point in the prime-order subgroup (torsion-free)
///
/// # Security
///
/// In curve25519-dalek v4, points with torsion can cause distributive
/// property failures in scalar multiplication. Always clear the cofactor
/// from hash-to-curve outputs before use in VRF operations.
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::common::clear_cofactor;
/// use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;
///
/// let cleared = clear_cofactor(&ED25519_BASEPOINT_POINT);
/// // cleared is guaranteed to be in the prime-order subgroup
/// ```
#[must_use]
pub fn clear_cofactor(point: &EdwardsPoint) -> EdwardsPoint {
    point.mul_by_cofactor()
}

/// Compute SHA-512 hash of input data
///
/// Provides a convenient wrapper around SHA-512 hashing with a
/// fixed-size output array.
///
/// # Arguments
///
/// * `data` - Input bytes to hash
///
/// # Returns
///
/// 64-byte SHA-512 digest
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::common::hash_sha512;
///
/// let data = b"Hello, Cardano!";
/// let hash = hash_sha512(data);
/// assert_eq!(hash.len(), 64);
/// ```
#[must_use]
pub fn hash_sha512(data: &[u8]) -> [u8; 64] {
    let hash = Sha512::digest(data);
    let mut result = [0u8; 64];
    result.copy_from_slice(&hash);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clamp_scalar() {
        let bytes = [0xffu8; 32];
        let clamped = clamp_scalar(bytes);

        assert_eq!(clamped[0] & 0x07, 0);
        assert_eq!(clamped[31] & 0x80, 0);
        assert_eq!(clamped[31] & 0x40, 0x40);
    }

    #[test]
    fn test_point_roundtrip() {
        use curve25519_dalek::constants::ED25519_BASEPOINT_POINT;

        let point = ED25519_BASEPOINT_POINT;
        let bytes = point_to_bytes(&point);
        let recovered = bytes_to_point(&bytes).unwrap();

        assert_eq!(point, recovered);
    }
}
