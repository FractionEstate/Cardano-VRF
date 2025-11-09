//! Hardware Security Module (HSM) integration for secure VRF operations
//!
//! This module provides a trait-based abstraction layer for delegating VRF
//! cryptographic operations to Hardware Security Modules (HSMs). HSMs provide
//! tamper-resistant key storage and cryptographic acceleration.
//!
//! # Supported Backends
//!
//! - **Software**: In-memory implementation for development and testing
//! - **PKCS#11**: Industry-standard HSM interface (e.g., Thales, Utimaco)
//! - **AWS CloudHSM**: Amazon's cloud-based HSM service
//! - **Azure Key Vault**: Microsoft Azure's managed HSM service
//!
//! # Security Benefits
//!
//! Using an HSM for VRF operations provides:
//! - **Key Protection**: Private keys never leave the HSM
//! - **Audit Logging**: All cryptographic operations are logged
//! - **Access Control**: Fine-grained permissions for key usage
//! - **FIPS Compliance**: Certified cryptographic modules
//!
//! # Architecture
//!
//! The module uses two traits:
//! - [`HsmVrfSigner`]: Operations requiring private keys (prove, sign)
//! - [`HsmVrfVerifier`]: Public operations (verify)
//!
//! This separation allows verification to run without HSM access while
//! keeping signing operations secure.
//!
//! # Examples
//!
//! ## Software HSM (Development)
//!
//! ```rust
//! use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
//!
//! # fn main() -> Result<(), cardano_vrf::VrfError> {
//! let hsm = SoftwareVrfSigner::new("/tmp/vrf-keys".to_string())?;
//!
//! // Generate keypair
//! let public_key = hsm.generate_keypair("my-vrf-key")?;
//!
//! // Generate proof
//! let message = b"Hello, HSM!";
//! let proof = hsm.prove("my-vrf-key", message)?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Production Deployment
//!
//! In production, replace `SoftwareHsm` with a hardware-backed implementation:
//!
//! ```ignore
//! use cardano_vrf::hsm::pkcs11::Pkcs11Hsm;
//!
//! let hsm = Pkcs11Hsm::new("/usr/lib/pkcs11/libhsm.so", "slot0")?;
//! let proof = hsm.prove("production-vrf-key", message)?;
//! ```

#[allow(unused_imports)] // VrfError used in doc links
use crate::{VrfError, VrfResult};

pub mod aws_cloudhsm;
pub mod azure_keyvault;
pub mod pkcs11;
pub mod software;

/// Trait for VRF signing operations delegated to an HSM
///
/// Implementors of this trait handle VRF proof generation using private keys
/// stored securely in an HSM. The private key never leaves the HSM hardware.
///
/// # Thread Safety
///
/// Implementations must be `Send + Sync` to support concurrent proof generation
/// in multi-threaded applications. HSM vendors typically handle internal locking.
///
/// # Security Considerations
///
/// - **Key Isolation**: Private keys must never be exportable
/// - **Audit Logging**: All operations should be logged by the HSM
/// - **Rate Limiting**: Consider implementing request throttling
/// - **Access Control**: Use HSM-native access controls for key_id
pub trait HsmVrfSigner: Send + Sync {
    /// Generate a VRF proof using an HSM-stored private key
    ///
    /// This is the core VRF operation. The HSM uses the stored private key
    /// to generate a proof that the VRF output was computed correctly for
    /// the given message.
    ///
    /// # Arguments
    ///
    /// * `key_id` - Identifier for the VRF signing key in the HSM (e.g., key alias, slot ID)
    /// * `message` - Arbitrary message bytes to prove
    ///
    /// # Returns
    ///
    /// VRF proof bytes (80 bytes for Draft-03, 128 bytes for Draft-13)
    ///
    /// # Errors
    ///
    /// - [`VrfError::InvalidInput`] if key_id is not found or inaccessible
    /// - HSM-specific errors (connection failures, authentication errors)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
    ///
    /// # fn main() -> Result<(), cardano_vrf::VrfError> {
    /// let hsm = SoftwareVrfSigner::new("/tmp/vrf-keys".to_string())?;
    /// hsm.generate_keypair("slot-1")?;
    ///
    /// let proof = hsm.prove("slot-1", b"block-12345")?;
    /// # Ok(())
    /// # }
    /// ```
    fn prove(&self, key_id: &str, message: &[u8]) -> VrfResult<Vec<u8>>;

    /// Retrieve the public key for an HSM-stored private key
    ///
    /// Public keys can be safely exported from the HSM for verification
    /// operations that don't require the private key.
    ///
    /// # Arguments
    ///
    /// * `key_id` - Identifier for the VRF signing key
    ///
    /// # Returns
    ///
    /// 32-byte Ed25519 public key in compressed format
    ///
    /// # Errors
    ///
    /// Returns [`VrfError::InvalidInput`] if key_id is not found
    fn get_public_key(&self, key_id: &str) -> VrfResult<[u8; 32]>;

    /// Generate a new VRF keypair in the HSM
    ///
    /// Creates a new Ed25519 keypair with the private key stored securely
    /// in the HSM. The private key is generated within the HSM and never
    /// exported.
    ///
    /// # Arguments
    ///
    /// * `key_id` - Identifier to assign to the new key
    ///
    /// # Returns
    ///
    /// Public key of the newly generated keypair
    ///
    /// # Errors
    ///
    /// - [`VrfError::InvalidInput`] if key_id already exists
    /// - HSM-specific errors (out of storage, permission denied)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
    ///
    /// # fn main() -> Result<(), cardano_vrf::VrfError> {
    /// let hsm = SoftwareVrfSigner::new("/tmp/vrf-keys".to_string())?;
    /// let public_key = hsm.generate_keypair("validator-001")?;
    /// println!("Public key: {:02x?}", &public_key[0..8]);
    /// # Ok(())
    /// # }
    /// ```
    fn generate_keypair(&self, key_id: &str) -> VrfResult<[u8; 32]>;

    /// Delete a key from the HSM
    ///
    /// **⚠️ WARNING**: This is a destructive operation that permanently
    /// deletes the private key. There is no recovery mechanism.
    ///
    /// # Security
    ///
    /// - Ensure proper access controls prevent unauthorized deletion
    /// - Log all deletion attempts for audit purposes
    /// - Consider implementing a key lifecycle policy with soft-delete
    ///
    /// # Arguments
    ///
    /// * `key_id` - Identifier of the key to delete
    ///
    /// # Errors
    ///
    /// Returns [`VrfError::InvalidInput`] if key_id is not found
    fn delete_key(&self, key_id: &str) -> VrfResult<()>;

    /// List all VRF keys available in the HSM
    ///
    /// Returns identifiers of all VRF keys accessible with current credentials.
    /// Useful for key discovery and management operations.
    ///
    /// # Returns
    ///
    /// Vector of key identifiers (e.g., aliases, labels)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
    ///
    /// # fn main() -> Result<(), cardano_vrf::VrfError> {
    /// let hsm = SoftwareVrfSigner::new("/tmp/vrf-keys".to_string())?;
    /// hsm.generate_keypair("key-1")?;
    /// hsm.generate_keypair("key-2")?;
    ///
    /// let keys = hsm.list_keys()?;
    /// assert!(keys.contains(&"key-1".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    fn list_keys(&self) -> VrfResult<Vec<String>>;

    /// Test HSM connectivity and authentication
    ///
    /// Verifies that the HSM is reachable, credentials are valid, and
    /// basic operations can be performed. Use this for health monitoring.
    ///
    /// # Returns
    ///
    /// `Ok(())` if the HSM is healthy and accessible
    ///
    /// # Errors
    ///
    /// Various errors depending on failure mode (network, authentication, etc.)
    fn health_check(&self) -> VrfResult<()>;
}

/// Trait for VRF verification operations (public-key only)
///
/// Verification doesn't require access to private keys, so implementations
/// typically don't need actual HSM connectivity. This trait exists for
/// API consistency and future extensibility.
pub trait HsmVrfVerifier: Send + Sync {
    /// Verify a VRF proof
    ///
    /// # Arguments
    ///
    /// * `public_key` - 32-byte Ed25519 public key
    /// * `proof` - VRF proof bytes
    /// * `message` - Message that was proven
    ///
    /// # Returns
    ///
    /// 64-byte VRF output if verification succeeds
    ///
    /// # Errors
    ///
    /// Returns [`VrfError::VerificationFailed`] if the proof is invalid
    fn verify(&self, public_key: &[u8; 32], proof: &[u8], message: &[u8]) -> VrfResult<Vec<u8>>;
}

/// HSM provider configuration
#[derive(Debug, Clone)]
pub enum HsmConfig {
    /// PKCS#11 compliant HSM
    Pkcs11 {
        /// Path to PKCS#11 library (e.g., `/usr/lib/libsofthsm2.so`)
        library_path: String,
        /// HSM slot identifier
        slot_id: u64,
        /// PIN for HSM access
        pin: String,
    },
    /// AWS CloudHSM
    AwsCloudHsm {
        /// AWS CloudHSM cluster identifier
        cluster_id: String,
        /// HSM user name
        user: String,
        /// HSM user password
        password: String,
    },
    /// Azure Key Vault
    AzureKeyVault {
        /// Azure Key Vault URL (e.g., `https://myvault.vault.azure.net`)
        vault_url: String,
        /// Azure AD application (client) ID
        client_id: String,
        /// Azure AD application client secret
        client_secret: String,
        /// Azure AD tenant ID
        tenant_id: String,
    },
    /// Software-based implementation (for testing)
    Software {
        /// Path to key storage directory
        key_storage_path: String,
    },
}

/// Factory for creating HSM-backed VRF signers
pub struct HsmFactory;

impl HsmFactory {
    /// Create an HSM VRF signer from configuration
    pub fn create_signer(config: HsmConfig) -> VrfResult<Box<dyn HsmVrfSigner>> {
        match config {
            HsmConfig::Pkcs11 {
                library_path,
                slot_id,
                pin,
            } => pkcs11::Pkcs11VrfSigner::new(library_path, slot_id, pin)
                .map(|s| Box::new(s) as Box<dyn HsmVrfSigner>),
            HsmConfig::AwsCloudHsm {
                cluster_id,
                user,
                password,
            } => aws_cloudhsm::AwsCloudHsmVrfSigner::new(cluster_id, user, password)
                .map(|s| Box::new(s) as Box<dyn HsmVrfSigner>),
            HsmConfig::AzureKeyVault {
                vault_url,
                client_id,
                client_secret,
                tenant_id,
            } => azure_keyvault::AzureKeyVaultVrfSigner::new(
                vault_url,
                client_id,
                client_secret,
                tenant_id,
            )
            .map(|s| Box::new(s) as Box<dyn HsmVrfSigner>),
            HsmConfig::Software { key_storage_path } => {
                software::SoftwareVrfSigner::new(key_storage_path)
                    .map(|s| Box::new(s) as Box<dyn HsmVrfSigner>)
            }
        }
    }
}
