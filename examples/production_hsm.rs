//! Production-ready VRF service with HSM integration
//!
//! Demonstrates enterprise deployment with:
//! - HSM key management
//! - Metrics collection
//! - Audit logging
//! - Error handling

use cardano_vrf::{HsmConfig, HsmFactory, LogLevel, VrfLogger, VrfMetrics, VrfOperation};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize production components
    let metrics = VrfMetrics::new();
    let logger = VrfLogger::new(LogLevel::Info);

    logger.info(
        VrfOperation::HsmOperation,
        "Initializing VRF service".to_string(),
    );

    // Create HSM signer (software implementation for demo)
    let hsm_config = HsmConfig::Software {
        key_storage_path: "/tmp/vrf_keys".to_string(),
    };

    let signer = match HsmFactory::create_signer(hsm_config) {
        Ok(s) => {
            logger.info(
                VrfOperation::HsmOperation,
                "HSM initialized successfully".to_string(),
            );
            metrics.record_hsm_operation(true);
            s
        }
        Err(e) => {
            logger.error(
                VrfOperation::HsmOperation,
                format!("HSM initialization failed: {}", e),
            );
            metrics.record_hsm_operation(false);
            return Err(Box::new(e));
        }
    };

    // Health check
    logger.info(
        VrfOperation::HsmOperation,
        "Performing HSM health check".to_string(),
    );
    if let Err(e) = signer.health_check() {
        logger.error(
            VrfOperation::HsmOperation,
            format!("Health check failed: {}", e),
        );
        return Err(Box::new(e));
    }

    // Generate a keypair
    logger.info(
        VrfOperation::KeyGeneration,
        "Generating VRF keypair".to_string(),
    );
    let key_id = "production_vrf_key_001";

    let start = Instant::now();
    let public_key = match signer.generate_keypair(key_id) {
        Ok(pk) => {
            let _duration = start.elapsed();
            logger.info(
                VrfOperation::KeyGeneration,
                format!("Keypair generated: {}", hex::encode(pk)),
            );
            pk
        }
        Err(e) => {
            logger.error(
                VrfOperation::KeyGeneration,
                format!("Key generation failed: {}", e),
            );
            return Err(Box::new(e));
        }
    };

    // Prove operation
    let message = b"Cardano block #12345";
    logger.info(
        VrfOperation::Prove,
        format!(
            "Creating VRF proof for message: {:?}",
            String::from_utf8_lossy(message)
        ),
    );

    let start = Instant::now();
    let proof = match signer.prove(key_id, message) {
        Ok(p) => {
            let duration = start.elapsed();
            metrics.record_prove(duration, true);
            logger.info(
                VrfOperation::Prove,
                format!(
                    "Proof generated in {:?}: {}",
                    duration,
                    hex::encode(&p[..16])
                ),
            );
            p
        }
        Err(e) => {
            let duration = start.elapsed();
            metrics.record_prove(duration, false);
            logger.error(VrfOperation::Prove, format!("Prove failed: {}", e));
            return Err(Box::new(e));
        }
    };

    // Verify operation (using public API)
    use cardano_vrf::VrfDraft03;

    logger.info(VrfOperation::Verify, "Verifying VRF proof".to_string());
    let start = Instant::now();

    let proof_array: [u8; 80] = proof.as_slice().try_into().unwrap();
    match VrfDraft03::verify(&public_key, &proof_array, message) {
        Ok(output) => {
            let duration = start.elapsed();
            metrics.record_verify(duration, true);
            logger.info(
                VrfOperation::Verify,
                format!(
                    "Verification succeeded in {:?}: {}",
                    duration,
                    hex::encode(&output[..16])
                ),
            );
            println!("\n✅ VRF Operation Successful!");
            println!("   Public Key: {}", hex::encode(public_key));
            println!("   Message: {:?}", String::from_utf8_lossy(message));
            println!("   Proof (first 32 bytes): {}", hex::encode(&proof[..32]));
            println!("   Output (first 32 bytes): {}", hex::encode(&output[..32]));
        }
        Err(e) => {
            let duration = start.elapsed();
            metrics.record_verify(duration, false);
            logger.error(VrfOperation::Verify, format!("Verification failed: {}", e));
            return Err(Box::new(e));
        }
    }

    // Display metrics
    println!("\n📊 Metrics:");
    println!("{}", metrics.json_format());

    // List all keys
    logger.info(VrfOperation::HsmOperation, "Listing HSM keys".to_string());
    match signer.list_keys() {
        Ok(keys) => {
            println!("\n🔑 HSM Keys: {:?}", keys);
        }
        Err(e) => {
            logger.error(
                VrfOperation::HsmOperation,
                format!("Failed to list keys: {}", e),
            );
        }
    }

    // Cleanup (optional - comment out to keep keys)
    logger.info(
        VrfOperation::HsmOperation,
        format!("Deleting key: {}", key_id),
    );
    if let Err(e) = signer.delete_key(key_id) {
        logger.error(
            VrfOperation::HsmOperation,
            format!("Failed to delete key: {}", e),
        );
    }

    println!("\n✅ Production VRF service completed successfully");

    Ok(())
}
