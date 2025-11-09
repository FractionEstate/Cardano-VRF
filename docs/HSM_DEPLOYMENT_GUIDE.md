# HSM Deployment Guide

Complete guide for deploying Cardano VRF with Hardware Security Modules in production environments.

## Table of Contents

- [Overview](#overview)
- [HSM Selection Guide](#hsm-selection-guide)
- [Software HSM (Development/Testing)](#software-hsm-developmenttesting)
- [PKCS#11 HSM (On-Premises Production)](#pkcs11-hsm-on-premises-production)
- [AWS CloudHSM (AWS Production)](#aws-cloudhsm-aws-production)
- [Azure Key Vault (Azure Production)](#azure-key-vault-azure-production)
- [Security Hardening](#security-hardening)
- [Monitoring & Alerts](#monitoring--alerts)
- [Disaster Recovery](#disaster-recovery)
- [Compliance & Audit](#compliance--audit)

## Overview

This library supports four HSM backends for VRF key management:

| HSM Type | Use Case | Security Level | Cost | Availability |
|----------|----------|----------------|------|--------------|
| Software | Development/Testing | ⚠️ Low | Free | ✅ Ready |
| PKCS#11 | On-premises production | High (FIPS 140-2 L3) | $$$$ | 🚧 Pending |
| AWS CloudHSM | AWS production | High (FIPS 140-2 L3) | $$$ | 🚧 Pending |
| Azure Key Vault | Azure production | High (FIPS 140-2 L3) | $$ | 🚧 Pending |

## HSM Selection Guide

### Decision Tree

```
Are you in production?
├── NO → Use Software HSM (testing only)
└── YES → Continue...
    │
    ├── Running on AWS?
    │   └── YES → Use AWS CloudHSM
    │
    ├── Running on Azure?
    │   └── YES → Use Azure Key Vault
    │
    └── On-premises or multi-cloud?
        └── YES → Use PKCS#11 HSM
```

### Detailed Comparison

#### Software HSM
**✅ Pros:**
- Zero cost
- Easy setup
- Fast development iteration
- Works everywhere

**❌ Cons:**
- Keys stored in plain files
- No tamper resistance
- No compliance certification
- **NEVER use in production**

**When to Use:**
- Local development
- CI/CD testing
- Integration testing
- Demonstrations

---

#### PKCS#11 HSM
**✅ Pros:**
- True hardware security
- FIPS 140-2 Level 3 certified
- Vendor-independent standard
- On-premises control
- No recurring cloud costs

**❌ Cons:**
- High upfront cost ($10K-$100K)
- Physical security required
- Manual maintenance
- No auto-scaling
- Implementation pending

**When to Use:**
- On-premises data centers
- Air-gapped environments
- Regulatory data residency
- Multi-cloud deployments
- High transaction volume

**Recommended Vendors:**
- Thales nShield (best performance)
- Utimaco SecurityServer (PCI certified)
- Gemalto SafeNet Luna (banking grade)

---

#### AWS CloudHSM
**✅ Pros:**
- Managed service (AWS handles hardware)
- Multi-AZ high availability
- Auto-failover
- FIPS 140-2 Level 3
- Scales with AWS infrastructure

**❌ Cons:**
- $1.60/hour per HSM (~$1,152/month)
- Minimum 2 HSMs for HA ($2,304/month)
- AWS vendor lock-in
- Implementation pending

**When to Use:**
- AWS-native applications
- EC2/ECS/Lambda workloads
- Need managed HA
- Variable workloads

**Cost Optimization:**
- Start with 2 HSMs (minimum HA)
- Scale up during high load
- Use Standard tier keys for dev/test
- Consider KMS for less sensitive keys

---

#### Azure Key Vault
**✅ Pros:**
- Fully managed service
- Global availability
- Integrated with Azure AD
- REST API (no client software)
- Geo-replication
- Lowest cost ($1/key/month)

**❌ Cons:**
- Multi-tenant (logical isolation)
- Higher latency (50-100ms)
- Rate limits (5000 ops/10s)
- Implementation pending

**When to Use:**
- Azure-native applications
- Global distributed apps
- Budget-conscious projects
- REST API preference
- Azure AD integration needed

**Cost Optimization:**
- Cache public keys
- Use Standard tier for dev ($0.03/10K ops)
- Batch operations when possible
- Monitor usage patterns

---

## Software HSM (Development/Testing)

### Quick Start

```bash
# Install dependencies
cargo add cardano-vrf

# Create test environment
mkdir -p /tmp/vrf-test-keys
chmod 700 /tmp/vrf-test-keys
```

```rust
use cardano_vrf::hsm::{HsmVrfSigner, software::SoftwareVrfSigner};

fn main() -> Result<(), cardano_vrf::VrfError> {
    // Initialize
    let signer = SoftwareVrfSigner::new("/tmp/vrf-test-keys".to_string())?;

    // Generate key
    let pk = signer.generate_keypair("test-validator")?;
    println!("Public key: {}", hex::encode(pk));

    // Create VRF proof
    let proof = signer.prove("test-validator", b"block-123")?;
    println!("Proof: {}", hex::encode(proof));

    Ok(())
}
```

### Security Notes

⚠️ **CRITICAL**: The Software HSM is **NOT SECURE** for production use:

- Keys stored in plain files (even with 0600 permissions)
- No protection against root/admin access
- No tamper detection
- No audit logging
- Vulnerable to memory dumps
- No FIPS certification

**Use only for:**
- Development
- Testing
- CI/CD pipelines
- Learning/demos

---

## PKCS#11 HSM (On-Premises Production)

### Status: 🚧 Implementation Pending

Full implementation requires the `cryptoki` crate and PKCS#11 library.

### Hardware Recommendations

#### For Cardano Stake Pools (1-10 pools)
**YubiHSM 2**
- Cost: ~$650
- Form factor: USB device
- Performance: 15ms/signature
- Capacity: ~256 keys
- Good for: Small-medium deployments

#### For Cardano Stake Pool Operators (10-100 pools)
**Thales nShield Connect**
- Cost: ~$15,000
- Form factor: 1U rack mount
- Performance: 2ms/signature
- Capacity: 10,000+ keys
- Good for: Professional operators

#### For Large Exchanges/Enterprises
**Thales nShield Edge**
- Cost: ~$50,000+
- Form factor: Rack mount cluster
- Performance: <1ms/signature
- Capacity: Unlimited (clustered)
- Good for: High-volume production

### Setup Example (SoftHSMv2 for Testing)

```bash
# Install SoftHSM (software emulator)
sudo apt-get install softhsm2

# Initialize token
softhsm2-util --init-token --slot 0 \
    --label "cardano-vrf" \
    --so-pin 1234 \
    --pin 5678

# Verify
softhsm2-util --show-slots
```

### Future Rust Usage

```rust
// When implemented:
use cardano_vrf::hsm::pkcs11::Pkcs11VrfSigner;

let signer = Pkcs11VrfSigner::new(
    "/usr/lib/softhsm/libsofthsm2.so".to_string(),
    0,  // slot
    "5678".to_string()  // PIN from environment!
)?;
```

---

## AWS CloudHSM (AWS Production)

### Status: 🚧 Implementation Pending

### Prerequisites

1. **AWS Account** with CloudHSM permissions
2. **VPC** with at least 2 subnets in different AZs
3. **EC2 instance** in same VPC for client
4. **IAM permissions** for CloudHSM management

### Architecture

```
┌─────────────────────────────────────────────────────┐
│ VPC (10.0.0.0/16)                                   │
│                                                     │
│  ┌─────────────────┐         ┌─────────────────┐  │
│  │ Subnet (AZ-1a)  │         │ Subnet (AZ-1b)  │  │
│  │ 10.0.1.0/24     │         │ 10.0.2.0/24     │  │
│  │                 │         │                 │  │
│  │  ┌───────────┐  │         │  ┌───────────┐  │  │
│  │  │ HSM 1     │  │         │  │ HSM 2     │  │  │
│  │  │ Primary   │◄─┼────────┼──┤ Replica   │  │  │
│  │  └───────────┘  │         │  └───────────┘  │  │
│  │                 │         │                 │  │
│  │  ┌───────────┐  │         │                 │  │
│  │  │ EC2       │  │         │                 │  │
│  │  │ Client    │──┼─────────┘                 │  │
│  │  └───────────┘  │                           │  │
│  └─────────────────┘         └─────────────────┘  │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Step-by-Step Setup

#### 1. Create CloudHSM Cluster

```bash
# Create cluster
aws cloudhsmv2 create-cluster \
    --hsm-type hsm1.medium \
    --subnet-ids subnet-abc123 subnet-def456 \
    --backup-retention-policy Type=DAYS,Value=90 \
    --tags Key=Environment,Value=production

# Note the cluster ID: cluster-abc123def456
```

#### 2. Create HSMs (Minimum 2 for HA)

```bash
# Create HSM in AZ 1
aws cloudhsmv2 create-hsm \
    --cluster-id cluster-abc123def456 \
    --availability-zone us-east-1a

# Create HSM in AZ 2 (for HA)
aws cloudhsmv2 create-hsm \
    --cluster-id cluster-abc123def456 \
    --availability-zone us-east-1b

# Wait for ACTIVE status (takes ~10 minutes)
aws cloudhsmv2 describe-clusters \
    --filters clusterIds=cluster-abc123def456
```

#### 3. Initialize Cluster

```bash
# Download cluster certificate
aws cloudhsmv2 describe-clusters \
    --filters clusterIds=cluster-abc123def456 \
    --query 'Clusters[0].Certificates.ClusterCsr' \
    --output text > cluster.csr

# Sign and upload (follow AWS documentation)
# This establishes trust between you and AWS
```

#### 4. Install Client Software

```bash
# Amazon Linux 2 / RHEL
wget https://s3.amazonaws.com/cloudhsmv2-software/CloudHsmClient/EL7/cloudhsm-client-latest.el7.x86_64.rpm
sudo yum install -y ./cloudhsm-client-latest.el7.x86_64.rpm
sudo yum install -y cloudhsm-client-pkcs11

# Configure client
sudo /opt/cloudhsm/bin/configure -a <HSM-ENI-IP>

# Start daemon
sudo systemctl start cloudhsm-client
sudo systemctl enable cloudhsm-client
```

#### 5. Create Crypto User

```bash
# Connect to HSM
/opt/cloudhsm/bin/cloudhsm_mgmt_util /opt/cloudhsm/etc/cloudhsm_mgmt_util.cfg

# In cloudhsm_mgmt_util:
aws-cloudhsm> loginHSM CO admin <initial-password>
aws-cloudhsm> changePswd CO admin <new-secure-password>
aws-cloudhsm> createUser CU vrf_operator <secure-password>
aws-cloudhsm> quit
```

#### 6. Store Credentials Securely

```bash
# Store in AWS Secrets Manager
aws secretsmanager create-secret \
    --name prod/cloudhsm/vrf-credentials \
    --secret-string '{
        "cluster_id": "cluster-abc123def456",
        "username": "vrf_operator",
        "password": "SECURE_PASSWORD_HERE"
    }'
```

### Security Hardening

#### Network Security

```bash
# Create security group
aws ec2 create-security-group \
    --group-name cloudhsm-client \
    --description "CloudHSM client access" \
    --vpc-id vpc-abc123

# Allow HSM communication (port 2223-2225)
aws ec2 authorize-security-group-ingress \
    --group-id sg-xyz789 \
    --protocol tcp \
    --port 2223-2225 \
    --source-group sg-xyz789

# Enable VPC Flow Logs
aws ec2 create-flow-logs \
    --resource-type VPC \
    --resource-ids vpc-abc123 \
    --traffic-type ALL \
    --log-destination-type cloud-watch-logs \
    --log-group-name /aws/vpc/cloudhsm
```

#### IAM Permissions

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "cloudhsm:DescribeClusters",
        "cloudhsm:DescribeBackups"
      ],
      "Resource": "*"
    },
    {
      "Effect": "Allow",
      "Action": [
        "secretsmanager:GetSecretValue"
      ],
      "Resource": "arn:aws:secretsmanager:us-east-1:123456789012:secret:prod/cloudhsm/*"
    }
  ]
}
```

### Monitoring

```bash
# Enable CloudWatch metrics
aws cloudwatch put-metric-alarm \
    --alarm-name cloudhsm-unhealthy \
    --alarm-description "Alert when HSM becomes unhealthy" \
    --metric-name HSMUnhealthy \
    --namespace AWS/CloudHSM \
    --statistic Maximum \
    --period 60 \
    --evaluation-periods 1 \
    --threshold 1 \
    --comparison-operator GreaterThanOrEqualToThreshold \
    --alarm-actions arn:aws:sns:us-east-1:123456789012:ops-alerts
```

### Cost Optimization

**Base Cost:** $1.60/hour per HSM = $1,152/month

**Minimum Production Setup:**
- 2 HSMs (HA): $2,304/month
- No data transfer charges
- Backup storage: Included

**Optimization Tips:**
1. Use exactly 2 HSMs for small deployments
2. Scale to 3+ HSMs only when needed
3. Use development account with 1 HSM
4. Delete unused clusters immediately

---

## Azure Key Vault (Azure Production)

### Status: 🚧 Implementation Pending

### Prerequisites

1. **Azure Subscription** with Key Vault permissions
2. **Azure AD Tenant**
3. **Service Principal** or Managed Identity
4. **Virtual Network** (for private endpoints)

### Architecture

```
┌─────────────────────────────────────────────────────┐
│ Azure Subscription                                  │
│                                                     │
│  ┌─────────────────────────────────────────────┐  │
│  │ Resource Group: cardano-vrf-rg              │  │
│  │                                             │  │
│  │  ┌────────────────────────────────────┐    │  │
│  │  │ Key Vault: cardano-vrf-kv          │    │  │
│  │  │ - Premium Tier (HSM-backed)        │    │  │
│  │  │ - Private Endpoint Enabled         │    │  │
│  │  │ - Soft Delete: Enabled             │    │  │
│  │  │ - Purge Protection: Enabled        │    │  │
│  │  └────────────────────────────────────┘    │  │
│  │                                             │  │
│  │  ┌────────────────────────────────────┐    │  │
│  │  │ VM/AKS: cardano-node               │    │  │
│  │  │ - Managed Identity: Enabled        │    │  │
│  │  │ - Role: Key Vault Crypto User      │    │  │
│  │  └────────────────────────────────────┘    │  │
│  │                                             │  │
│  │  ┌────────────────────────────────────┐    │  │
│  │  │ Log Analytics: cardano-logs        │    │  │
│  │  │ - Retention: 90 days               │    │  │
│  │  └────────────────────────────────────┘    │  │
│  │                                             │  │
│  └─────────────────────────────────────────────┘  │
│                                                     │
└─────────────────────────────────────────────────────┘
```

### Step-by-Step Setup

#### 1. Create Resource Group

```bash
az group create \
    --name cardano-vrf-rg \
    --location eastus \
    --tags Environment=production Project=cardano-vrf
```

#### 2. Create Key Vault (Premium Tier)

```bash
az keyvault create \
    --name cardano-vrf-kv \
    --resource-group cardano-vrf-rg \
    --location eastus \
    --sku premium \
    --enable-rbac-authorization true \
    --enable-soft-delete true \
    --retention-days 90 \
    --enable-purge-protection true \
    --tags Environment=production Compliance=FIPS-140-2
```

#### 3. Configure Authentication

**Option A: Managed Identity (Recommended)**

```bash
# Enable on VM
az vm identity assign \
    --resource-group cardano-vrf-rg \
    --name cardano-node-vm

# Get identity
IDENTITY_ID=$(az vm identity show \
    --resource-group cardano-vrf-rg \
    --name cardano-node-vm \
    --query principalId \
    --output tsv)

# Grant permissions
az role assignment create \
    --role "Key Vault Crypto User" \
    --assignee-object-id $IDENTITY_ID \
    --scope /subscriptions/$(az account show --query id -o tsv)/resourceGroups/cardano-vrf-rg/providers/Microsoft.KeyVault/vaults/cardano-vrf-kv
```

**Option B: Service Principal**

```bash
# Create service principal
az ad sp create-for-rbac \
    --name cardano-vrf-sp \
    --role "Key Vault Crypto User" \
    --scopes /subscriptions/$(az account show --query id -o tsv)/resourceGroups/cardano-vrf-rg

# Save the output:
# - appId (client_id)
# - password (client_secret)
# - tenant
```

#### 4. Configure Network Security

```bash
# Create private endpoint
az network private-endpoint create \
    --name kv-private-endpoint \
    --resource-group cardano-vrf-rg \
    --vnet-name cardano-vnet \
    --subnet cardano-subnet \
    --private-connection-resource-id $(az keyvault show \
        --name cardano-vrf-kv \
        --resource-group cardano-vrf-rg \
        --query id \
        --output tsv) \
    --group-id vault \
    --connection-name kv-connection

# Disable public access
az keyvault update \
    --name cardano-vrf-kv \
    --default-action Deny
```

#### 5. Enable Diagnostic Logging

```bash
# Create Log Analytics workspace
az monitor log-analytics workspace create \
    --resource-group cardano-vrf-rg \
    --workspace-name cardano-vrf-logs \
    --retention-time 90

# Get workspace ID
WORKSPACE_ID=$(az monitor log-analytics workspace show \
    --resource-group cardano-vrf-rg \
    --workspace-name cardano-vrf-logs \
    --query id \
    --output tsv)

# Enable diagnostics
az monitor diagnostic-settings create \
    --name kv-diagnostics \
    --resource $(az keyvault show \
        --name cardano-vrf-kv \
        --resource-group cardano-vrf-rg \
        --query id \
        --output tsv) \
    --workspace $WORKSPACE_ID \
    --logs '[
        {
            "category": "AuditEvent",
            "enabled": true,
            "retentionPolicy": {
                "enabled": true,
                "days": 90
            }
        }
    ]' \
    --metrics '[
        {
            "category": "AllMetrics",
            "enabled": true,
            "retentionPolicy": {
                "enabled": true,
                "days": 90
            }
        }
    ]'
```

### Security Hardening

#### RBAC Configuration

```bash
# Principle of least privilege
# Crypto User: For normal operations
az role assignment create \
    --role "Key Vault Crypto User" \
    --assignee-object-id $APP_IDENTITY \
    --scope $KEYVAULT_ID

# Crypto Officer: Only for key management
az role assignment create \
    --role "Key Vault Crypto Officer" \
    --assignee-object-id $ADMIN_IDENTITY \
    --scope $KEYVAULT_ID
```

#### Network Policies

```bash
# Allow specific IP only
az keyvault network-rule add \
    --name cardano-vrf-kv \
    --ip-address 203.0.113.10/32

# Or allow VNet only
az keyvault network-rule add \
    --name cardano-vrf-kv \
    --vnet-name cardano-vnet \
    --subnet cardano-subnet
```

### Monitoring & Alerts

```bash
# Alert on failed authentication
az monitor metrics alert create \
    --name kv-auth-failures \
    --resource-group cardano-vrf-rg \
    --scopes $KEYVAULT_ID \
    --condition "total ServiceApiResult where ResultType includes AuthenticationFailed > 5" \
    --window-size 5m \
    --evaluation-frequency 1m \
    --action $ACTION_GROUP_ID

# Alert on high latency
az monitor metrics alert create \
    --name kv-high-latency \
    --resource-group cardano-vrf-rg \
    --scopes $KEYVAULT_ID \
    --condition "avg ServiceApiLatency > 1000" \
    --window-size 5m \
    --evaluation-frequency 1m
```

### Cost Optimization

**Premium Tier Pricing:**
- Per key: $1.00/month
- Operations: $0.15 per 10,000

**Optimization Tips:**
1. Cache public keys (avoid repeated gets)
2. Use Standard tier for dev/test
3. Batch operations when possible
4. Clean up unused keys
5. Monitor usage with Azure Cost Management

**Example Monthly Cost:**
- 10 VRF keys: $10
- 1M operations: $15
- **Total: $25/month**

Much cheaper than CloudHSM ($2,304/month)!

---

## Security Hardening

### General Security Checklist

#### ✅ Credentials Management
- [ ] Never hardcode credentials in source code
- [ ] Use environment variables or secrets managers
- [ ] Rotate credentials every 90 days
- [ ] Use different credentials per environment
- [ ] Enable multi-factor authentication where possible

#### ✅ Network Security
- [ ] Use private endpoints/VPC
- [ ] Enable firewall rules
- [ ] Implement IP whitelisting
- [ ] Use TLS 1.2+ for all communications
- [ ] Enable network flow logs

#### ✅ Access Control
- [ ] Principle of least privilege
- [ ] Separate read/write permissions
- [ ] Use role-based access control (RBAC)
- [ ] Regular access reviews
- [ ] Audit all permission changes

#### ✅ Key Management
- [ ] Set keys as non-extractable
- [ ] Enable key deletion protection
- [ ] Implement key rotation policies
- [ ] Tag keys with metadata (env, purpose)
- [ ] Regular key inventory audits

#### ✅ Monitoring
- [ ] Enable audit logging
- [ ] Set up alerts for failures
- [ ] Monitor latency and errors
- [ ] Track key usage patterns
- [ ] Review logs weekly

#### ✅ Backup & DR
- [ ] Enable automatic backups
- [ ] Test restore procedures monthly
- [ ] Document disaster recovery plan
- [ ] Store backups in separate region
- [ ] Encrypt all backups

---

## Monitoring & Alerts

### Key Metrics to Track

#### Performance Metrics
```
- VRF proof generation latency (p50, p95, p99)
- Public key retrieval latency
- HSM health check response time
- Error rate (per operation type)
- Throughput (operations per second)
```

#### Security Metrics
```
- Failed authentication attempts
- Unauthorized access attempts
- Key access patterns
- Unusual operation volumes
- Geographic access anomalies
```

### Prometheus Integration

The library includes built-in Prometheus metrics:

```rust
use cardano_vrf::metrics::VrfMetrics;

let metrics = VrfMetrics::new();

// Metrics are automatically tracked:
// - vrf_prove_duration_seconds
// - vrf_verify_duration_seconds
// - vrf_prove_total
// - vrf_verify_total
// - vrf_prove_errors_total
// - vrf_verify_errors_total
```

### Example Alert Rules

```yaml
# Prometheus alert rules
groups:
  - name: vrf_hsm_alerts
    rules:
      - alert: HighVRFProveLatency
        expr: histogram_quantile(0.95, vrf_prove_duration_seconds) > 0.5
        for: 5m
        annotations:
          summary: "VRF prove latency above 500ms"

      - alert: HSMAuthenticationFailure
        expr: increase(vrf_prove_errors_total[5m]) > 10
        for: 1m
        annotations:
          summary: "HSM authentication failures detected"

      - alert: HSMUnhealthy
        expr: up{job="vrf_hsm"} == 0
        for: 2m
        annotations:
          summary: "HSM health check failing"
```

---

## Disaster Recovery

### Backup Strategy

#### Software HSM
```bash
# Backup keys
tar -czf vrf-keys-backup-$(date +%Y%m%d).tar.gz /path/to/keys/
gpg --encrypt --recipient admin@example.com vrf-keys-backup-*.tar.gz

# Store off-site
aws s3 cp vrf-keys-backup-*.tar.gz.gpg s3://backups/vrf/
```

#### AWS CloudHSM
```bash
# Automatic daily backups (enabled by default)
# Manual backup
aws cloudhsmv2 create-backup \
    --cluster-id cluster-abc123 \
    --tag Key=Purpose,Value=pre-migration

# List backups
aws cloudhsmv2 describe-backups \
    --filters clusterIds=cluster-abc123

# Restore to new cluster
aws cloudhsmv2 create-cluster --from-backup backup-xyz789
```

#### Azure Key Vault
```bash
# Export keys (requires "Key Vault Crypto Officer" role)
az keyvault key backup \
    --vault-name cardano-vrf-kv \
    --name validator-001 \
    --file validator-001-backup.blob

# Restore
az keyvault key restore \
    --vault-name cardano-vrf-kv \
    --file validator-001-backup.blob
```

### Recovery Procedures

#### HSM Failure Recovery

1. **Detection**
   ```bash
   # Monitor health check
   curl http://localhost:9090/health/hsm
   ```

2. **Failover** (AWS CloudHSM auto-fails over to replica)
   ```
   Primary HSM fails → Client automatically connects to replica
   ```

3. **Alert Team**
   ```
   Send PagerDuty alert to on-call engineer
   ```

4. **Restore Primary** (if needed)
   ```bash
   aws cloudhsmv2 create-hsm \
       --cluster-id cluster-abc123 \
       --availability-zone us-east-1a
   ```

#### Complete Site Failure

1. **Geographic Failover**
   - AWS: Restore cluster from backup in different region
   - Azure: Use geo-replicated secondary region

2. **Verify Data**
   ```bash
   # List all keys
   ./verify-keys.sh
   ```

3. **Resume Operations**
   ```bash
   # Update DNS to new region
   # Restart applications
   ```

---

## Compliance & Audit

### Compliance Requirements

#### FIPS 140-2 Level 3
- ✅ AWS CloudHSM (certified)
- ✅ Azure Key Vault Premium (certified)
- ✅ PKCS#11 HSMs (vendor-specific)
- ❌ Software HSM (not certified)

#### SOC 2 Type II
- Enable comprehensive audit logging
- Implement access reviews
- Document security procedures
- Regular penetration testing

#### PCI DSS
- Use HSM for key storage (requirement 3.5.3)
- Implement dual control for key management
- Log all key access (requirement 10)
- Regular security assessments

### Audit Logging

#### What to Log
```
- All key operations (create, use, delete)
- Authentication attempts (success and failure)
- Configuration changes
- Access to audit logs themselves
- System errors and exceptions
```

#### Log Retention
- **Production:** 90 days minimum (1 year recommended)
- **Development:** 30 days
- **Compliance:** Follow industry requirements (often 7 years)

#### Log Analysis
```bash
# AWS CloudWatch Insights query
fields @timestamp, @message
| filter @message like /vrf_prove/
| stats count() by bin(5m)

# Azure Log Analytics query
AzureDiagnostics
| where ResourceProvider == "MICROSOFT.KEYVAULT"
| where Category == "AuditEvent"
| where OperationName == "Sign"
| summarize Count=count() by bin(TimeGenerated, 5m)
```

---

## Production Deployment Checklist

### Pre-Deployment

- [ ] HSM hardware/service provisioned
- [ ] Network security configured
- [ ] Authentication configured
- [ ] Credentials stored securely
- [ ] Monitoring and alerts set up
- [ ] Backup strategy implemented
- [ ] Disaster recovery plan documented
- [ ] Security audit completed
- [ ] Load testing performed
- [ ] Documentation updated

### Deployment

- [ ] Deploy to staging first
- [ ] Verify all operations work
- [ ] Run integration tests
- [ ] Monitor for 24 hours
- [ ] Deploy to production
- [ ] Verify health checks
- [ ] Test failover procedures
- [ ] Document deployment

### Post-Deployment

- [ ] Monitor for 1 week
- [ ] Review audit logs
- [ ] Performance tuning
- [ ] Update runbooks
- [ ] Train operations team
- [ ] Schedule regular reviews

---

## Support & Resources

### Documentation
- [Rust API Documentation](https://docs.rs/cardano-vrf)
- [GitHub Repository](https://github.com/FractionEstate/Cardano-VRF)
- [IOHK VRF Specification](https://github.com/input-output-hk/vrf)

### HSM Vendor Documentation
- [AWS CloudHSM Docs](https://docs.aws.amazon.com/cloudhsm/)
- [Azure Key Vault Docs](https://docs.microsoft.com/en-us/azure/key-vault/)
- [PKCS#11 Specification](http://docs.oasis-open.org/pkcs11/pkcs11-base/v2.40/)
- [Thales nShield Docs](https://thalesdocs.com/gphsm/)

### Community
- GitHub Issues: Bug reports and feature requests
- GitHub Discussions: Questions and community support

---

**Last Updated:** November 2025
**Version:** 1.0.0
**Maintainer:** FractionEstate
