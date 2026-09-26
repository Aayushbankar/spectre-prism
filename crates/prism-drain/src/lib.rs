use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DrainError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Cluster capacity exceeded: {0}")]
    CapacityExceeded(String),
    #[error("Laya enrichment failed: {0}")]
    LayaEnrichment(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: String,
    pub pattern: Vec<String>,
    pub count: usize,
    pub samples: Vec<String>,
    pub specificity: f64,
    pub vendor_hint: Option<String>,
    pub class_uid_hint: Option<i64>,
    pub detector_terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichedTemplate {
    pub template_id: String,
    pub pattern: Vec<String>,
    pub count: usize,
    pub samples: Vec<String>,
    pub specificity: f64,
    pub class_uid: i64,
    pub category_uid: i64,
    pub vendor: String,
    pub confidence: f64,
    pub detector_terms: Vec<String>,
}

pub struct Drain {
    similarity_threshold: f64,
    max_clusters: usize,
    min_surviving_literals: usize,
    buckets: HashMap<usize, Vec<String>>,
    clusters: HashMap<String, Template>,
    next_id: usize,
    overflow: usize,
}

impl Default for Drain {
    fn default() -> Self {
        Self::new()
    }
}

impl Drain {
    pub fn new() -> Self {
        Self::with_capacity(1000)
    }

    pub fn with_capacity(max_clusters: usize) -> Self {
        Self {
            similarity_threshold: 0.4,
            max_clusters: max_clusters.max(1),
            min_surviving_literals: 2,
            buckets: HashMap::new(),
            clusters: HashMap::new(),
            next_id: 1,
            overflow: 0,
        }
    }

    pub fn process(&mut self, raw: &str) -> Option<String> {
        let tokens = tokenize(raw);
        if tokens.is_empty() {
            return None;
        }
        let bucket = self.buckets.entry(tokens.len()).or_default();

        let mut best: Option<(String, f64)> = None;
        for id in bucket.iter() {
            let Some(cluster) = self.clusters.get(id) else {
                continue;
            };
            let score = similarity(&tokens, &cluster.pattern);
            if score < self.similarity_threshold {
                continue;
            }
            let literals = cluster.pattern.iter().filter(|t| *t != "<*>").count();
            if literals >= self.min_surviving_literals {
                let surviving = cluster
                    .pattern
                    .iter()
                    .zip(tokens.iter())
                    .filter(|(slot, token)| *slot != "<*>" && slot == token)
                    .count();
                if surviving < self.min_surviving_literals {
                    continue;
                }
            }
            if best.as_ref().is_none_or(|(_, b)| score > *b) {
                best = Some((id.clone(), score));
            }
        }

        if let Some((id, _)) = best {
            let cluster = self.clusters.get_mut(&id).expect("id came from bucket index");
            cluster.count += 1;
            if cluster.samples.len() < 30 {
                cluster.samples.push(raw.to_string());
            }
            for (slot, token) in cluster.pattern.iter_mut().zip(tokens.iter()) {
                if slot != token && slot != "<*>" {
                    *slot = "<*>".to_string();
                }
            }
            return Some(id);
        }

        if self.clusters.len() >= self.max_clusters {
            self.overflow += 1;
            return None;
        }

        let id = format!("C{:04}", self.next_id);
        self.next_id += 1;
        self.buckets.entry(tokens.len()).or_default().push(id.clone());
        self.clusters.insert(
            id.clone(),
            Template {
                id: id.clone(),
                pattern: tokens,
                count: 1,
                samples: vec![raw.to_string()],
                specificity: 1.0,
                vendor_hint: None,
                class_uid_hint: None,
                detector_terms: Vec::new(),
            },
        );
        Some(id)
    }

    pub fn process_batch(&mut self, logs: &[String]) -> Vec<Option<String>> {
        logs.iter().map(|log| self.process(log)).collect()
    }

    pub fn overflow(&self) -> usize {
        self.overflow
    }

    pub fn len(&self) -> usize {
        self.clusters.len()
    }

    pub fn is_empty(&self) -> bool {
        self.clusters.is_empty()
    }

    pub fn ranked_templates(&self) -> Vec<Template> {
        let mut sorted: Vec<Template> = self.clusters.values().cloned().collect();
        sorted.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.id.cmp(&b.id)));
        sorted
    }

    pub async fn enrich_with_laya(&mut self) -> Result<Vec<EnrichedTemplate>, DrainError> {
        let templates = self.ranked_templates();
        enrich_with_laya(templates).await
    }
}

fn tokenize(raw: &str) -> Vec<String> {
    raw.split(|c: char| c.is_whitespace() || matches!(c, '|' | ',' | ';' | '\t'))
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}

fn similarity(tokens: &[String], template: &[String]) -> f64 {
    if tokens.is_empty() {
        return 0.0;
    }
    let matched = tokens
        .iter()
        .zip(template.iter())
        .filter(|(t, slot)| t == slot || slot.as_str() == "<*>")
        .count();
    matched as f64 / tokens.len() as f64
}

pub fn specificity(template: &Template) -> f64 {
    if template.pattern.is_empty() {
        return 0.0;
    }
    let literal = template.pattern.iter().filter(|t| *t != "<*>").count();
    literal as f64 / template.pattern.len() as f64
}

async fn enrich_with_laya(templates: Vec<Template>) -> Result<Vec<EnrichedTemplate>, DrainError> {
    let input = serde_json::to_string(&templates).map_err(|e| DrainError::LayaEnrichment(e.to_string()))?;

    let output = tokio::process::Command::new("python3")
        .arg("prism-brain/laya_enricher.py")
        .arg("--input")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| DrainError::LayaEnrichment(e.to_string()))?
        .stdin
        .write_all(input.as_bytes())
        .await
        .map_err(|e| DrainError::LayaEnrichment(e.to_string()))?
        .wait_with_output()
        .await
        .map_err(|e| DrainError::LayaEnrichment(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DrainError::LayaEnrichment(stderr.to_string()));
    }

    let enriched: Vec<EnrichedTemplate> = serde_json::from_slice(&output.stdout)
        .map_err(|e| DrainError::LayaEnrichment(e.to_string()))?;

    for enriched in &enriched {
        if let Some(template) = templates.iter_mut().find(|t| t.id == enriched.template_id) {
            template.class_uid_hint = Some(enriched.class_uid);
            template.vendor_hint = Some(enriched.vendor.clone());
            template.detector_terms = enriched.detector_terms.clone();
        }
    }

    Ok(enriched)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_lines_share_one_cluster() {
        let mut d = Drain::new();
        let line = "kernel: INBOUND TCP SRC=1.2.3.4 DPT=445";
        let a = d.process(line).unwrap();
        let b = d.process(line).unwrap();
        assert_eq!(a, b);
        assert_eq!(d.len(), 1);
        assert_eq!(d.ranked_templates()[0].count, 2);
    }

    #[test]
    fn varying_positions_generalise_to_wildcard() {
        let mut d = Drain::new();
        d.process("kernel: INBOUND TCP SRC=1.2.3.4 DPT=445");
        d.process("kernel: INBOUND TCP SRC=5.6.7.8 DPT=445");
        let template = &d.ranked_templates()[0].pattern;
        assert_eq!(template[3], "<*>");
        assert_eq!(template[4], "DPT=445");
    }

    #[test]
    fn different_token_counts_never_share_cluster() {
        let mut d = Drain::new();
        d.process("a b c").unwrap();
        d.process("a b c d").unwrap();
        assert_eq!(d.len(), 2);
    }

    #[test]
    fn cluster_cap_enforced() {
        let mut d = Drain::with_capacity(3);
        for i in 0..50 {
            d.process(&"x ".repeat(i + 1));
        }
        assert_eq!(d.len(), 3);
        assert!(d.overflow() > 0);
    }

    #[test]
    fn specificity_calculation() {
        let mut d = Drain::new();
        d.process("kernel: INBOUND TCP SRC=1.2.3.4 DPT=445");
        d.process("kernel: INBOUND TCP SRC=5.6.7.8 DPT=445");
        let spec = specificity(&d.ranked_templates()[0]);
        assert_eq!(spec, 0.8);
    }
}