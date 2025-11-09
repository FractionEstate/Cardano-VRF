//! Test vector parser for Cardano VRF test files
//!
//! Parses the format used in cardano-base-rust test vectors

use std::collections::HashMap;

/// Parsed VRF test vector
#[derive(Debug, Clone)]
pub struct TestVector {
    pub vrf_algorithm: String,
    pub version: String,
    pub ciphersuite: Option<String>,
    pub sk: Vec<u8>,
    pub pk: Vec<u8>,
    pub alpha: Vec<u8>,
    pub pi: Vec<u8>,
    pub beta: Vec<u8>,
}

/// Parse a test vector file content
pub fn parse_test_vector(content: &str) -> Result<TestVector, String> {
    let mut fields = HashMap::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim();
            let value = line[pos + 1..].trim();
            fields.insert(key.to_string(), value.to_string());
        }
    }

    let vrf_algorithm = fields.get("vrf")
        .ok_or("Missing 'vrf' field")?
        .clone();

    let version = fields.get("ver")
        .ok_or("Missing 'ver' field")?
        .clone();

    let ciphersuite = fields.get("ciphersuite").cloned();

    let sk = parse_hex_field(&fields, "sk")?;
    let pk = parse_hex_field(&fields, "pk")?;
    let alpha = parse_alpha_field(&fields)?;
    let pi = parse_hex_field(&fields, "pi")?;
    let beta = parse_hex_field(&fields, "beta")?;

    Ok(TestVector {
        vrf_algorithm,
        version,
        ciphersuite,
        sk,
        pk,
        alpha,
        pi,
        beta,
    })
}

fn parse_hex_field(fields: &HashMap<String, String>, key: &str) -> Result<Vec<u8>, String> {
    let value = fields.get(key)
        .ok_or_else(|| format!("Missing '{}' field", key))?;

    hex::decode(value)
        .map_err(|e| format!("Invalid hex in '{}': {}", key, e))
}

fn parse_alpha_field(fields: &HashMap<String, String>) -> Result<Vec<u8>, String> {
    let value = fields.get("alpha")
        .ok_or("Missing 'alpha' field")?;

    if value == "empty" {
        Ok(Vec::new())
    } else {
        hex::decode(value)
            .map_err(|e| format!("Invalid hex in 'alpha': {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_vector() {
        let content = r#"
vrf: PraosVRF
ver: ietfdraft03
ciphersuite: ECVRF-ED25519-SHA512-Elligator2
sk: 9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60
pk: d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a
alpha: empty
pi: b6b4699f87d56126c9117a7da55bd0085246f4c56dbc95d20172612e9d38e8d7ca65e573a126ed88d4e30a46f80a666854d675cf3ba81de0de043c3774f061560f55edc256a787afe701677c0f602900
beta: 5b49b554d05c0cd5a5325376b3387de59d924fd1e13ded44648ab33c21349a603f25b84ec5ed887995b33da5e3bfcb87cd2f64521c4c62cf825cffabbe5d31cc
"#;

        let vector = parse_test_vector(content).unwrap();

        assert_eq!(vector.vrf_algorithm, "PraosVRF");
        assert_eq!(vector.version, "ietfdraft03");
        assert_eq!(vector.sk.len(), 32);
        assert_eq!(vector.pk.len(), 32);
        assert_eq!(vector.alpha.len(), 0); // empty
        assert_eq!(vector.pi.len(), 80);
        assert_eq!(vector.beta.len(), 64);
    }
}
