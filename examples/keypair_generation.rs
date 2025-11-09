//! Keypair generation example
//!
//! This example demonstrates different ways to generate VRF keypairs.

use cardano_vrf::{VrfDraft03, VrfDraft13};

fn main() {
    println!("=== Cardano VRF Keypair Generation ===\n");

    // Method 1: Generate from a deterministic seed
    println!("1. Deterministic keypair generation:");
    let seed = [1u8; 32];
    let (sk_03, pk_03) = VrfDraft03::keypair_from_seed(&seed);
    println!("  Draft-03 Public Key: {}", hex::encode(pk_03));

    let (_sk_13, pk_13) = VrfDraft13::keypair_from_seed(&seed);
    println!("  Draft-13 Public Key: {}", hex::encode(pk_13));
    println!();

    // Method 2: Generate from random seed (using rand crate)
    println!("2. Random keypair generation:");
    use rand::RngCore;
    let mut rng = rand::thread_rng();
    let mut random_seed = [0u8; 32];
    rng.fill_bytes(&mut random_seed);

    let (_sk, pk) = VrfDraft03::keypair_from_seed(&random_seed);
    println!("  Random Public Key: {}", hex::encode(pk));
    println!();

    // Method 3: Demonstrate keypair structure
    println!("3. Understanding keypair structure:");
    println!("  Seed size: 32 bytes");
    println!("  Secret key size: 64 bytes (seed + public key)");
    println!("  Public key size: 32 bytes");
    println!();

    println!("  Secret key structure:");
    println!("    Bytes [0..32]:  Seed material");
    println!("    Bytes [32..64]: Public key");
    println!();

    // Verify that the public key in the secret key matches
    assert_eq!(&sk_03[32..64], &pk_03[..]);
    println!("  ✓ Public key in secret key matches standalone public key");
    println!();

    // Method 4: Compare Draft-03 and Draft-13
    println!("4. Comparing Draft-03 and Draft-13:");
    let test_seed = [255u8; 32];
    let (_, pk_draft03) = VrfDraft03::keypair_from_seed(&test_seed);
    let (_, pk_draft13) = VrfDraft13::keypair_from_seed(&test_seed);

    // They should produce the same public key (same key derivation)
    assert_eq!(pk_draft03, pk_draft13);
    println!("  ✓ Both draft versions use the same key derivation");
    println!();

    // Method 5: Multiple keypairs from different seeds
    println!("5. Generating multiple keypairs:");
    for i in 0..3 {
        let mut seed = [0u8; 32];
        seed[0] = i;

        let (_, pk) = VrfDraft03::keypair_from_seed(&seed);
        println!("  Keypair {}: {}", i, hex::encode(&pk[..16]));
    }

    println!("\n=== Example completed successfully! ===");
}
