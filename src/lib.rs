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
pub mod metrics;
pub mod logging;

pub use draft03::VrfDraft03;
pub use draft13::VrfDraft13;
pub use hsm::{HsmConfig, HsmFactory, HsmVrfSigner, HsmVrfVerifier};
pub use metrics::VrfMetrics;
pub use logging::{VrfLogger, LogLevel, VrfOperation};

/// Error types for VRF operations
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum VrfError {
    /// Invalid proof provided
    #[error("Invalid VRF proof")]
    InvalidProof,

    /// Invalid public key
    #[error("Invalid public key")]
    InvalidPublicKey,

    /// Invalid secret key
    #[error("Invalid secret key")]
    InvalidSecretKey,

    /// Invalid point encoding
    #[error("Invalid point encoding")]
    InvalidPoint,

    /// Invalid scalar encoding
    #[error("Invalid scalar encoding")]
    InvalidScalar,

    /// Verification failed
    #[error("VRF verification failed")]
    VerificationFailed,

    /// Invalid input parameter
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Result type for VRF operations
pub type VrfResult<T> = Result<T, VrfError>;
