//! Software-based VRF signer for testing and development
//!
//! This module provides a software implementation of the HSM trait
//! that stores keys in encrypted files. NOT for production use.

use crate::{VrfError, VrfResult};
use crate::hsm::HsmVrfSigner;
use crate::cardano_compat::cardano_vrf_prove;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;
use ed25519_dalek::SigningKey;
use zeroize::Zeroizing;

/// Software-based VRF signer (for testing only)
pub struct SoftwareVrfSigner {
    storage_path: PathBuf,
    keys: RwLock<HashMap<String, [u8; 64]>>,
}

impl SoftwareVrfSigner {
    /// Create a new software signer
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
    fn prove(&self, key_id: &str, message: &[u8]) -> VrfResult<Vec<u8>> {
        let sk = self.load_key(key_id)?;
        let proof = cardano_vrf_prove(&sk, message)?;
        Ok(proof.to_vec())
    }

    fn get_public_key(&self, key_id: &str) -> VrfResult<[u8; 32]> {
        let sk = self.load_key(key_id)?;
        let mut pk = [0u8; 32];
        pk.copy_from_slice(&sk[32..64]);
        Ok(pk)
    }

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

    fn list_keys(&self) -> VrfResult<Vec<String>> {
        let mut keys = Vec::new();

        for entry in fs::read_dir(&self.storage_path)
            .map_err(|e| VrfError::InvalidInput(format!("Cannot read directory: {}", e)))? {
            let entry = entry.map_err(|e| VrfError::InvalidInput(format!("Cannot read entry: {}", e)))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("key") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    keys.push(stem.to_string());
                }
            }
        }

        Ok(keys)
    }

    fn health_check(&self) -> VrfResult<()> {
        // Verify storage path exists and is writable
        if !self.storage_path.exists() {
            return Err(VrfError::InvalidInput("Storage path does not exist".into()));
        }

        if !self.storage_path.is_dir() {
            return Err(VrfError::InvalidInput("Storage path is not a directory".into()));
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
