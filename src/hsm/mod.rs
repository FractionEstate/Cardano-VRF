//! Hardware Security Module (HSM) Integration
//!
//! This module provides a trait-based abstraction for VRF operations
//! that can be backed by various HSM providers including PKCS#11,
//! AWS CloudHSM, Azure Key Vault, and software fallback.

use crate::{VrfError, VrfResult};

pub mod pkcs11;
pub mod aws_cloudhsm;
pub mod azure_keyvault;
pub mod software;

/// VRF signing operations that can be delegated to an HSM
pub trait HsmVrfSigner: Send + Sync {
    /// Generate a VRF proof using the HSM-stored key
    ///
    /// # Arguments
    /// * `key_id` - Identifier for the VRF signing key in the HSM
    /// * `message` - Message to prove
    ///
    /// # Returns
    /// VRF proof bytes
    fn prove(&self, key_id: &str, message: &[u8]) -> VrfResult<Vec<u8>>;

    /// Retrieve the public key corresponding to an HSM-stored private key
    ///
    /// # Arguments
    /// * `key_id` - Identifier for the VRF signing key
    ///
    /// # Returns
    /// 32-byte Ed25519 public key
    fn get_public_key(&self, key_id: &str) -> VrfResult<[u8; 32]>;

    /// Generate a new VRF keypair in the HSM
    ///
    /// # Arguments
    /// * `key_id` - Identifier to assign to the new key
    ///
    /// # Returns
    /// Public key of the newly generated keypair
    fn generate_keypair(&self, key_id: &str) -> VrfResult<[u8; 32]>;

    /// Delete a key from the HSM
    ///
    /// # Security
    /// This is a destructive operation. Ensure proper access controls.
    fn delete_key(&self, key_id: &str) -> VrfResult<()>;

    /// List all VRF keys available in the HSM
    fn list_keys(&self) -> VrfResult<Vec<String>>;

    /// Test HSM connectivity and authentication
    fn health_check(&self) -> VrfResult<()>;
}

/// VRF verification operations (typically don't require HSM)
pub trait HsmVrfVerifier: Send + Sync {
    /// Verify a VRF proof
    ///
    /// # Arguments
    /// * `public_key` - 32-byte Ed25519 public key
    /// * `proof` - VRF proof
    /// * `message` - Message that was proven
    ///
    /// # Returns
    /// 64-byte VRF output
    fn verify(&self, public_key: &[u8; 32], proof: &[u8], message: &[u8]) -> VrfResult<Vec<u8>>;
}

/// HSM provider configuration
#[derive(Debug, Clone)]
pub enum HsmConfig {
    /// PKCS#11 compliant HSM
    Pkcs11 {
        library_path: String,
        slot_id: u64,
        pin: String,
    },
    /// AWS CloudHSM
    AwsCloudHsm {
        cluster_id: String,
        user: String,
        password: String,
    },
    /// Azure Key Vault
    AzureKeyVault {
        vault_url: String,
        client_id: String,
        client_secret: String,
        tenant_id: String,
    },
    /// Software-based implementation (for testing)
    Software {
        key_storage_path: String,
    },
}

/// Factory for creating HSM-backed VRF signers
pub struct HsmFactory;

impl HsmFactory {
    /// Create an HSM VRF signer from configuration
    pub fn create_signer(config: HsmConfig) -> VrfResult<Box<dyn HsmVrfSigner>> {
        match config {
            HsmConfig::Pkcs11 { library_path, slot_id, pin } => {
                pkcs11::Pkcs11VrfSigner::new(library_path, slot_id, pin)
                    .map(|s| Box::new(s) as Box<dyn HsmVrfSigner>)
            }
            HsmConfig::AwsCloudHsm { cluster_id, user, password } => {
                aws_cloudhsm::AwsCloudHsmVrfSigner::new(cluster_id, user, password)
                    .map(|s| Box::new(s) as Box<dyn HsmVrfSigner>)
            }
            HsmConfig::AzureKeyVault { vault_url, client_id, client_secret, tenant_id } => {
                azure_keyvault::AzureKeyVaultVrfSigner::new(vault_url, client_id, client_secret, tenant_id)
                    .map(|s| Box::new(s) as Box<dyn HsmVrfSigner>)
            }
            HsmConfig::Software { key_storage_path } => {
                software::SoftwareVrfSigner::new(key_storage_path)
                    .map(|s| Box::new(s) as Box<dyn HsmVrfSigner>)
            }
        }
    }
}
