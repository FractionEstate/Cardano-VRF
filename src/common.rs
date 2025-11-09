//! Common utilities for VRF implementations

use curve25519_dalek::{edwards::{CompressedEdwardsY, EdwardsPoint}, scalar::Scalar};
use sha2::{Digest, Sha512};

use crate::{VrfError, VrfResult};

/// Suite identifier for Draft-03 (ECVRF-ED25519-SHA512-ELL2)
pub const SUITE_DRAFT03: u8 = 0x04;

/// Suite identifier for Draft-13 (ECVRF-ED25519-SHA512-TAI)
pub const SUITE_DRAFT13: u8 = 0x03;

/// Marker byte 0x01
pub const ONE: u8 = 0x01;

/// Marker byte 0x02
pub const TWO: u8 = 0x02;

/// Marker byte 0x03
pub const THREE: u8 = 0x03;

/// Convert bytes to Edwards point, validating the encoding
///
/// # Errors
///
/// Returns `VrfError::InvalidPoint` if the bytes do not represent a valid Edwards point.
pub fn bytes_to_point(bytes: &[u8; 32]) -> VrfResult<EdwardsPoint> {
    CompressedEdwardsY(*bytes)
        .decompress()
        .ok_or(VrfError::InvalidPoint)
}

/// Convert Edwards point to bytes
#[must_use]
pub fn point_to_bytes(point: &EdwardsPoint) -> [u8; 32] {
    point.compress().to_bytes()
}

/// Convert bytes to scalar
pub fn bytes_to_scalar(bytes: &[u8; 32]) -> Scalar {
    Scalar::from_bytes_mod_order(*bytes)
}

/// Convert scalar to bytes
#[must_use]
pub fn scalar_to_bytes(scalar: &Scalar) -> [u8; 32] {
    scalar.to_bytes()
}

/// Clamp a scalar for Ed25519 use
#[must_use]
pub fn clamp_scalar(mut bytes: [u8; 32]) -> [u8; 32] {
    bytes[0] &= 248;
    bytes[31] &= 127;
    bytes[31] |= 64;
    bytes
}

/// Clear cofactor from a point (multiply by 8)
#[must_use]
pub fn clear_cofactor(point: &EdwardsPoint) -> EdwardsPoint {
    point.mul_by_cofactor()
}

/// Hash data using SHA-512
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
