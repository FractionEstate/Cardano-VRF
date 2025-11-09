# Contributing to Cardano VRF

Thank you for your interest in contributing to Cardano VRF! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How Can I Contribute?](#how-can-i-contribute)
- [Development Setup](#development-setup)
- [Coding Standards](#coding-standards)
- [Testing Requirements](#testing-requirements)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [HSM Implementation Guidelines](#hsm-implementation-guidelines)

## Code of Conduct

### Our Pledge

We pledge to make participation in our project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

### Our Standards

**Positive behavior includes:**
- Using welcoming and inclusive language
- Being respectful of differing viewpoints
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

**Unacceptable behavior includes:**
- Trolling, insulting/derogatory comments, and personal attacks
- Public or private harassment
- Publishing others' private information without permission
- Other conduct which could reasonably be considered inappropriate

### Enforcement

Instances of abusive, harassing, or otherwise unacceptable behavior may be reported by contacting the project team at contribute@fractionestate.com. All complaints will be reviewed and investigated promptly and fairly.

## How Can I Contribute?

### Reporting Bugs

Before creating bug reports, please check existing issues to avoid duplicates.

**When submitting a bug report, include:**
- Clear, descriptive title
- Steps to reproduce the issue
- Expected vs actual behavior
- Rust version and platform
- Minimal code example demonstrating the issue
- Stack traces or error messages

**Example:**

```markdown
**Title:** VRF Draft-13 proof verification fails with specific message length

**Description:**
When using VrfDraft13::verify with messages exactly 64 bytes long, verification fails with "Invalid proof" error.

**Steps to Reproduce:**
1. Generate keypair with `keypair_from_seed`
2. Create proof with exactly 64-byte message
3. Attempt to verify the proof

**Expected:** Verification succeeds
**Actual:** Returns `VrfError::InvalidProof`

**Environment:**
- Rust version: 1.91.0
- Platform: Linux x86_64
- cardano-vrf version: 0.1.0

**Code:**
[paste minimal reproducible example]
```

### Suggesting Enhancements

Enhancement suggestions are tracked as GitHub issues.

**When suggesting enhancements, include:**
- Clear, descriptive title
- Detailed description of the proposed functionality
- Use cases and examples
- Why this enhancement would be useful
- Potential implementation approach (if known)

### Pull Requests

We actively welcome your pull requests:

1. Fork the repo and create your branch from `main`
2. Make your changes following our coding standards
3. Add tests for any new functionality
4. Update documentation
5. Ensure all tests pass
6. Submit your pull request

## Development Setup

### Prerequisites

- Rust 1.91 or later
- Git
- (Optional) Docker for HSM testing

### Clone and Build

```bash
# Clone repository
git clone https://github.com/FractionEstate/Cardano-VRF.git
cd Cardano-VRF

# Build
cargo build

# Run tests
cargo test

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy -- -D warnings

# Build documentation
cargo doc --no-deps --open
```

### Development Workflow

```bash
# Create feature branch
git checkout -b feature/my-new-feature

# Make changes
# ... edit files ...

# Run full CI checks locally
./scripts/ci-check.sh  # Or run commands manually below

# Format code
cargo fmt

# Check with clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test --all-features

# Run doc tests
cargo test --doc

# Build docs
cargo doc --no-deps

# Commit changes
git add .
git commit -m "feat: Add new feature"

# Push to your fork
git push origin feature/my-new-feature

# Open pull request on GitHub
```

## Coding Standards

### Rust Style Guide

Follow the [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style/):

- Use `cargo fmt` for automatic formatting
- Maximum line length: 100 characters
- Use meaningful variable names
- Prefer explicit types in public APIs
- Use `rustdoc` comments for public items

### Code Organization

```rust
// 1. Module documentation at the top
//! Module description
//!
//! Detailed explanation...

// 2. Imports grouped by:
// - Standard library
// - External crates
// - Internal modules
use std::collections::HashMap;

use curve25519_dalek::scalar::Scalar;
use sha2::Sha512;

use crate::common::*;

// 3. Constants
const PROOF_SIZE: usize = 80;

// 4. Type definitions
pub struct VrfProof {
    // ...
}

// 5. Implementations
impl VrfProof {
    // Public methods first
    pub fn new() -> Self {
        // ...
    }

    // Private methods after
    fn internal_helper(&self) {
        // ...
    }
}

// 6. Tests at the end
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }
}
```

### Error Handling

```rust
// ❌ Don't use unwrap/expect in library code
let value = some_operation().unwrap();

// ✅ Propagate errors with ?
let value = some_operation()?;

// ✅ Or handle explicitly
let value = match some_operation() {
    Ok(v) => v,
    Err(e) => return Err(VrfError::from(e)),
};
```

### Documentation

All public items must have documentation:

```rust
/// Generates a VRF proof for the given message.
///
/// This function creates a cryptographic proof that can be verified
/// to produce a deterministic, unpredictable output.
///
/// # Arguments
///
/// * `secret_key` - The 64-byte Ed25519 secret key
/// * `message` - The message to create a proof for
///
/// # Returns
///
/// Returns a 80-byte VRF proof on success.
///
/// # Errors
///
/// Returns `VrfError::InvalidInput` if the secret key is invalid.
///
/// # Examples
///
/// ```rust
/// use cardano_vrf::VrfDraft03;
///
/// # fn main() -> Result<(), cardano_vrf::VrfError> {
/// let seed = [0u8; 32];
/// let (sk, pk) = VrfDraft03::keypair_from_seed(&seed);
/// let proof = VrfDraft03::prove(&sk, b"test message")?;
/// # Ok(())
/// # }
/// ```
pub fn prove(secret_key: &[u8; 64], message: &[u8]) -> VrfResult<[u8; 80]> {
    // Implementation
}
```

### Security Considerations

1. **No unsafe code** without explicit justification and review
2. **Zeroize sensitive data** using the `zeroize` crate
3. **Constant-time operations** for cryptographic code
4. **No secret-dependent branches** in security-critical paths
5. **Validate all inputs** before processing
6. **Document security assumptions**

Example:

```rust
use zeroize::Zeroize;

pub fn process_secret(mut secret: Vec<u8>) -> Result<(), Error> {
    // Use the secret
    let result = do_something(&secret)?;

    // Zeroize before dropping
    secret.zeroize();

    Ok(result)
}
```

## Testing Requirements

### Unit Tests

All new code must include unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prove_verify_roundtrip() {
        let seed = [0u8; 32];
        let (sk, pk) = VrfDraft03::keypair_from_seed(&seed);
        let message = b"test";

        let proof = VrfDraft03::prove(&sk, message).unwrap();
        let output = VrfDraft03::verify(&pk, &proof, message).unwrap();

        assert_eq!(output.len(), 64);
    }

    #[test]
    fn test_invalid_proof_rejected() {
        let seed = [0u8; 32];
        let (_, pk) = VrfDraft03::keypair_from_seed(&seed);
        let invalid_proof = [0u8; 80];

        let result = VrfDraft03::verify(&pk, &invalid_proof, b"test");
        assert!(result.is_err());
    }
}
```

### Integration Tests

Place integration tests in `tests/`:

```rust
// tests/integration_test.rs
use cardano_vrf::*;

#[test]
fn test_end_to_end_workflow() {
    // Test complete workflow
}
```

### Test Coverage Requirements

- **Minimum 80% code coverage** for new code
- **100% coverage** for critical cryptographic functions
- **All error paths** must be tested
- **Edge cases** must have dedicated tests

### Running Tests

```bash
# All tests
cargo test

# Specific test
cargo test test_prove_verify_roundtrip

# With coverage (requires tarpaulin)
cargo tarpaulin --out Html

# Integration tests only
cargo test --test '*'

# Doc tests only
cargo test --doc
```

## Documentation

### Rustdoc Comments

Use `///` for public items and `//!` for modules:

```rust
//! This module implements VRF Draft-03.
//!
//! # Overview
//!
//! VRF Draft-03 uses Elligator2 mapping...

/// Generates a VRF proof.
///
/// # Examples
///
/// ```
/// # use cardano_vrf::*;
/// // Example code
/// ```
pub fn prove() { }
```

### Documentation Structure

Each module should include:
1. **Overview** - What the module does
2. **Examples** - How to use it
3. **Details** - Implementation specifics
4. **References** - Links to specs/papers

### Building Documentation

```bash
# Build docs
cargo doc --no-deps

# Build and open in browser
cargo doc --no-deps --open

# Check for warnings
cargo doc --no-deps 2>&1 | grep warning

# Test doc examples
cargo test --doc
```

## Pull Request Process

### Before Submitting

- [ ] Code follows style guidelines
- [ ] All tests pass locally
- [ ] New tests added for new functionality
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] No clippy warnings
- [ ] Code is properly formatted

### PR Description Template

```markdown
## Description
[Describe what this PR does]

## Motivation
[Why is this change needed?]

## Changes
- [ ] Feature addition
- [ ] Bug fix
- [ ] Documentation
- [ ] Performance improvement
- [ ] Refactoring

## Testing
[Describe testing done]

## Checklist
- [ ] Tests pass
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] No clippy warnings
- [ ] Formatted with cargo fmt
```

### Review Process

1. **Automated Checks**: CI must pass
2. **Code Review**: At least one maintainer approval required
3. **Testing**: All tests must pass
4. **Documentation**: Docs must be updated
5. **Merge**: Squash and merge to main

### After Merge

- PR will be squashed and merged
- Original PR author will be credited
- Changes will appear in next release

## HSM Implementation Guidelines

### Implementing New HSM Backend

If you're implementing support for a new HSM:

1. **Create new module** in `src/hsm/`
2. **Implement `HsmVrfSigner` trait**
3. **Add comprehensive documentation** (see existing implementations)
4. **Add integration tests**
5. **Update deployment guide**

### HSM Implementation Checklist

- [ ] Implement all `HsmVrfSigner` trait methods
- [ ] Add module-level documentation (100+ lines)
- [ ] Add struct documentation with examples
- [ ] Document all public methods with examples
- [ ] Include security considerations
- [ ] Add performance metrics
- [ ] Create setup/deployment guide section
- [ ] Add error handling examples
- [ ] Include troubleshooting tips
- [ ] Add to README.md feature list
- [ ] Update CHANGELOG.md

### Example HSM Implementation

```rust
//! My HSM implementation
//!
//! [Comprehensive module documentation here]

use crate::hsm::HsmVrfSigner;
use crate::{VrfError, VrfResult};

/// My HSM VRF signer
///
/// [Detailed struct documentation]
pub struct MyHsmVrfSigner {
    // fields
}

impl MyHsmVrfSigner {
    /// Creates a new HSM signer
    ///
    /// [Complete constructor documentation with examples]
    pub fn new(/* params */) -> VrfResult<Self> {
        // Implementation
    }
}

impl HsmVrfSigner for MyHsmVrfSigner {
    fn prove(&self, key_id: &str, message: &[u8]) -> VrfResult<Vec<u8>> {
        // Implementation with comprehensive error handling
    }

    // ... other trait methods
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hsm_operations() {
        // Tests
    }
}
```

## Release Process

Releases are managed by maintainers:

1. Version bump in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag: `git tag -a v0.x.0 -m "Release 0.x.0"`
4. Push tag: `git push origin v0.x.0`
5. GitHub Actions publishes to crates.io
6. Create GitHub release with changelog

## Getting Help

- **Questions**: Open a GitHub Discussion
- **Bugs**: Open a GitHub Issue
- **Security**: Email security@fractionestate.com
- **Chat**: [Discord/Slack link if available]

## Recognition

Contributors will be:
- Listed in release notes
- Credited in commit messages
- Acknowledged in documentation
- Added to CONTRIBUTORS.md (if created)

## License

By contributing, you agree that your contributions will be licensed under the same MIT OR Apache-2.0 dual license as the project.

---

Thank you for contributing to Cardano VRF!
