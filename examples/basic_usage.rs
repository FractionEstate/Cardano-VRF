//! Basic usage example for Cardano VRF
//!
//! This example demonstrates the basic prove and verify workflow using Draft-03 VRF.

use cardano_vrf::{VrfDraft03, VrfError};

fn main() -> Result<(), VrfError> {
    println!("=== Cardano VRF Basic Usage Example ===\n");

    // Generate a keypair from a seed
    let seed = [42u8; 32];
    let (secret_key, public_key) = VrfDraft03::keypair_from_seed(&seed);

    println!("Generated keypair:");
    println!("  Public key: {}", hex::encode(public_key));
    println!();

    // Create a VRF proof
    let message = b"Hello, Cardano VRF!";
    println!("Message: {}", String::from_utf8_lossy(message));

    let proof = VrfDraft03::prove(&secret_key, message)?;
    println!(
        "Proof generated ({} bytes): {}",
        proof.len(),
        hex::encode(proof)
    );
    println!();

    // Verify the proof and get VRF output
    let output = VrfDraft03::verify(&public_key, &proof, message)?;
    println!("Proof verified successfully!");
    println!(
        "VRF output ({} bytes): {}",
        output.len(),
        hex::encode(output)
    );
    println!();

    // Extract VRF output from proof without full verification
    let hash = VrfDraft03::proof_to_hash(&proof)?;
    println!("VRF hash (from proof): {}", hex::encode(hash));
    println!();

    // Demonstrate that VRF output is deterministic
    let proof2 = VrfDraft03::prove(&secret_key, message)?;
    let output2 = VrfDraft03::verify(&public_key, &proof2, message)?;

    assert_eq!(output, output2);
    println!("✓ VRF output is deterministic for the same message");

    // Demonstrate that different messages produce different outputs
    let different_message = b"Different message";
    let different_proof = VrfDraft03::prove(&secret_key, different_message)?;
    let different_output = VrfDraft03::verify(&public_key, &different_proof, different_message)?;

    assert_ne!(output, different_output);
    println!("✓ Different messages produce different VRF outputs");

    // Demonstrate that invalid proofs are rejected
    let mut invalid_proof = proof;
    invalid_proof[0] ^= 0xFF; // Corrupt the proof

    let verify_result = VrfDraft03::verify(&public_key, &invalid_proof, message);
    assert!(verify_result.is_err());
    println!("✓ Invalid proofs are rejected");

    println!("\n=== Example completed successfully! ===");
    Ok(())
}
