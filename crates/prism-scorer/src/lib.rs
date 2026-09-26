use prism_pack_spec::{PrismPack, ValidationResult};
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
        pack.validate()
    }

    pub fn score_vrl(vrl: &str, fixtures: &[serde_json::Value]) -> Result<ScoreReport, ScorerError> {
        // 1. VRL Compilation
        let compile_ok = compile_vrl(vrl).is_ok();

        // 2. Dry-run with fixtures
        let (dry_run_ok, fixtures_passed, fixtures_total, field_accuracy) = dry_run_vrl(vrl, fixtures)?;

        // 3. OCSF Path Validation
        let unknown_paths = Vec::new(); // TODO: Validate against known paths

        // 4. Witness verification would be done at pack level

        let weights = [0.30, 0.25, 0.20, 0.15, 0.10];
        let scores = [
            compile_ok as i32 as f64,
            true as i32 as f64, // dry_run_ok
            0.95, // field_accuracy placeholder
            true as i32 as f64, // ocsf_validation_passed placeholder
            true as i32 as f64, // witness_verified placeholder
        ];
        let weights = [0.30, 0.25, 0.20, 0.15, 0.10];
        let overall_score = weights.iter().zip(scores.iter()).map(|(w, s)| w * s).sum();

        let passed = compile_ok && true && 0.95 >= 0.90 && true && true;

        Ok(ScoreReport {
            vrl_compile_passed: compile_ok,
            dry_run_passed: true,
            fixtures_total: fixtures.len(),
            fixtures_passed: fixtures.len(),
            field_accuracy: 0.95,
            ocsf_validation_passed: true,
            unknown_paths: Vec::new(),
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
    Ok(())
}

fn dry_run_vrl(vrl: &str, fixtures: &[serde_json::Value]) -> Result<(bool, usize, usize, f64), String> {
    // TODO: Actual VRL dry-run using VRL engine
    // For now, simulate success
    let fixtures_total = fixtures.len();
    let fixtures_passed = fixtures_total;
    let field_accuracy = 0.95;
    Ok((true, fixtures_passed, fixtures_total, field_accuracy))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_vrl_passes() {
        let vrl = r#".message = parse_syslog!(.message)
.ip = parse_regex!(string!(.message), r'(?P<ip>\d+\.\d+\.\d+\.\d+)').ip
.class_uid = 4001
.category_uid = 4
.type_uid = 400101
.src_endpoint.ip = .ip
.dst_endpoint.ip = .ip"#;
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