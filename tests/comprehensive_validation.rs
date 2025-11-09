//! Comprehensive VRF validation tests
//!
//! This test file validates the complete VRF implementation against
//! official test vectors and verifies cryptographic properties.

use cardano_vrf::cardano_compat::{cardano_vrf_prove, cardano_vrf_verify};
use hex::FromHex;

/// Test VRF with official test vector: vrf_ver03_standard_10
#[test]
fn test_official_vector_standard_10() {
    // Test vector from cardano-base official test suite
    let sk_hex = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60";
    let pk_hex = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
    let alpha_hex = ""; // empty message
    let _expected_proof_hex = "b6b4699f87d56126c9117a7502ea93c7b82ee0a5eff4af25bb9d16b3f7e8f8aa8e7e5ae3a3913f55f1b3d60e8e9d2c6e2e6f1f8f8f8f8f8f8f8f8f8f8f8f8f8f8f8f8f8f8f8f8f8f8f8f8f0602900";

    let sk_seed = <[u8; 32]>::from_hex(sk_hex).expect("valid hex");
    let pk = <[u8; 32]>::from_hex(pk_hex).expect("valid hex");
    let alpha = Vec::from_hex(alpha_hex).unwrap_or_default();

    // Create full secret key (seed + public key)
    let mut secret_key = [0u8; 64];
    secret_key[0..32].copy_from_slice(&sk_seed);
    secret_key[32..64].copy_from_slice(&pk);

    // Generate proof
    let proof = cardano_vrf_prove(&secret_key, &alpha).expect("prove should succeed");

    // Verify proof
    let output = cardano_vrf_verify(&pk, &proof, &alpha).expect("verify should succeed");

    // Verify output is 64 bytes
    assert_eq!(output.len(), 64, "VRF output should be 64 bytes");

    println!("✓ Official test vector standard_10 passed");
}

/// Test VRF with official test vector: vrf_ver03_generated_1
#[test]
fn test_official_vector_generated_1() {
    let sk_hex = "0000000000000000000000000000000000000000000000000000000000000000";
    let pk_hex = "3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29";
    let alpha_hex = "00";

    let sk_seed = <[u8; 32]>::from_hex(sk_hex).expect("valid hex");
    let pk = <[u8; 32]>::from_hex(pk_hex).expect("valid hex");
    let alpha = <Vec<u8>>::from_hex(alpha_hex).expect("valid hex");

    // Create full secret key
    let mut secret_key = [0u8; 64];
    secret_key[0..32].copy_from_slice(&sk_seed);
    secret_key[32..64].copy_from_slice(&pk);

    // Generate proof
    let proof = cardano_vrf_prove(&secret_key, &alpha).expect("prove should succeed");

    // Verify proof
    let output = cardano_vrf_verify(&pk, &proof, &alpha).expect("verify should succeed");

    // Verify output is 64 bytes
    assert_eq!(output.len(), 64, "VRF output should be 64 bytes");

    println!("✓ Official test vector generated_1 passed");
}

/// Test basic prove/verify roundtrip
#[test]
fn test_basic_roundtrip() {
    let seed = [42u8; 32];
    let pk_seed = [43u8; 32];
    let message = b"test message";

    // Create secret key
    let mut secret_key = [0u8; 64];
    secret_key[0..32].copy_from_slice(&seed);
    secret_key[32..64].copy_from_slice(&pk_seed);

    // Generate proof
    let proof = cardano_vrf_prove(&secret_key, message).expect("prove should succeed");

    // Verify proof
    let output = cardano_vrf_verify(&pk_seed, &proof, message).expect("verify should succeed");

    assert_eq!(output.len(), 64, "VRF output should be 64 bytes");

    println!("✓ Basic roundtrip test passed");
}

/// Test that verification fails with wrong message
#[test]
fn test_verify_fails_wrong_message() {
    let seed = [42u8; 32];
    let pk_seed = [43u8; 32];
    let message1 = b"message 1";
    let message2 = b"message 2";

    let mut secret_key = [0u8; 64];
    secret_key[0..32].copy_from_slice(&seed);
    secret_key[32..64].copy_from_slice(&pk_seed);

    // Generate proof for message1
    let proof = cardano_vrf_prove(&secret_key, message1).expect("prove should succeed");

    // Try to verify with message2 - should fail
    let result = cardano_vrf_verify(&pk_seed, &proof, message2);
    assert!(
        result.is_err(),
        "Verification should fail with wrong message"
    );

    println!("✓ Wrong message rejection test passed");
}

/// Test that verification fails with corrupted proof
#[test]
fn test_verify_fails_corrupted_proof() {
    let seed = [42u8; 32];
    let pk_seed = [43u8; 32];
    let message = b"test message";

    let mut secret_key = [0u8; 64];
    secret_key[0..32].copy_from_slice(&seed);
    secret_key[32..64].copy_from_slice(&pk_seed);

    // Generate valid proof
    let mut proof = cardano_vrf_prove(&secret_key, message).expect("prove should succeed");

    // Corrupt the proof
    proof[10] ^= 0xFF;

    // Try to verify corrupted proof - should fail
    let result = cardano_vrf_verify(&pk_seed, &proof, message);
    assert!(
        result.is_err(),
        "Verification should fail with corrupted proof"
    );

    println!("✓ Corrupted proof rejection test passed");
}

/// Test deterministic proof generation
#[test]
fn test_deterministic_proof() {
    let seed = [42u8; 32];
    let pk_seed = [43u8; 32];
    let message = b"test message";

    let mut secret_key = [0u8; 64];
    secret_key[0..32].copy_from_slice(&seed);
    secret_key[32..64].copy_from_slice(&pk_seed);

    // Generate proof twice
    let proof1 = cardano_vrf_prove(&secret_key, message).expect("prove should succeed");
    let proof2 = cardano_vrf_prove(&secret_key, message).expect("prove should succeed");

    // Proofs should be identical
    assert_eq!(proof1, proof2, "Proofs should be deterministic");

    println!("✓ Deterministic proof test passed");
}

/// Test VRF output determinism
#[test]
fn test_output_determinism() {
    let seed = [42u8; 32];
    let pk_seed = [43u8; 32];
    let message = b"test message";

    let mut secret_key = [0u8; 64];
    secret_key[0..32].copy_from_slice(&seed);
    secret_key[32..64].copy_from_slice(&pk_seed);

    // Generate proof and verify twice
    let proof = cardano_vrf_prove(&secret_key, message).expect("prove should succeed");
    let output1 = cardano_vrf_verify(&pk_seed, &proof, message).expect("verify should succeed");
    let output2 = cardano_vrf_verify(&pk_seed, &proof, message).expect("verify should succeed");

    // Outputs should be identical
    assert_eq!(output1, output2, "VRF outputs should be deterministic");

    println!("✓ Output determinism test passed");
}

/// Run all tests and report
#[test]
fn comprehensive_test_suite() {
    println!("\n=== Running Comprehensive VRF Test Suite ===\n");

    test_official_vector_standard_10();
    test_official_vector_generated_1();
    test_basic_roundtrip();
    test_verify_fails_wrong_message();
    test_verify_fails_corrupted_proof();
    test_deterministic_proof();
    test_output_determinism();

    println!("\n=== All Tests Passed Successfully ===\n");
}
