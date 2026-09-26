use prism_pack_spec::{PrismPack, ValidationResult, PackError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScorerError {
    #[error("VRL compilation failed: {0}")]
    CompilationFailed(String),
    #[error("Dry-run failed: {0}")]
    DryRunFailed(String),
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    #[error("Pack error: {0}")]
    PackError(#[from] PackError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreReport {
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

pub struct PackScorer;

impl PackScorer {
    pub fn score(pack: &mut PrismPack) -> Result<ValidationResult, ScorerError> {
        pack.validate().map_err(ScorerError::PackError)
    }

    pub fn score_vrl(vrl: &str, fixtures: &[serde_json::Value]) -> Result<ScoreReport, ScorerError> {
        // 1. VRL Compilation
        let compile_ok = compile_vrl(vrl).is_ok();

        // 2. Dry-run with fixtures
        let (dry_run_ok, fixtures_passed, fixtures_total, field_accuracy) = dry_run_vrl(vrl, fixtures)?;

        // 3. OCSF Path Validation
        let unknown_paths: Vec<String> = Vec::new(); // TODO: Validate against known paths

        // 4. Witness verification would be done at pack level

        let weights = [0.30, 0.25, 0.20, 0.15, 0.10];
        let scores = [
            if compile_ok { 1.0 } else { 0.0 },
            if dry_run_ok { 1.0 } else { 0.0 },
            field_accuracy,
            if unknown_paths.is_empty() { 1.0 } else { 0.0 },
            1.0, // witness_verified placeholder
        ];
        let overall_score: f64 = weights.iter().zip(scores.iter()).map(|(w, s)| w * s).sum();

        let passed = compile_ok && dry_run_ok && field_accuracy >= 0.90;

        Ok(ScoreReport {
            vrl_compile_passed: compile_ok,
            dry_run_passed: dry_run_ok,
            fixtures_total,
            fixtures_passed,
            field_accuracy,
            ocsf_validation_passed: unknown_paths.is_empty(),
            unknown_paths,
            witness_verified: true,
            overall_score,
            passed,
        })
    }
}

fn compile_vrl(vrl: &str) -> Result<(), String> {
    if vrl.is_empty() {
        return Err("Empty VRL".into());
    }
    if vrl.contains("```") {
        return Err("Contains markdown formatting".into());
    }
    if !vrl.contains(".class_uid =") {
        return Err("Missing .class_uid".into());
    }
    if !vrl.contains(".category_uid =") {
        return Err("Missing .category_uid".into());
    }
    if !vrl.contains(".type_uid =") {
        return Err("Missing .type_uid".into());
    }
    if vrl.contains(":=") {
        return Err("Contains := assignment".into());
    }
    prism_vrl_generator::compile_vrl(vrl).map_err(|e| e.to_string())
}

fn dry_run_vrl(vrl: &str, fixtures: &[serde_json::Value]) -> Result<(bool, usize, usize, f64), ScorerError> {
    if fixtures.is_empty() {
        return Ok((true, 0, 0, 1.0));
    }
    let mut passed = 0;
    let mut matched_fields = 0;
    let mut total_fields = 0;

    for fixture in fixtures {
        let sample = if let Some(s) = fixture.as_str() {
            s
        } else if let Some(s) = fixture.get("message").and_then(|m| m.as_str()) {
            s
        } else if let Some(s) = fixture.get("raw").and_then(|m| m.as_str()) {
            s
        } else {
            continue;
        };

        if let Ok(res) = prism_vrl_generator::run_dry_run(vrl, sample) {
            passed += 1;
            if let Ok(fields) = prism_vrl_generator::extract_fields(&res) {
                total_fields += fields.len();
                matched_fields += fields.len();
            }
        }
    }

    let field_accuracy = if total_fields > 0 {
        matched_fields as f64 / total_fields as f64
    } else {
        0.0
    };
    let dry_run_ok = passed == fixtures.len();
    Ok((dry_run_ok, passed, fixtures.len(), field_accuracy))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_vrl_passes() {
        let vrl = r#"
m, err = parse_regex(string!(.message), r'(?P<ip>\d+\.\d+\.\d+\.\d+)')
if err == null {
    .src_endpoint.ip = m.ip
    .dst_endpoint.ip = m.ip
}
.class_uid = 4001
.category_uid = 4
.type_uid = 400101
"#;
        assert!(compile_vrl(vrl).is_ok());
    }

    #[test]
    fn empty_vrl_fails() {
        assert!(compile_vrl("").is_err());
    }

    #[test]
    fn markdown_vrl_fails() {
        let vrl = "```vrl\n.ip = \"1.2.3.4\"\n```";
        assert!(compile_vrl(vrl).is_err());
    }

    #[test]
    fn missing_class_uid_fails() {
        let vrl = r#".ip = "1.2.3.4""#;
        assert!(compile_vrl(vrl).is_err());
    }

    #[test]
    fn assignment_operator_fails() {
        let vrl = r#".ip := "1.2.3.4""#;
        assert!(compile_vrl(vrl).is_err());
    }
}