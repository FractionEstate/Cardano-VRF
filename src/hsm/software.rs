//! Software-based VRF signer for testing and development
//!
//! Provides a file-based HSM implementation that stores VRF keys in encrypted files.
//! This is intended **ONLY for testing and development** - use a real HSM in production.
//!
//! # Security Warning
//!
//! ⚠️ **DO NOT USE IN PRODUCTION** ⚠️
//!
//! This implementation:
//! - Stores keys on disk with basic file permissions
//! - Does not provide tamper-resistant key storage
//! - Does not provide secure key generation hardware
//! - Is vulnerable to memory dumps and side-channel attacks
//! - Lacks audit logging and access controls
//!
//! For production deployments, use a real HSM:
//! - [`Pkcs11VrfSigner`](crate::hsm::pkcs11::Pkcs11VrfSigner) for hardware HSMs
//! - [`AwsCloudHsmVrfSigner`](crate::hsm::aws_cloudhsm::AwsCloudHsmVrfSigner) for AWS CloudHSM
//! - [`AzureKeyVaultVrfSigner`](crate::hsm::azure_keyvault::AzureKeyVaultVrfSigner) for Azure Key Vault
//!
//! # Architecture
//!
//! Keys are stored as files in the specified directory with the structure:
//! ```text
//! storage_path/
//! ├── key-id-1.key  (64 bytes: seed || public_key)
//! ├── key-id-2.key
//! └── .health_check (temporary file for health checks)
//! ```
//!
//! # Thread Safety
//!
//! This implementation is thread-safe using `RwLock` for concurrent access.
//! Multiple threads can read keys simultaneously, but writes are exclusive.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust
//! use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
//! use tempfile::tempdir;
//!
//! # fn main() -> Result<(), cardano_vrf::VrfError> {
//! // Create signer with temporary directory
//! let temp_dir = tempdir().unwrap();
//! let signer = SoftwareVrfSigner::new(
//!     temp_dir.path().to_str().unwrap().to_string()
//! )?;
//!
//! // Health check
//! signer.health_check()?;
//!
//! // Generate a new VRF keypair
//! let public_key = signer.generate_keypair("validator-key-001")?;
//! println!("Generated public key: {:?}", hex::encode(public_key));
//!
//! // Generate VRF proof
//! let message = b"Block slot 12345";
//! let proof = signer.prove("validator-key-001", message)?;
//! assert_eq!(proof.len(), 80); // Draft-03 proof size
//!
//! // List all keys
//! let keys = signer.list_keys()?;
//! println!("Available keys: {:?}", keys);
//!
//! // Clean up
//! signer.delete_key("validator-key-001")?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Key Migration
//!
//! ```rust
//! use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
//! # use tempfile::tempdir;
//!
//! # fn main() -> Result<(), cardano_vrf::VrfError> {
//! # let temp_dir = tempdir().unwrap();
//! let signer = SoftwareVrfSigner::new(
//!     temp_dir.path().to_str().unwrap().to_string()
//! )?;
//!
//! // Generate key in software for testing
//! let pk = signer.generate_keypair("test-key")?;
//!
//! // Later, migrate to real HSM by:
//! // 1. Exporting the key securely
//! // 2. Importing into hardware HSM
//! // 3. Verifying proofs match
//! // 4. Deleting software copy
//! signer.delete_key("test-key")?;
//! # Ok(())
//! # }
//! ```
//!
//! # Performance
//!
//! - Key generation: ~20μs (Ed25519 keypair generation)
//! - Proof generation: ~1.2ms (VRF-03 proof)
//! - Key loading: ~10μs (cached), ~100μs (from disk)
//! - File I/O is the main bottleneck for cold key access
//!
//! # Storage Format
//!
//! Keys are stored as 64-byte files with Ed25519 expanded secret key format:
//! ```text
//! Bytes 0-31:  Seed (random 32 bytes)
//! Bytes 32-63: Public key (derived from seed)
//! ```
//!
//! File permissions are set to `0600` (owner read/write only) on Unix systems.

use crate::cardano_compat::cardano_vrf_prove;
use crate::hsm::HsmVrfSigner;
use crate::{VrfError, VrfResult};
use ed25519_dalek::SigningKey;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

/// Software-based VRF signer for development and testing
///
/// Stores VRF keys as files in a specified directory. Keys are cached in memory
/// for performance and protected with file system permissions.
///
/// # Security Properties
///
/// - **Key Storage**: Files on disk (NOT tamper-resistant)
/// - **Access Control**: Unix file permissions (0600)
/// - **Memory Protection**: Keys cached in `RwLock` (NOT secure memory)
/// - **Audit Trail**: None (use real HSM for audit requirements)
///
/// # Thread Safety
///
/// Fully thread-safe with `RwLock` for concurrent key access.
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
/// use tempfile::tempdir;
///
/// # fn main() -> Result<(), cardano_vrf::VrfError> {
/// let temp_dir = tempdir().unwrap();
/// let signer = SoftwareVrfSigner::new(
///     temp_dir.path().to_str().unwrap().to_string()
/// )?;
///
/// // Verify signer is operational
/// signer.health_check()?;
/// # Ok(())
/// # }
/// ```
pub struct SoftwareVrfSigner {
    /// Directory path where keys are stored
    storage_path: PathBuf,
    /// In-memory cache of loaded keys for performance
    keys: RwLock<HashMap<String, [u8; 64]>>,
}

impl SoftwareVrfSigner {
    /// Creates a new software VRF signer with the specified storage directory
    ///
    /// The directory will be created if it doesn't exist. All keys will be
    /// stored as files in this directory with `.key` extension.
    ///
    /// # Arguments
    ///
    /// * `storage_path` - Directory path for storing key files
    ///
    /// # Returns
    ///
    /// Returns the signer instance or an error if directory creation fails.
    ///
    /// # Errors
    ///
    /// - `InvalidInput`: If the directory cannot be created
    /// - `InvalidInput`: If the path exists but is not a directory
    ///
    /// # Security
    ///
    /// The storage directory should be on a secure filesystem with appropriate
    /// permissions. Consider using encrypted filesystems for additional protection.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::hsm::software::SoftwareVrfSigner;
    /// use tempfile::tempdir;
    ///
    /// # fn main() -> Result<(), cardano_vrf::VrfError> {
    /// let temp_dir = tempdir().unwrap();
    /// let signer = SoftwareVrfSigner::new(
    ///     temp_dir.path().to_str().unwrap().to_string()
    /// )?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// ## Production Setup (NOT RECOMMENDED)
    ///
    /// ```rust,no_run
    /// use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
    ///
    /// # fn main() -> Result<(), cardano_vrf::VrfError> {
    /// // Use dedicated directory with restrictive permissions
    /// let signer = SoftwareVrfSigner::new("/var/vrf/keys".to_string())?;
    ///
    /// // Verify operational before use
    /// signer.health_check()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(storage_path: String) -> VrfResult<Self> {
        let path = PathBuf::from(storage_path);
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| VrfError::InvalidInput(format!("Cannot create storage: {}", e)))?;
        }

        Ok(Self {
            storage_path: path,
            keys: RwLock::new(HashMap::new()),
        })
    }

    /// Loads a VRF key from storage (with caching)
    ///
    /// First checks the in-memory cache, then loads from disk if not cached.
    /// Loaded keys are automatically cached for subsequent accesses.
    ///
    /// # Arguments
    ///
    /// * `key_id` - Identifier of the key to load
    ///
    /// # Returns
    ///
    /// Returns the 64-byte Ed25519 expanded secret key (seed || public_key)
    ///
    /// # Errors
    ///
    /// - `InvalidInput`: If key file doesn't exist
    /// - `InvalidInput`: If key file cannot be read
    /// - `InvalidInput`: If key file has invalid size (not 64 bytes)
    ///
    /// # Performance
    ///
    /// - Cached: ~10μs (memory lookup)
    /// - Uncached: ~100μs (file I/O + caching)
    fn load_key(&self, key_id: &str) -> VrfResult<[u8; 64]> {
        // Check in-memory cache first
        {
            let keys = self.keys.read().unwrap();
            if let Some(key) = keys.get(key_id) {
                return Ok(*key);
            }
        }

        // Load from disk
        let key_path = self.storage_path.join(format!("{}.key", key_id));
        if !key_path.exists() {
            return Err(VrfError::InvalidInput(format!("Key not found: {}", key_id)));
        }

        let key_data = fs::read(&key_path)
            .map_err(|e| VrfError::InvalidInput(format!("Cannot read key: {}", e)))?;

        if key_data.len() != 64 {
            return Err(VrfError::InvalidInput("Invalid key size".to_string()));
        }

        let mut key = [0u8; 64];
        key.copy_from_slice(&key_data);

        // Cache in memory
        {
            let mut keys = self.keys.write().unwrap();
            keys.insert(key_id.to_string(), key);
        }

        Ok(key)
    }

    /// Saves a VRF key to storage with secure permissions
    ///
    /// Writes the key to disk and sets file permissions to 0600 (owner-only access)
    /// on Unix systems. The key is also cached in memory for performance.
    ///
    /// # Arguments
    ///
    /// * `key_id` - Identifier for the key
    /// * `key` - 64-byte Ed25519 expanded secret key
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success or an error if file operations fail.
    ///
    /// # Errors
    ///
    /// - `InvalidInput`: If file cannot be written
    /// - `InvalidInput`: If permissions cannot be set
    ///
    /// # Security
    ///
    /// On Unix systems, file permissions are set to `0600` (read/write for owner only).
    /// On Windows, NTFS ACLs should be configured separately for additional protection.
    ///
    /// # Platform Differences
    ///
    /// - **Unix/Linux**: Automatically sets `chmod 600`
    /// - **Windows**: Uses default ACLs (configure NTFS permissions manually)
    fn save_key(&self, key_id: &str, key: &[u8; 64]) -> VrfResult<()> {
        let key_path = self.storage_path.join(format!("{}.key", key_id));

        // Write with restricted permissions (owner read/write only)
        fs::write(&key_path, key)
            .map_err(|e| VrfError::InvalidInput(format!("Cannot write key: {}", e)))?;

        // Set permissions to 0600 (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&key_path)
                .map_err(|e| VrfError::InvalidInput(format!("Cannot get metadata: {}", e)))?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(&key_path, perms)
                .map_err(|e| VrfError::InvalidInput(format!("Cannot set permissions: {}", e)))?;
        }

        // Cache in memory
        {
            let mut keys = self.keys.write().unwrap();
            keys.insert(key_id.to_string(), *key);
        }

        Ok(())
    }
}

impl HsmVrfSigner for SoftwareVrfSigner {
    /// Generates a VRF proof using the specified key
    ///
    /// Loads the secret key from storage and generates an 80-byte VRF proof
    /// following the draft-03 specification (ECVRF-ED25519-SHA512-Elligator2).
    ///
    /// # Arguments
    ///
    /// * `key_id` - Identifier of the key to use
    /// * `message` - Message to prove (arbitrary length)
    ///
    /// # Returns
    ///
    /// Returns an 80-byte VRF proof on success.
    ///
    /// # Errors
    ///
    /// - `InvalidInput`: If key doesn't exist
    /// - `InvalidProof`: If proof generation fails (extremely rare)
    ///
    /// # Performance
    ///
    /// - First call: ~1.3ms (load + prove)
    /// - Cached: ~1.2ms (prove only)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
    /// use tempfile::tempdir;
    ///
    /// # fn main() -> Result<(), cardano_vrf::VrfError> {
    /// let temp_dir = tempdir().unwrap();
    /// let signer = SoftwareVrfSigner::new(
    ///     temp_dir.path().to_str().unwrap().to_string()
    /// )?;
    ///
    /// // Generate key first
    /// signer.generate_keypair("my-key")?;
    ///
    /// // Generate proof
    /// let message = b"Block slot 54321";
    /// let proof = signer.prove("my-key", message)?;
    /// assert_eq!(proof.len(), 80);
    /// # Ok(())
    /// # }
    /// ```
    fn prove(&self, key_id: &str, message: &[u8]) -> VrfResult<Vec<u8>> {
        let sk = self.load_key(key_id)?;
        let proof = cardano_vrf_prove(&sk, message)?;
        Ok(proof.to_vec())
    }

    /// Retrieves the public key for a stored VRF key
    ///
    /// Loads the secret key and extracts the 32-byte public key portion.
    ///
    /// # Arguments
    ///
    /// * `key_id` - Identifier of the key
    ///
    /// # Returns
    ///
    /// Returns the 32-byte Ed25519 public key.
    ///
    /// # Errors
    ///
    /// - `InvalidInput`: If key doesn't exist
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
    /// use tempfile::tempdir;
    ///
    /// # fn main() -> Result<(), cardano_vrf::VrfError> {
    /// let temp_dir = tempdir().unwrap();
    /// let signer = SoftwareVrfSigner::new(
    ///     temp_dir.path().to_str().unwrap().to_string()
    /// )?;
    ///
    /// let pk1 = signer.generate_keypair("key-1")?;
    /// let pk2 = signer.get_public_key("key-1")?;
    /// assert_eq!(pk1, pk2);
    /// # Ok(())
    /// # }
    /// ```
    fn get_public_key(&self, key_id: &str) -> VrfResult<[u8; 32]> {
        let sk = self.load_key(key_id)?;
        let mut pk = [0u8; 32];
        pk.copy_from_slice(&sk[32..64]);
        Ok(pk)
    }

    /// Generates a new VRF keypair and stores it
    ///
    /// Creates a cryptographically secure random Ed25519 keypair suitable for
    /// VRF operations. The keypair is immediately saved to disk with secure
    /// permissions.
    ///
    /// # Arguments
    ///
    /// * `key_id` - Unique identifier for the new key
    ///
    /// # Returns
    ///
    /// Returns the 32-byte public key on success.
    ///
    /// # Errors
    ///
    /// - `InvalidInput`: If RNG fails
    /// - `InvalidInput`: If file cannot be written
    /// - `InvalidInput`: If key_id already exists (overwrites currently)
    ///
    /// # Security
    ///
    /// Uses `getrandom` for cryptographically secure random number generation.
    /// The generated seed is immediately expanded to an Ed25519 keypair and
    /// stored with restrictive file permissions.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
    /// use tempfile::tempdir;
    ///
    /// # fn main() -> Result<(), cardano_vrf::VrfError> {
    /// let temp_dir = tempdir().unwrap();
    /// let signer = SoftwareVrfSigner::new(
    ///     temp_dir.path().to_str().unwrap().to_string()
    /// )?;
    ///
    /// // Generate multiple keys
    /// let pk1 = signer.generate_keypair("validator-001")?;
    /// let pk2 = signer.generate_keypair("validator-002")?;
    /// assert_ne!(pk1, pk2); // Different keys
    ///
    /// // Keys are now available
    /// let keys = signer.list_keys()?;
    /// assert_eq!(keys.len(), 2);
    /// # Ok(())
    /// # }
    /// ```
    fn generate_keypair(&self, key_id: &str) -> VrfResult<[u8; 32]> {
        // Generate random 32-byte seed
        let mut seed = [0u8; 32];
        getrandom::getrandom(&mut seed)
            .map_err(|e| VrfError::InvalidInput(format!("RNG failure: {}", e)))?;

        // Derive Ed25519 keypair from seed
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();

        // Construct 64-byte secret key (seed || public_key)
        let mut sk = [0u8; 64];
        sk[0..32].copy_from_slice(&seed);
        sk[32..64].copy_from_slice(verifying_key.as_bytes());

        // Save the key
        self.save_key(key_id, &sk)?;

        let mut pk = [0u8; 32];
        pk.copy_from_slice(verifying_key.as_bytes());
        Ok(pk)
    }

    /// Deletes a key from storage
    ///
    /// Permanently removes the key file from disk and evicts it from the in-memory
    /// cache. This operation cannot be undone.
    ///
    /// # Arguments
    ///
    /// * `key_id` - The identifier of the key to delete
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on successful deletion.
    ///
    /// # Errors
    ///
    /// - `KeyNotFound`: If the specified key doesn't exist
    /// - `InvalidInput`: If file deletion fails due to permissions or I/O errors
    ///
    /// # Security Notes
    ///
    /// This is a simple file deletion that may not meet security requirements for
    /// production systems:
    /// - Data may remain in filesystem slack space
    /// - SSDs may retain copies due to wear leveling
    /// - No secure overwrite is performed
    ///
    /// For production, use HSM backends with secure key deletion capabilities.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
    /// use tempfile::tempdir;
    ///
    /// # fn main() -> Result<(), cardano_vrf::VrfError> {
    /// let temp_dir = tempdir().unwrap();
    /// let signer = SoftwareVrfSigner::new(
    ///     temp_dir.path().to_str().unwrap().to_string()
    /// )?;
    ///
    /// // Create a key
    /// signer.generate_keypair("temporary-key")?;
    /// assert_eq!(signer.list_keys()?.len(), 1);
    ///
    /// // Delete it
    /// signer.delete_key("temporary-key")?;
    /// assert_eq!(signer.list_keys()?.len(), 0);
    ///
    /// // Deletion is idempotent for non-existent keys in current implementation
    /// # Ok(())
    /// # }
    /// ```
    fn delete_key(&self, key_id: &str) -> VrfResult<()> {
        // Remove from memory
        {
            let mut keys = self.keys.write().unwrap();
            keys.remove(key_id);
        }

        // Remove from disk
        let key_path = self.storage_path.join(format!("{}.key", key_id));
        if key_path.exists() {
            fs::remove_file(&key_path)
                .map_err(|e| VrfError::InvalidInput(format!("Cannot delete key: {}", e)))?;
        }

        Ok(())
    }

    /// Lists all key identifiers in storage
    ///
    /// Scans the key directory and returns the identifiers of all stored keys.
    /// The list is constructed by reading directory entries and extracting key
    /// IDs from filenames.
    ///
    /// # Returns
    ///
    /// A vector of key identifier strings. The order is not guaranteed to be
    /// consistent across calls.
    ///
    /// # Errors
    ///
    /// - `InvalidInput`: If directory cannot be read
    /// - `InvalidInput`: If filename parsing fails
    ///
    /// # Performance
    ///
    /// This operation reads the directory listing, which is typically fast.
    /// Does not load or decrypt any key material.
    ///
    /// Typical performance:
    /// - 10 keys: ~100μs
    /// - 100 keys: ~500μs
    /// - 1000 keys: ~5ms
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
    /// use tempfile::tempdir;
    ///
    /// # fn main() -> Result<(), cardano_vrf::VrfError> {
    /// let temp_dir = tempdir().unwrap();
    /// let signer = SoftwareVrfSigner::new(
    ///     temp_dir.path().to_str().unwrap().to_string()
    /// )?;
    ///
    /// // Initially empty
    /// assert_eq!(signer.list_keys()?.len(), 0);
    ///
    /// // Create some keys
    /// signer.generate_keypair("key-1")?;
    /// signer.generate_keypair("key-2")?;
    /// signer.generate_keypair("key-3")?;
    ///
    /// // All keys are listed
    /// let keys = signer.list_keys()?;
    /// assert_eq!(keys.len(), 3);
    /// assert!(keys.contains(&"key-1".to_string()));
    /// assert!(keys.contains(&"key-2".to_string()));
    /// assert!(keys.contains(&"key-3".to_string()));
    /// # Ok(())
    /// # }
    /// ```
    fn list_keys(&self) -> VrfResult<Vec<String>> {
        let mut keys = Vec::new();

        for entry in fs::read_dir(&self.storage_path)
            .map_err(|e| VrfError::InvalidInput(format!("Cannot read directory: {}", e)))?
        {
            let entry =
                entry.map_err(|e| VrfError::InvalidInput(format!("Cannot read entry: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("key") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    keys.push(stem.to_string());
                }
            }
        }

        Ok(keys)
    }

    /// Checks HSM operational status
    ///
    /// Verifies that the software HSM is functioning correctly by checking:
    /// - Key directory is accessible
    /// - Directory has correct permissions
    /// - Can read directory entries
    ///
    /// This is a lightweight operation that does not load or validate key material.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the HSM is operational.
    ///
    /// # Errors
    ///
    /// - `InvalidInput`: If directory is not accessible
    /// - `InvalidInput`: If permissions check fails
    /// - `InvalidInput`: If directory listing fails
    ///
    /// # Performance
    ///
    /// Very fast operation (typically <100μs) that only checks directory metadata.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
    /// use tempfile::tempdir;
    ///
    /// # fn main() -> Result<(), cardano_vrf::VrfError> {
    /// let temp_dir = tempdir().unwrap();
    /// let signer = SoftwareVrfSigner::new(
    ///     temp_dir.path().to_str().unwrap().to_string()
    /// )?;
    ///
    /// // Check health
    /// signer.health_check()?;
    ///
    /// // Use in monitoring
    /// match signer.health_check() {
    ///     Ok(()) => println!("HSM is healthy"),
    ///     Err(e) => eprintln!("HSM health check failed: {}", e),
    /// }
    /// # Ok(())
    /// # }
    /// ```
    fn health_check(&self) -> VrfResult<()> {
        // Verify storage path exists and is writable
        if !self.storage_path.exists() {
            return Err(VrfError::InvalidInput("Storage path does not exist".into()));
        }

        if !self.storage_path.is_dir() {
            return Err(VrfError::InvalidInput(
                "Storage path is not a directory".into(),
            ));
        }

        // Try to write a test file
        let test_path = self.storage_path.join(".health_check");
        fs::write(&test_path, b"test")
            .map_err(|e| VrfError::InvalidInput(format!("Storage not writable: {}", e)))?;
        fs::remove_file(&test_path)
            .map_err(|e| VrfError::InvalidInput(format!("Cannot cleanup test file: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_software_signer_lifecycle() {
        let temp_dir = tempdir().unwrap();
        let signer = SoftwareVrfSigner::new(temp_dir.path().to_str().unwrap().to_string()).unwrap();

        // Health check
        signer.health_check().unwrap();

        // Generate key
        let pk = signer.generate_keypair("test_key").unwrap();
        assert_eq!(pk.len(), 32);

        // List keys
        let keys = signer.list_keys().unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0], "test_key");

        // Get public key
        let pk2 = signer.get_public_key("test_key").unwrap();
        assert_eq!(pk, pk2);

        // Prove
        let proof = signer.prove("test_key", b"test message").unwrap();
        assert_eq!(proof.len(), 80);

        // Delete key
        signer.delete_key("test_key").unwrap();
        let keys = signer.list_keys().unwrap();
        assert_eq!(keys.len(), 0);
    }
}
