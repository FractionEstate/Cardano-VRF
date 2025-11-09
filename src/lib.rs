//! Pure Rust implementation of VRF (Verifiable Random Function) for Cardano
//!
//! This crate provides a 100% Rust implementation of the VRF algorithms used in Cardano,
//! following the IETF specifications:
//! - ECVRF-ED25519-SHA512-Elligator2 (draft-03)
//! - ECVRF-ED25519-SHA512-TAI (draft-13 batch-compatible)
//!
//! All implementations are memory-safe and use constant-time operations where appropriate
//! to prevent timing attacks.
//!
//! # Production Features
//!
//! - **HSM Integration**: PKCS#11, AWS CloudHSM, Azure Key Vault support
//! - **Metrics**: Prometheus-compatible metrics for monitoring
//! - **Logging**: Structured audit logging for compliance
//! - **Cryptographic Parity**: 100% byte-for-byte compatibility with Cardano libsodium
//!
//! # Examples
//!
//! ## Using Draft-03 VRF
//!
//! ```rust
//! use cardano_vrf::{VrfDraft03, VrfError};
//!
//! # fn main() -> Result<(), VrfError> {
//! let seed = [0u8; 32];
//! let (secret_key, public_key) = VrfDraft03::keypair_from_seed(&seed);
//!
//! let message = b"Hello, Cardano!";
//! let proof = VrfDraft03::prove(&secret_key, message)?;
//! let output = VrfDraft03::verify(&public_key, &proof, message)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Using Draft-13 VRF (Batch-Compatible)
//!
//! ```rust
//! use cardano_vrf::{VrfDraft13, VrfError};
//!
//! # fn main() -> Result<(), VrfError> {
//! let seed = [0u8; 32];
//! let (secret_key, public_key) = VrfDraft13::keypair_from_seed(&seed);
//!
//! let message = b"Batch compatible VRF";
//! let proof = VrfDraft13::prove(&secret_key, message)?;
//! let output = VrfDraft13::verify(&public_key, &proof, message)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Debug logging
//!
//! The crate ships completely silent cryptographic primitives by default. When
//! troubleshooting interoperability issues you can enable feature `vrf-debug`
//! and set the environment variable `CARDANO_VRF_DEBUG=1` to surface detailed
//! diagnostics from the Cardano compatibility layer (including Elligator and
//! hash-to-curve internals). Both the feature flag and environment variable are
//! required to emit logs so production builds remain noise-free.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
#![cfg_attr(test, allow(clippy::unwrap_used))]
#![deny(unsafe_code)]
#![warn(missing_docs)]

#[cfg(not(feature = "std"))]
extern crate alloc;

pub mod cardano_compat;
pub mod common;
pub mod draft03;
pub mod draft13;
pub mod hsm;
pub mod logging;
pub mod metrics;

pub use draft03::VrfDraft03;
pub use draft13::VrfDraft13;
pub use hsm::{HsmConfig, HsmFactory, HsmVrfSigner, HsmVrfVerifier};
pub use logging::{LogEntry, LogLevel, VrfLogger, VrfOperation};
pub use metrics::{MetricsTimer, VrfMetrics};

/// Error types returned by VRF operations
///
/// These errors represent the various failure modes that can occur during
/// VRF proof generation, verification, and related cryptographic operations.
///
/// # Security Considerations
///
/// Error types are deliberately generic to avoid leaking information about
/// why a particular operation failed. Detailed error information is only
/// available via debug logging when the `vrf-debug` feature is enabled.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum VrfError {
    /// The provided VRF proof is malformed or invalid
    ///
    /// This can occur if:
    /// - The proof bytes cannot be parsed (wrong length, invalid encoding)
    /// - The proof components (Gamma, c, s) are not valid curve points/scalars
    /// - The proof was corrupted or tampered with
    #[error("Invalid VRF proof")]
    InvalidProof,

    /// The public key is malformed or not a valid Ed25519 point
    ///
    /// This occurs when the 32-byte public key cannot be decompressed to
    /// a valid point on the Edwards curve, or represents a point with
    /// invalid properties (e.g., not in the correct subgroup).
    #[error("Invalid public key")]
    InvalidPublicKey,

    /// The secret key is malformed or has an invalid format
    ///
    /// For VRF operations, secret keys are 64 bytes (32-byte seed + 32-byte public key).
    /// This error indicates the key doesn't meet these requirements.
    #[error("Invalid secret key")]
    InvalidSecretKey,

    /// Failed to decode bytes as an Edwards curve point
    ///
    /// This occurs during hash-to-curve or proof parsing when bytes
    /// cannot be decompressed to a valid Edwards point.
    #[error("Invalid point encoding")]
    InvalidPoint,

    /// Failed to decode bytes as a valid scalar value
    ///
    /// Scalars must be in the range [0, L) where L is the order of
    /// the Ed25519 group. This error indicates invalid encoding.
    #[error("Invalid scalar encoding")]
    InvalidScalar,

    /// VRF proof verification failed
    ///
    /// The proof is well-formed but the verification equation does not hold.
    /// This means either:
    /// - The proof was not generated with the claimed public key
    /// - The proof was generated for a different message
    /// - The proof has been tampered with
    ///
    /// This is the expected error for invalid/forged proofs.
    #[error("VRF verification failed")]
    VerificationFailed,

    /// Generic invalid input error with descriptive message
    ///
    /// Used for various input validation failures such as incorrect
    /// buffer sizes, invalid parameters, or HSM-specific errors.
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Result type alias for VRF operations
///
/// All VRF functions return this type, which is either a successful
/// value of type `T` or a [`VrfError`].
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::{VrfResult, VrfError};
///
/// fn validate_proof() -> VrfResult<bool> {
///     // ... validation logic
///     Ok(true)
/// }
/// ```
pub type VrfResult<T> = Result<T, VrfError>;
