# Security Policy

## Supported Versions

| Version | Supported          | Security Updates |
| ------- | ------------------ | ---------------- |
| 0.1.x   | :white_check_mark: | Active           |

## Security Features

### Cryptographic Implementation

This library implements Cardano-compatible VRF (Verifiable Random Function) with the following security properties:

- **VRF Draft-03**: 80-byte proofs using Elligator2 mapping (100% compatible with libsodium)
- **VRF Draft-13**: 128-byte proofs using Try-and-Increment (TAI) mapping (Cardano mainnet)
- **Curve25519**: Industry-standard elliptic curve cryptography
- **Ed25519**: Digital signatures with 128-bit security level
- **SHA-512**: Cryptographic hashing

### Key Management Security

#### Software HSM (Development Only)
⚠️ **NOT SECURE FOR PRODUCTION**
- Keys stored as plain files with 0600 permissions
- No tamper resistance
- Vulnerable to root/admin access
- **Use only for development and testing**

#### Hardware HSM (Production)
✅ **Production-Ready Security**
- Private keys never leave HSM boundary
- FIPS 140-2 Level 3 tamper-resistant hardware
- Physical security and zeroization on attack
- Certified random number generation
- Audit logging of all key operations

### Memory Safety

- Written in 100% safe Rust
- No unsafe blocks in core VRF code
- Automatic memory management prevents:
  - Buffer overflows
  - Use-after-free
  - Double-free
  - Memory leaks

### Side-Channel Resistance

Using `curve25519-dalek` which implements:
- Constant-time operations
- Protection against timing attacks
- Protection against cache-timing attacks
- No secret-dependent branches

### Dependencies Security

All dependencies are:
- Actively maintained
- Audited for security issues
- Pinned to specific versions in Cargo.lock
- Regularly updated

## Reporting a Vulnerability

### Where to Report

**DO NOT** open public GitHub issues for security vulnerabilities.

Please report security vulnerabilities privately:

1. **Email**: security@fractionestate.com
2. **GitHub Security Advisories**: Use the "Security" tab on GitHub

### What to Include

Please include:

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)
- Your contact information

### Response Timeline

- **Initial Response**: Within 48 hours
- **Severity Assessment**: Within 5 business days
- **Fix Development**: Depends on severity
  - Critical: 7-14 days
  - High: 14-30 days
  - Medium: 30-60 days
  - Low: Next scheduled release
- **Public Disclosure**: After fix is released

### Disclosure Policy

We follow **coordinated disclosure**:

1. You report the vulnerability privately
2. We confirm and develop a fix
3. We release the fix
4. We publicly acknowledge your contribution (with your permission)

### Bug Bounty

Currently, we do not have a formal bug bounty program. However, we will:
- Publicly acknowledge responsible disclosures
- Consider reasonable compensation for critical findings
- Credit you in release notes and security advisories

## Security Best Practices

### For Library Users

#### 1. Never Use Software HSM in Production

```rust
// ❌ NEVER DO THIS IN PRODUCTION
let signer = SoftwareVrfSigner::new("/var/keys".to_string())?;

// ✅ USE HARDWARE HSM INSTEAD
let signer = AwsCloudHsmVrfSigner::new(/* ... */)?;
```

#### 2. Secure Credential Storage

```rust
// ❌ NEVER HARDCODE CREDENTIALS
let signer = AwsCloudHsmVrfSigner::new(
    "cluster-123".to_string(),
    "my_user".to_string(),
    "hardcoded_password".to_string(), // WRONG!
)?;

// ✅ USE ENVIRONMENT VARIABLES OR SECRETS MANAGER
let signer = AwsCloudHsmVrfSigner::new(
    std::env::var("CLOUDHSM_CLUSTER_ID")?,
    std::env::var("CLOUDHSM_USER")?,
    std::env::var("CLOUDHSM_PASSWORD")?,
)?;
```

#### 3. Validate All Inputs

```rust
use cardano_vrf::{VrfProver, VrfVerifier};

// Always verify proofs
match verifier.verify(&public_key, &proof, &message) {
    Ok(output) => {
        // Safe to use output
    },
    Err(_) => {
        // Invalid proof - reject
        return Err("Invalid VRF proof");
    }
}
```

#### 4. Use Constant-Time Comparison

```rust
use subtle::ConstantTimeEq;

// ❌ TIMING ATTACK VULNERABLE
if output1 == output2 {
    // ...
}

// ✅ CONSTANT-TIME COMPARISON
if bool::from(output1.ct_eq(&output2)) {
    // ...
}
```

#### 5. Zeroize Sensitive Data

```rust
use zeroize::Zeroize;

fn handle_secret(mut secret: Vec<u8>) {
    // Use secret

    // Zeroize before dropping
    secret.zeroize();
}
```

#### 6. Enable All Security Features

```toml
# Cargo.toml
[dependencies]
cardano-vrf = { version = "0.1", features = ["metrics", "logging"] }
```

#### 7. Regular Updates

```bash
# Check for vulnerabilities
cargo audit

# Update dependencies
cargo update

# Review changes
git diff Cargo.lock
```

### For HSM Deployment

#### AWS CloudHSM Security

```bash
# 1. Enable VPC Flow Logs
aws ec2 create-flow-logs \
    --resource-type VPC \
    --resource-ids vpc-abc123 \
    --traffic-type ALL \
    --log-destination-type cloud-watch-logs

# 2. Restrict Security Group
aws ec2 authorize-security-group-ingress \
    --group-id sg-xyz789 \
    --protocol tcp \
    --port 2223-2225 \
    --source-group sg-xyz789  # Only from same SG

# 3. Enable CloudTrail
aws cloudtrail create-trail \
    --name cloudhsm-audit \
    --s3-bucket-name audit-logs

# 4. Use IAM roles, not access keys
# Attach role to EC2 instance instead of embedding credentials
```

#### Azure Key Vault Security

```bash
# 1. Enable Private Endpoint
az network private-endpoint create \
    --name kv-endpoint \
    --resource-group prod-rg \
    --vnet-name prod-vnet \
    --subnet default \
    --private-connection-resource-id $KEYVAULT_ID \
    --group-id vault \
    --connection-name kv-connection

# 2. Disable Public Access
az keyvault update \
    --name my-keyvault \
    --default-action Deny

# 3. Enable Purge Protection
az keyvault update \
    --name my-keyvault \
    --enable-purge-protection true

# 4. Use Managed Identity
az vm identity assign \
    --resource-group prod-rg \
    --name my-vm
```

#### PKCS#11 HSM Security

```bash
# 1. Set restrictive PIN policy
# In HSM: Minimum 8 characters, complexity requirements

# 2. Enable audit logging
# Configure HSM to log all operations to syslog

# 3. Physical security
# HSMs must be in locked, access-controlled room
# Video surveillance recommended
# Tamper-evident seals on devices

# 4. Dual control for sensitive operations
# Require two administrators for key export/destruction
```

## Security Audits

### Code Audits

This library has undergone:
- ✅ Internal security review
- ✅ Automated security scanning (cargo-audit)
- ✅ Dependency vulnerability scanning
- ⏳ External cryptographic audit (planned)

### Audit Reports

Audit reports will be published in `/docs/audits/` when available.

### Continuous Security

We use automated security scanning:

```yaml
# .github/workflows/security.yml
- name: Security Audit
  run: cargo audit

- name: Dependency Check
  run: cargo outdated --exit-code 1

- name: Vulnerability Scan
  run: cargo deny check advisories
```

## Cryptographic Validation

### Test Vectors

We validate against official test vectors:

- ✅ IETF VRF Draft-03 test vectors
- ✅ IETF VRF Draft-13 test vectors
- ✅ Cardano libsodium compatibility tests
- ✅ IOHK VRF implementation comparison

### Fuzzing

Continuous fuzzing with `cargo-fuzz`:

```bash
# Fuzz VRF proof generation
cargo fuzz run fuzz_prove

# Fuzz VRF verification
cargo fuzz run fuzz_verify

# Fuzz key generation
cargo fuzz run fuzz_keygen
```

## Known Limitations

### Software HSM

⚠️ **Known Issues:**
- Keys stored in plain files
- No protection against privileged users
- No tamper detection
- **Status:** By design - for development only

### PKCS#11 Implementation

⚠️ **Status:** Pending implementation
- Currently placeholder returning errors
- Full implementation requires `cryptoki` crate
- See HSM_DEPLOYMENT_GUIDE.md for details

### AWS CloudHSM Integration

⚠️ **Status:** Pending implementation
- Currently placeholder returning errors
- Full implementation requires AWS SDK
- See HSM_DEPLOYMENT_GUIDE.md for details

### Azure Key Vault Integration

⚠️ **Status:** Pending implementation
- Currently placeholder returning errors
- Full implementation requires Azure SDK
- See HSM_DEPLOYMENT_GUIDE.md for details

## Compliance

### Standards Compliance

- **FIPS 140-2**: When using certified HSMs
- **NIST SP 800-90A**: Random number generation (via HSM)
- **RFC 8032**: Ed25519 signatures
- **IETF VRF Draft-03**: 80-byte proofs
- **IETF VRF Draft-13**: 128-byte proofs

### Cardano Compatibility

- ✅ 100% byte-for-byte compatible with libsodium VRF
- ✅ Tested against Cardano mainnet test vectors
- ✅ Compatible with cardano-node VRF operations

## Security Contacts

- **Security Email**: security@fractionestate.com
- **GitHub Security**: Use Security Advisories tab
- **PGP Key**: Available on request

## Attribution

We would like to thank the following for security contributions:

- [List will be updated as contributions are made]

## Additional Resources

- [OWASP Cryptographic Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Cryptographic_Storage_Cheat_Sheet.html)
- [NIST Cryptographic Standards](https://csrc.nist.gov/projects/cryptographic-standards-and-guidelines)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [HSM Deployment Guide](./HSM_DEPLOYMENT_GUIDE.md)

---

**Last Updated:** November 2025
**Next Review:** February 2026
