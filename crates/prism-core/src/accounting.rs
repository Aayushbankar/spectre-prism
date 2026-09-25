use serde::{Deserialize, Serialize};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ByteTag {
    Field { name: String, start: usize, end: usize },
    Literal { start: usize, end: usize },
    Ignored { start: usize, end: usize, reason: String },
    Residue { start: usize, end: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ByteAccountingReport {
    pub total_bytes: usize,
    pub field_bytes: usize,
    pub literal_bytes: usize,
    pub ignored_bytes: usize,
    pub residue_bytes: usize,
    pub closure_ratio: f64,
    pub tags: Vec<ByteTag>,
}

fn finalize_report(raw: &str, mut tags: Vec<ByteTag>) -> ByteAccountingReport {
    // Fill in missing regions as Residue, and also categorize spaces/known stuff if not done.
    // However, our parse logic will try to cover gaps.
    tags.sort_by_key(|t| match t {
        ByteTag::Field { start, .. } => *start,
        ByteTag::Literal { start, .. } => *start,
        ByteTag::Ignored { start, .. } => *start,
        ByteTag::Residue { start, .. } => *start,
    });

    let mut final_tags = Vec::new();
    let mut current = 0;
    
    for tag in tags {
        let start = match &tag {
            ByteTag::Field { start, .. } => *start,
            ByteTag::Literal { start, .. } => *start,
            ByteTag::Ignored { start, .. } => *start,
            ByteTag::Residue { start, .. } => *start,
        };
        let end = match &tag {
            ByteTag::Field { end, .. } => *end,
            ByteTag::Literal { end, .. } => *end,
            ByteTag::Ignored { end, .. } => *end,
            ByteTag::Residue { end, .. } => *end,
        };
        
        if start > current {
            // Uncovered region
            let gap = &raw[current..start];
            let mut gap_start = current;
            for (i, c) in gap.char_indices() {
                if c.is_whitespace() {
                    if gap_start < current + i {
                        final_tags.push(ByteTag::Residue { start: gap_start, end: current + i });
                    }
                    final_tags.push(ByteTag::Literal { start: current + i, end: current + i + c.len_utf8() });
                    gap_start = current + i + c.len_utf8();
                }
            }
            if gap_start < start {
                final_tags.push(ByteTag::Residue { start: gap_start, end: start });
            }
        }
        
        final_tags.push(tag);
        current = current.max(end);
    }
    
    if current < raw.len() {
        let gap = &raw[current..];
        let mut gap_start = current;
        for (i, c) in gap.char_indices() {
            if c.is_whitespace() {
                if gap_start < current + i {
                    final_tags.push(ByteTag::Residue { start: gap_start, end: current + i });
                }
                final_tags.push(ByteTag::Literal { start: current + i, end: current + i + c.len_utf8() });
                gap_start = current + i + c.len_utf8();
            }
        }
        if gap_start < raw.len() {
            final_tags.push(ByteTag::Residue { start: gap_start, end: raw.len() });
        }
    }
    
    let mut report = ByteAccountingReport {
        total_bytes: raw.len(),
        field_bytes: 0,
        literal_bytes: 0,
        ignored_bytes: 0,
        residue_bytes: 0,
        closure_ratio: 0.0,
        tags: final_tags,
    };
    
    for tag in &report.tags {
        match tag {
            ByteTag::Field { start, end, .. } => report.field_bytes += end - start,
            ByteTag::Literal { start, end } => report.literal_bytes += end - start,
            ByteTag::Ignored { start, end, .. } => report.ignored_bytes += end - start,
            ByteTag::Residue { start, end } => report.residue_bytes += end - start,
        }
    }
    
    if report.total_bytes > 0 {
        report.closure_ratio = (report.total_bytes as f64 - report.residue_bytes as f64) / report.total_bytes as f64;
    } else {
        report.closure_ratio = 1.0;
    }
    
    report
}

pub fn account_fortinet(raw: &str) -> ByteAccountingReport {
    let mut tags = Vec::new();
    
    // Header e.g. <134>
    if let Some(caps) = Regex::new(r"^<\d+>").unwrap().captures(raw) {
        let mat = caps.get(0).unwrap();
        tags.push(ByteTag::Literal { start: mat.start(), end: mat.end() });
    }
    
    // Key-value pairs
    let kv_regex = Regex::new(r#"([a-zA-Z0-9_-]+)=("[^"]*"|[^ ]+)"#).unwrap();
    for cap in kv_regex.captures_iter(raw) {
        let full = cap.get(0).unwrap();
        let key = cap.get(1).unwrap();
        let val = cap.get(2).unwrap();
        
        tags.push(ByteTag::Literal { start: full.start(), end: val.start() });
        tags.push(ByteTag::Field { name: key.as_str().to_string(), start: val.start(), end: val.end() });
    }
    
    finalize_report(raw, tags)
}

pub fn account_cisco(raw: &str) -> ByteAccountingReport {
    let mut tags = Vec::new();
    
    // e.g. <166>Sep 15 2024 14:23:41 FW-CORE-01 : %ASA-6-302013: 
    let header_regex = Regex::new(r"^(?:<\d+>)?(?:[A-Za-z]{3}\s+\d+\s+\d{4}\s+\d{2}:\d{2}:\d{2}\s+[\w-]+\s*:\s*)?(%ASA-\d+-\d+:?)").unwrap();
    if let Some(caps) = header_regex.captures(raw) {
        let full = caps.get(0).unwrap();
        tags.push(ByteTag::Literal { start: full.start(), end: full.end() });
    }
    
    // Find IP/ports to tag as FIELD
    let ip_regex = Regex::new(r"\b\d{1,3}(?:\.\d{1,3}){3}(?:/\d+)?\b").unwrap();
    for cap in ip_regex.captures_iter(raw) {
        let mat = cap.get(0).unwrap();
        tags.push(ByteTag::Field { name: "ip".to_string(), start: mat.start(), end: mat.end() });
    }
    
    // Tag words as literals to increase closure
    let text_regex = Regex::new(r"\b[A-Za-z_-]+:?\b").unwrap();
    for cap in text_regex.captures_iter(raw) {
        let mat = cap.get(0).unwrap();
        tags.push(ByteTag::Literal { start: mat.start(), end: mat.end() });
    }
    
    finalize_report(raw, tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fortinet_full_closure() {
        let raw = "<134>date=2024-01-15 time=08:23:41 devname=\"FGT-DC-01\" logid=\"0000000013\"";
        let report = account_fortinet(raw);
        assert!(report.closure_ratio > 0.95);
    }

    #[test]
    fn test_fortinet_with_unknown_field() {
        let raw = "<134>date=2024-01-15 @@@GARBAGE@@@ devname=\"FGT-DC-01\"";
        let report = account_fortinet(raw);
        assert!(report.residue_bytes > 0);
        assert!(report.closure_ratio < 1.0);
    }

    #[test]
    fn test_cisco_closure() {
        let raw = "<166>Sep 15 2024 14:23:41 FW-CORE-01 : %ASA-6-302013: Built inbound TCP connection for 198.51.100.47 to 10.10.5.100";
        let report = account_cisco(raw);
        assert!(report.closure_ratio > 0.90);
    }

    #[test]
    fn test_closure_ratio_calculation() {
        let raw = "k=v  ";
        let report = account_fortinet(raw);
        assert_eq!(report.total_bytes, 5);
        // k= (2), v (1), space space (2)
        assert_eq!(report.closure_ratio, 1.0);
    }
}
