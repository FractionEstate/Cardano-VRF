# Quick Start Guide

Get started with Cardano VRF in 5 minutes!

## Table of Contents

- [Installation](#installation)
- [Basic Usage](#basic-usage)
- [Development Workflow](#development-workflow)
- [Production Deployment](#production-deployment)
- [Next Steps](#next-steps)

## Installation

### Prerequisites

- Rust 1.91 or later
- Cargo (comes with Rust)

### Install Rust

If you don't have Rust installed:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Add to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
cardano-vrf = "0.1"
```

Or using cargo:

```bash
cargo add cardano-vrf
```

## Basic Usage

### 1. Simple VRF Proof Generation

Create a file `src/main.rs`:

```rust
use cardano_vrf::{VrfDraft03, VrfError};

fn main() -> Result<(), VrfError> {
    // Generate keypair from seed
    let seed = [0u8; 32];
    let (secret_key, public_key) = VrfDraft03::keypair_from_seed(&seed);

    // Create VRF proof
    let message = b"Hello, Cardano!";
    let proof = VrfDraft03::prove(&secret_key, message)?;

    // Verify proof
    let output = VrfDraft03::verify(&public_key, &proof, message)?;

    println!("✓ VRF proof verified!");
    println!("Output hash: {}", hex::encode(&output));

    Ok(())
}
```

Run it:

```bash
cargo run
```

### 2. Using VRF Draft-13 (Batch Compatible)

```rust
use cardano_vrf::{VrfDraft13, VrfError};

fn main() -> Result<(), VrfError> {
    let seed = [0u8; 32];
    let (secret_key, public_key) = VrfDraft13::keypair_from_seed(&seed);

    let message = b"Batch-compatible VRF";
    let proof = VrfDraft13::prove(&secret_key, message)?;
    let output = VrfDraft13::verify(&public_key, &proof, message)?;

    println!("✓ VRF Draft-13 proof verified!");
    println!("Output: {}", hex::encode(&output));

    Ok(())
}
```

### 3. With hex crate for Display

Add hex to dependencies:

```toml
[dependencies]
cardano-vrf = "0.1"
hex = "0.4"
```

```rust
use cardano_vrf::{VrfDraft03, VrfError};

fn main() -> Result<(), VrfError> {
    let seed = [0u8; 32];
    let (sk, pk) = VrfDraft03::keypair_from_seed(&seed);

    println!("Public key: {}", hex::encode(&pk));

    let proof = VrfDraft03::prove(&sk, b"test")?;
    println!("Proof (80 bytes): {}", hex::encode(&proof));

    let output = VrfDraft03::verify(&pk, &proof, b"test")?;
    println!("Output (64 bytes): {}", hex::encode(&output));

    Ok(())
}
```

## Development Workflow

### Project Structure

```bash
my-vrf-project/
├── Cargo.toml
├── src/
│   ├── main.rs
│   └── lib.rs
└── tests/
    └── integration_test.rs
```

### Example: Cardano Block Validation

```rust
use cardano_vrf::{VrfDraft03, VrfError};
use sha2::{Sha256, Digest};

struct BlockValidator {
    pool_vrf_key: [u8; 32],
}

impl BlockValidator {
    pub fn new(pool_vrf_key: [u8; 32]) -> Self {
        Self { pool_vrf_key }
    }

    pub fn validate_block_vrf(
        &self,
        slot: u64,
        epoch_nonce: &[u8],
        proof: &[u8; 80],
    ) -> Result<bool, VrfError> {
        // Construct VRF input (simplified)
        let mut hasher = Sha256::new();
        hasher.update(&slot.to_le_bytes());
        hasher.update(epoch_nonce);
        let message = hasher.finalize();

        // Verify VRF proof
        let output = VrfDraft03::verify(&self.pool_vrf_key, proof, &message)?;

        // Check if leader (simplified - in reality, compare with stake)
        let leader_threshold = self.calculate_threshold();
        let output_value = u64::from_le_bytes(output[0..8].try_into().unwrap());

        Ok(output_value < leader_threshold)
    }

    fn calculate_threshold(&self) -> u64 {
        // Simplified threshold calculation
        u64::MAX / 100  // 1% chance of being leader
    }
}

fn main() -> Result<(), VrfError> {
    let pool_vrf_key = [0u8; 32];  // Replace with actual key
    let validator = BlockValidator::new(pool_vrf_key);

    // Simulate block validation
    let seed = [1u8; 32];
    let (sk, _) = VrfDraft03::keypair_from_seed(&seed);
    let epoch_nonce = b"epoch123nonce";
    let slot = 12345u64;

    // Pool creates proof
    let mut hasher = Sha256::new();
    hasher.update(&slot.to_le_bytes());
    hasher.update(epoch_nonce);
    let message = hasher.finalize();
    let proof = VrfDraft03::prove(&sk, &message)?;

    // Validator checks if pool is leader
    let pool_vrf_key = VrfDraft03::keypair_from_seed(&seed).1;
    let validator = BlockValidator::new(pool_vrf_key);
    let is_leader = validator.validate_block_vrf(slot, epoch_nonce, &proof)?;

    println!("Is pool leader for slot {}? {}", slot, is_leader);

    Ok(())
}
```

### Testing

Create `tests/integration_test.rs`:

```rust
use cardano_vrf::{VrfDraft03, VrfError};

#[test]
fn test_vrf_roundtrip() -> Result<(), VrfError> {
    let seed = [42u8; 32];
    let (sk, pk) = VrfDraft03::keypair_from_seed(&seed);

    let messages = [
        b"test1" as &[u8],
        b"test2",
        b"longer test message with more bytes",
    ];

    for msg in &messages {
        let proof = VrfDraft03::prove(&sk, msg)?;
        let output = VrfDraft03::verify(&pk, &proof, msg)?;
        assert_eq!(output.len(), 64);
    }

    Ok(())
}

#[test]
fn test_invalid_proof_rejected() {
    let seed = [42u8; 32];
    let (_, pk) = VrfDraft03::keypair_from_seed(&seed);

    let invalid_proof = [0u8; 80];
    let result = VrfDraft03::verify(&pk, &invalid_proof, b"test");

    assert!(result.is_err());
}
```

Run tests:

```bash
cargo test
```

## Production Deployment

### With HSM (Development)

```toml
[dependencies]
cardano-vrf = { version = "0.1", features = ["hsm-software"] }
```

```rust
use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};
use cardano_vrf::VrfError;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize HSM (file-based for development)
    let hsm = SoftwareVrfSigner::new("/tmp/vrf-keys".to_string())?;

    // Generate key in HSM
    let key_id = "validator-001";
    let public_key = hsm.generate_keypair(key_id)?;
    println!("Generated key: {}", hex::encode(public_key));

    // Use key for VRF operations
    let message = b"block-12345";
    let proof = hsm.prove(key_id, message)?;
    println!("Generated proof: {} bytes", proof.len());

    // List all keys
    let keys = hsm.list_keys()?;
    println!("Keys in HSM: {:?}", keys);

    Ok(())
}
```

### With Metrics and Logging

```toml
[dependencies]
cardano-vrf = { version = "0.1", features = ["std", "hsm-software"] }
```

```rust
use cardano_vrf::{
    VrfDraft03, VrfLogger, VrfMetrics,
    LogLevel, VrfOperation,
};
use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let logger = VrfLogger::new(LogLevel::Info);

    // Initialize metrics
    let metrics = VrfMetrics::new();

    // Initialize HSM
    let hsm = SoftwareVrfSigner::new("/secure/keys".to_string())?;
    logger.info(VrfOperation::KeyGeneration, "HSM initialized".to_string());

    // Generate key
    let key_id = "pool-vrf-001";
    let pk = hsm.generate_keypair(key_id)?;
    logger.info(
        VrfOperation::KeyGeneration,
        format!("Generated key: {}", hex::encode(pk))
    );

    // Prove
    let start = std::time::Instant::now();
    let proof = hsm.prove(key_id, b"block-12345")?;
    let duration = start.elapsed();

    metrics.record_prove_success();
    logger.info(
        VrfOperation::Prove,
        format!("Proof generated in {:?}", duration)
    );

    // Export metrics (Prometheus format)
    println!("\n=== Metrics ===");
    println!("{}", metrics.export_prometheus());

    Ok(())
}
```

### Production Configuration

For production, see detailed guides:

- **[HSM Deployment Guide](docs/HSM_DEPLOYMENT_GUIDE.md)** - Complete deployment instructions
- **[Security Policy](docs/SECURITY.md)** - Security best practices

Quick security checklist:
- ✅ Use hardware HSM (AWS CloudHSM, Azure Key Vault, or PKCS#11)
- ✅ Store credentials in secrets manager
- ✅ Enable audit logging
- ✅ Set up monitoring and alerts
- ✅ Implement key rotation
- ✅ Regular security audits

## Next Steps

### Learn More

1. **Read the Documentation**
   - [API Documentation](https://docs.rs/cardano-vrf)
   - [HSM Deployment Guide](docs/HSM_DEPLOYMENT_GUIDE.md)
   - [Security Policy](docs/SECURITY.md)

2. **Explore Examples**
   ```bash
   git clone https://github.com/FractionEstate/Cardano-VRF.git
   cd Cardano-VRF
   cargo run --example basic_usage
   cargo run --example keypair_generation
   cargo run --example production_hsm
   ```

3. **Understand VRF Specifications**
   - [IETF VRF Draft-03](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-vrf-03)
   - [IETF VRF Draft-13](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-vrf-13)
   - [Cardano VRF Documentation](https://github.com/input-output-hk/vrf)

### Common Use Cases

#### 1. Cardano Stake Pool

```rust
// Generate pool VRF key
let seed = [/* secure random seed */];
let (sk, pk) = VrfDraft03::keypair_from_seed(&seed);

// Produce block VRF proof
let epoch_nonce = b"...";
let slot = 12345u64;
let proof = VrfDraft03::prove(&sk, /* construct message */)?;

// Others verify your block
let output = VrfDraft03::verify(&pk, &proof, /* message */)?;
```

#### 2. Randomness Generation

```rust
// Deterministic random value from seed
let seed = b"my application seed";
let (sk, _) = VrfDraft03::keypair_from_seed(&[0u8; 32]);

let input = format!("random-{}", sequence_number);
let proof = VrfDraft03::prove(&sk, input.as_bytes())?;
let random_output = VrfDraft03::proof_to_hash(&proof)?;

// random_output is deterministic but unpredictable
```

#### 3. Fair Leader Election

```rust
// Each validator generates proof
let validators = vec![validator1, validator2, validator3];

for validator in validators {
    let proof = validator.generate_vrf_proof(epoch_nonce, slot)?;
    let output = VrfDraft03::proof_to_hash(&proof)?;

    // Lowest output wins (fair, verifiable)
    if output < current_min {
        current_leader = validator;
        current_min = output;
    }
}
```

### Troubleshooting

**Q: Compilation error about missing features?**

A: Enable the required feature:
```toml
cardano-vrf = { version = "0.1", features = ["std", "hsm-software"] }
```

**Q: VRF verification fails?**

A: Ensure you're using the same VRF version for prove and verify:
- Use VrfDraft03 for both, or
- Use VrfDraft13 for both
- Don't mix versions!

**Q: Need production HSM?**

A: See [HSM Deployment Guide](docs/HSM_DEPLOYMENT_GUIDE.md) for:
- AWS CloudHSM setup
- Azure Key Vault setup
- PKCS#11 hardware HSM setup

### Get Help

- **Documentation**: [docs.rs/cardano-vrf](https://docs.rs/cardano-vrf)
- **Examples**: [GitHub examples/](https://github.com/FractionEstate/Cardano-VRF/tree/main/examples)
- **Issues**: [GitHub Issues](https://github.com/FractionEstate/Cardano-VRF/issues)
- **Security**: security@fractionestate.com

### Contributing

Want to contribute? See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Development setup
- Coding standards
- Testing requirements
- Pull request process

---

**Happy VRF coding!** 🚀
