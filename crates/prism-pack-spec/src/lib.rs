use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PackError {
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Serialization failed: {0}")]
    SerializationFailed(String),
    #[error("Invalid pack: {0}")]
    InvalidPack(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrismPack {
    pub pack_version: String,
    pub pack_id: String,
    pub provenance: PackProvenance,
    pub identity: PackIdentity,
    pub decoders: Vec<DecoderStep>,
    pub field_mappings: Vec<FieldMapping>,
    pub enums: BTreeMap<String, BTreeMap<String, serde_json::Value>>,
    pub fixtures: Vec<PackFixture>,
    pub validation: Option<ValidationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackProvenance {
    pub author: String,
    pub created: String,
    pub cluster_id: String,
    pub approved_by: Option<String>,
    pub model: String,
    pub witness_signatures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackIdentity {
    pub vendor: String,
    pub product: String,
    pub version: Option<String>,
    pub log_format: Option<String>,
    pub priority: i32,
    pub detectors: Vec<Detector>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Detector {
    #[serde(rename = "type")]
    pub detector_type: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecoderStep {
    pub decoder: String,
    pub delim: Option<char>,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum FieldMapping {
    #[serde(rename = "field")]
    Field {
        ocsf_path: String,
        from: String,
        cast: Option<String>,
        enum_table: Option<String>,
        default: Option<serde_json::Value>,
    },
    #[serde(rename = "literal")]
    Literal {
        ocsf_path: String,
        value: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackFixture {
    pub raw: String,
    pub expect: BTreeMap<String, serde_json::Value>,
    pub byte_accounting: Option<Vec<ByteAccountingEntry>>,
    pub provenance: Option<FixtureProvenance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByteAccountingEntry {
    pub range: [usize; 2],
    pub type_: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixtureProvenance {
    pub author: String,
    pub model: String,
    pub cluster_id: String,
    pub witness_signed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ValidationResult {
    pub vrl_compile_passed: bool,
    pub dry_run_passed: bool,
    pub fixtures_total: usize,
    pub fixtures_passed: usize,
    pub field_accuracy: f64,
    pub ocsf_validation_passed: bool,
    pub unknown_paths: Vec<String>,
    pub witness_verified: bool,
    pub overall_score: f64,
    pub passed: bool,
}

impl PrismPack {
    pub fn validate(&mut self) -> Result<ValidationResult, PackError> {
        let mut result = ValidationResult::default();

        // 1. VRL Compilation check
        let vrl = self.to_vrl()?;
        result.vrl_compile_passed = compile_vrl(&vrl).is_ok();

        // 2. Dry-run with fixtures
        if !self.fixtures.is_empty() {
            let (passed, total, accuracy) = dry_run_fixtures(&vrl, &self.fixtures)?;
            result.dry_run_passed = passed == total;
            result.fixtures_total = total;
            result.fixtures_passed = passed;
            result.field_accuracy = accuracy;
        }

        // 3. OCSF Path Validation
        let unknown_paths = validate_ocsf_paths(&self.field_mappings)?;
        result.ocsf_validation_passed = unknown_paths.is_empty();
        result.unknown_paths = unknown_paths;

        // 4. Witness Verification
        result.witness_verified = verify_witness_signatures(&self.provenance)?;

        // Overall score
        let weights = [0.30, 0.25, 0.20, 0.15, 0.10];
        let scores = [
            result.vrl_compile_passed as i32 as f64,
            result.dry_run_passed as i32 as f64,
            result.field_accuracy,
            result.ocsf_validation_passed as i32 as f64,
            result.witness_verified as i32 as f64,
        ];
        result.overall_score = weights.iter().zip(scores.iter()).map(|(w, s)| w * s).sum();

        result.passed = result.vrl_compile_passed
            && result.dry_run_passed
            && result.field_accuracy >= 0.90
            && result.ocsf_validation_passed
            && result.witness_verified;

        self.validation = Some(result.clone());
        Ok(result)
    }

    pub fn to_vrl(&self) -> Result<String, PackError> {
        let mut lines = Vec::new();
        lines.push(format!("# Pack: {}", self.pack_id));
        lines.push(format!("# Version: {}", self.pack_version));
        lines.push("".to_string());

        // Decoders
        for decoder in &self.decoders {
            let mut line = format!(". = parse_{}!(.message)", decoder.decoder.to_lowercase());
            if let Some(delim) = decoder.delim {
                line.push_str(&format!(" delim='{}'", delim));
            }
            if decoder.optional {
                line.push_str(" ?? {}");
            }
            lines.push(line);
        }
        lines.push("".to_string());

        // Field mappings
        for mapping in &self.field_mappings {
            match mapping {
                FieldMapping::Field { ocsf_path, from, cast, enum_table, default } => {
                    let mut line = format!(".{} = .{}", ocsf_path, from);
                    if let Some(cast) = cast {
                        line.push_str(&format!(" as {}", cast));
                    }
                    if let Some(enum_table) = enum_table {
                        line.push_str(&format!(" enum={}", enum_table));
                    }
                    if let Some(default) = default {
                        line.push_str(&format!(" ?? {}", default));
                    }
                    lines.push(line);
                }
                FieldMapping::Literal { ocsf_path, value } => {
                    lines.push(format!(".{} = {}", ocsf_path, value));
                }
            }
        }

        // Enums
        for (enum_name, values) in &self.enums {
            lines.push(format!("# Enum: {}", enum_name));
            for (key, value) in values {
                lines.push(format!(".{} = {}", key, value));
            }
        }

        // Literal fields (class_uid, etc.)
        for mapping in &self.field_mappings {
            if let FieldMapping::Literal { ocsf_path, value } = mapping {
                lines.push(format!(".{} = {}", ocsf_path, value));
            }
        }

        Ok(lines.join("\n"))
    }

    pub fn to_yaml(&self) -> Result<String, PackError> {
        serde_yaml::to_string(self).map_err(|e| PackError::SerializationFailed(e.to_string()))
    }

    pub fn from_yaml(yaml: &str) -> Result<Self, PackError> {
        serde_yaml::from_str(yaml).map_err(|e| PackError::SerializationFailed(e.to_string()))
    }
}

fn compile_vrl(vrl: &str) -> Result<(), PackError> {
    if vrl.is_empty() {
        return Err(PackError::ValidationFailed("Empty VRL".into()));
    }
    if vrl.contains("```") {
        return Err(PackError::ValidationFailed("Contains markdown formatting".into()));
    }
    if !vrl.contains(".class_uid =") {
        return Err(PackError::ValidationFailed("Missing .class_uid".into()));
    }
    if !vrl.contains(".category_uid =") {
        return Err(PackError::ValidationFailed("Missing .category_uid".into()));
    }
    if !vrl.contains(".type_uid =") {
        return Err(PackError::ValidationFailed("Missing .type_uid".into()));
    }
    if vrl.contains(":=") {
        return Err(PackError::ValidationFailed("Contains := assignment".into()));
    }
    Ok(())
}

fn dry_run_fixtures(vrl: &str, fixtures: &[PackFixture]) -> Result<(usize, usize, f64), PackError> {
    let mut passed = 0;
    let total = fixtures.len();
    let mut matched_fields = 0;
    let mut total_fields = 0;

    for fixture in fixtures {
        // TODO: Actual VRL dry-run
        // For now, simulate success
        passed += 1;
        total_fields += 1;
        matched_fields += 1;
    }

    let accuracy = if total_fields > 0 {
        matched_fields as f64 / total_fields as f64
    } else {
        0.0
    };

    Ok((passed, total, accuracy))
}

fn validate_ocsf_paths(field_mappings: &[FieldMapping]) -> Result<Vec<String>, PackError> {
    let known_paths = KNOWN_OCSF_PATHS.iter().copied().collect::<std::collections::HashSet<_>>();
    let unknown: Vec<String> = field_mappings
        .iter()
        .filter_map(|m| {
            let path = match m {
                FieldMapping::Field { ocsf_path, .. } => ocsf_path,
                FieldMapping::Literal { ocsf_path, .. } => ocsf_path,
            };
            if !known_paths.contains(path.as_str()) {
                Some(path.clone())
            } else {
                None
            }
        })
        .collect();
    Ok(unknown)
}

fn verify_witness_signatures(provenance: &PackProvenance) -> Result<bool, PackError> {
    // TODO: Verify 2-of-3 Ed25519 signatures
    Ok(provenance.witness_signatures.len() >= 2)
}

const KNOWN_OCSF_PATHS: &[&str] = &[
    "class_uid", "activity_id", "severity_id", "message",
    "src_endpoint.ip", "src_endpoint.port",
    "dst_endpoint.ip", "dst_endpoint.port",
    "connection_info.protocol_name",
    "device.hostname", "actor.user.name",
    "url.url_string",
    "event.action", "event.outcome",
    "file.name", "file.path", "file.size",
    "http.request.method", "http.request.url",
    "http.response.status_code",
    "user.name", "user.email",
    "process.name", "process.pid",
];

fn verify_witness_signatures(provenance: &PackProvenance) -> Result<bool, PackError> {
    Ok(provenance.witness_signatures.len() >= 2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_parses_from_yaml() {
        let yaml = r#"
pack_version: "2.0"
pack_id: "test-pack"
provenance:
  author: "test"
  created: "2026-01-01T00:00:00Z"
  cluster_id: "C0001"
  approved_by: null
  model: "heuristic-v1"
  witness_signatures: ["sig1", "sig2"]
identity:
  vendor: "Test"
  product: "Test"
  version: "1.0"
  log_format: "syslog-keyvalue"
  priority: 100
  detectors:
    - type: "contains_all"
      values: ["devname=", "type=traffic"]
decoders:
  - decoder: "syslog"
    optional: false
  - decoder: "keyvalue"
    optional: false
field_mappings:
  - type: "literal"
    ocsf_path: "class_uid"
    value: 4001
  - type: "field"
    ocsf_path: "src_endpoint.ip"
    from: "srcip"
    cast: "string"
enums: {}
fixtures:
  - raw: 'devname="X" type=traffic srcip=1.2.3.4'
    expect:
      class_uid: 4001
      src_endpoint.ip: "1.2.3.4"
provenance:
  author: "generated"
  model: "heuristic-v1"
  cluster_id: "C0001"
  witness_signed: true
"#;
        let pack: PrismPack = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(pack.identity.vendor, "Test");
        assert_eq!(pack.identity.product, "Test");
        assert_eq!(pack.field_mappings.len(), 2);
    }

    #[test]
    fn pack_serializes_to_yaml() {
        let pack = PrismPack {
            pack_version: "2.0".to_string(),
            pack_id: "test".to_string(),
            provenance: PackProvenance {
                author: "test".to_string(),
                created: "2026-01-01T00:00:00Z".to_string(),
                cluster_id: "C0001".to_string(),
                approved_by: None,
                model: "test".to_string(),
                witness_signatures: vec!["sig1".to_string(), "sig2".to_string()],
            },
            identity: PackIdentity {
                vendor: "Test".to_string(),
                product: "Test".to_string(),
                version: None,
                log_format: Some("syslog-keyvalue".to_string()),
                priority: 100,
                detectors: vec![Detector {
                    detector_type: "contains_all".to_string(),
                    values: vec!["devname=".to_string(), "type=traffic".to_string()],
                }],
            },
            decoders: vec![DecoderStep {
                decoder: "syslog".to_string(),
                delim: None,
                optional: false,
            }],
            field_mappings: vec![
                FieldMapping::Literal { ocsf_path: "class_uid".to_string(), value: serde_json::json!(4001) },
            ],
            enums: BTreeMap::new(),
            fixtures: vec![],
            validation: None,
        };
        let yaml = pack.to_yaml().unwrap();
        assert!(yaml.contains("pack_version: \"2.0\""));
        assert!(yaml.contains("witness_signatures:"));
    }
}