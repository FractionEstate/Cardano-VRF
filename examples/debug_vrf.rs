//! Debug VRF to see intermediate values

use cardano_vrf::{VrfDraft03, VrfError};

fn main() -> Result<(), VrfError> {
    println!("=== VRF Debug Example ===\n");

    // Generate a simple keypair
    let seed = [1u8; 32];
    let (secret_key, public_key) = VrfDraft03::keypair_from_seed(&seed);

    println!("Seed: {}", hex::encode(seed));
    println!("Secret key (64 bytes):");
    println!("  First 32 (seed): {}", hex::encode(&secret_key[0..32]));
    println!("  Last 32 (pubkey): {}", hex::encode(&secret_key[32..64]));
    println!("Public key: {}", hex::encode(public_key));
    println!();

    // Simple message
    let message = b"test";
    println!("Message: {}", String::from_utf8_lossy(message));
    println!();

    // Generate proof
    let proof = VrfDraft03::prove(&secret_key, message)?;
    println!("Proof ({} bytes): {}", proof.len(), hex::encode(proof));
    println!("  Gamma (32 bytes): {}", hex::encode(&proof[0..32]));
    println!("  c (16 bytes):     {}", hex::encode(&proof[32..48]));
    println!("  s (32 bytes):     {}", hex::encode(&proof[48..80]));
    println!();

    // Try verification
    match VrfDraft03::verify(&public_key, &proof, message) {
        Ok(output) => {
            println!("✓ Verification succeeded!");
            println!("Output: {}", hex::encode(output));
        }
        Err(e) => {
            println!("✗ Verification failed: {:?}", e);
        }
    }

    Ok(())
}
