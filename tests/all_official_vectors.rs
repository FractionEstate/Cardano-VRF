//! Complete test suite with all 14 official Cardano VRF test vectors
//!
//! This test suite validates 100% cryptographic parity with Cardano's
//! libsodium implementation using official test vectors from cardano-base-rust.

mod test_vector_parser;
use test_vector_parser::parse_test_vector;

use cardano_vrf::cardano_compat::{cardano_vrf_prove, cardano_vrf_verify};
use cardano_vrf::draft13::{VrfDraft13, OUTPUT_SIZE, PROOF_SIZE as PROOF_SIZE_13};

const TEST_VECTORS_PATH: &str = "test_vectors";

macro_rules! test_draft03_vector {
    ($name:ident, $file:expr) => {
        #[test]
        fn $name() {
            let content = std::fs::read_to_string(format!("{}/{}", TEST_VECTORS_PATH, $file))
                .expect("Failed to read test vector file");

            let vector = parse_test_vector(&content).expect("Failed to parse test vector");

            // Verify it's a Draft-03 vector
            assert_eq!(vector.version, "ietfdraft03", "Expected Draft-03 vector");

            // Construct 64-byte secret key (seed || public_key)
            let mut sk = [0u8; 64];
            sk[0..32].copy_from_slice(&vector.sk);
            sk[32..64].copy_from_slice(&vector.pk);

            let mut pk = [0u8; 32];
            pk.copy_from_slice(&vector.pk);

            // Test prove
            let proof = cardano_vrf_prove(&sk, &vector.alpha).expect("Proof generation failed");

            assert_eq!(&proof[..], &vector.pi[..], "Proof mismatch for {}", $file);

            // Test verify
            let output =
                cardano_vrf_verify(&pk, &proof, &vector.alpha).expect("Verification failed");

            assert_eq!(
                &output[..],
                &vector.beta[..],
                "Output mismatch for {}",
                $file
            );

            println!("✅ {} passed", $file);
        }
    };
}

macro_rules! test_draft13_vector {
    ($name:ident, $file:expr) => {
        #[test]
        fn $name() {
            let content = std::fs::read_to_string(format!("{}/{}", TEST_VECTORS_PATH, $file))
                .expect("Failed to read test vector file");

            let vector = parse_test_vector(&content).expect("Failed to parse test vector");

            // Verify it's a Draft-13 vector
            assert_eq!(vector.version, "ietfdraft13", "Expected Draft-13 vector");

            // Construct secret key (seed || public_key)
            let mut sk = [0u8; 64];
            sk[0..32].copy_from_slice(&vector.sk);
            sk[32..64].copy_from_slice(&vector.pk);

            let mut pk = [0u8; 32];
            pk.copy_from_slice(&vector.pk);

            // Test prove
            let proof = VrfDraft13::prove(&sk, &vector.alpha).expect("Proof generation failed");

            let expected_proof: [u8; PROOF_SIZE_13] = vector
                .pi
                .as_slice()
                .try_into()
                .expect("Invalid proof length");

            assert_eq!(proof, expected_proof, "Proof mismatch for {}", $file);

            // Test verify
            let output =
                VrfDraft13::verify(&pk, &proof, &vector.alpha).expect("Verification failed");

            let expected_beta: [u8; OUTPUT_SIZE] = vector
                .beta
                .as_slice()
                .try_into()
                .expect("Invalid beta length");

            assert_eq!(output, expected_beta, "Output mismatch for {}", $file);

            println!("✅ {} passed", $file);
        }
    };
}

// Draft-03 Standard Vectors (from IETF specification)
test_draft03_vector!(vrf_ver03_standard_10, "vrf_ver03_standard_10");
test_draft03_vector!(vrf_ver03_standard_11, "vrf_ver03_standard_11");
test_draft03_vector!(vrf_ver03_standard_12, "vrf_ver03_standard_12");

// Draft-03 Generated Vectors (Cardano-specific)
test_draft03_vector!(vrf_ver03_generated_1, "vrf_ver03_generated_1");
test_draft03_vector!(vrf_ver03_generated_2, "vrf_ver03_generated_2");
test_draft03_vector!(vrf_ver03_generated_3, "vrf_ver03_generated_3");
test_draft03_vector!(vrf_ver03_generated_4, "vrf_ver03_generated_4");

// Draft-13 Standard Vectors (from IETF specification)
test_draft13_vector!(vrf_ver13_standard_10, "vrf_ver13_standard_10");
test_draft13_vector!(vrf_ver13_standard_11, "vrf_ver13_standard_11");
test_draft13_vector!(vrf_ver13_standard_12, "vrf_ver13_standard_12");

// Draft-13 Generated Vectors (Cardano-specific)
test_draft13_vector!(vrf_ver13_generated_1, "vrf_ver13_generated_1");
test_draft13_vector!(vrf_ver13_generated_2, "vrf_ver13_generated_2");
test_draft13_vector!(vrf_ver13_generated_3, "vrf_ver13_generated_3");
test_draft13_vector!(vrf_ver13_generated_4, "vrf_ver13_generated_4");

#[test]
fn verify_all_test_vectors_present() {
    let expected_files = vec![
        "vrf_ver03_standard_10",
        "vrf_ver03_standard_11",
        "vrf_ver03_standard_12",
        "vrf_ver03_generated_1",
        "vrf_ver03_generated_2",
        "vrf_ver03_generated_3",
        "vrf_ver03_generated_4",
        "vrf_ver13_standard_10",
        "vrf_ver13_standard_11",
        "vrf_ver13_standard_12",
        "vrf_ver13_generated_1",
        "vrf_ver13_generated_2",
        "vrf_ver13_generated_3",
        "vrf_ver13_generated_4",
    ];

    for file in expected_files {
        let path = format!("{}/{}", TEST_VECTORS_PATH, file);
        assert!(
            std::path::Path::new(&path).exists(),
            "Missing test vector: {}",
            file
        );
    }

    println!("✅ All 14 test vectors are present");
}

#[test]
fn test_vector_summary() {
    println!("\n=== Cardano VRF Test Vector Summary ===\n");
    println!("Draft-03 (ECVRF-ED25519-SHA512-Elligator2):");
    println!("  Standard Vectors: 3 (vrf_ver03_standard_10, 11, 12)");
    println!("  Generated Vectors: 4 (vrf_ver03_generated_1, 2, 3, 4)");
    println!("\nDraft-13 (ECVRF-ED25519-SHA512-ELL2):");
    println!("  Standard Vectors: 3 (vrf_ver13_standard_10, 11, 12)");
    println!("  Generated Vectors: 4 (vrf_ver13_generated_1, 2, 3, 4)");
    println!("\nTotal: 14 test vectors");
    println!("\n========================================\n");
}
