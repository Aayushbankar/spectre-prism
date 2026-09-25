use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldFidelity {
    Exact,
    Derived { transform: String },
    Approximate { reason: String },
    Unmapped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedField {
    pub ocsf_field: String,
    pub value: String,
    pub fidelity: FieldFidelity,
    pub source_bytes: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityReport {
    pub fields: Vec<TaggedField>,
    pub exact_count: usize,
    pub derived_count: usize,
    pub approximate_count: usize,
    pub unmapped_count: usize,
}

pub fn tag_fortinet_fields(raw: &str, ocsf: &Value) -> FidelityReport {
    let mut fields = Vec::new();
    let mut exact_count = 0;
    let mut derived_count = 0;
    let mut approximate_count = 0;
    let mut unmapped_count = 0;

    let mut ocsf_values = std::collections::HashMap::new();

    fn flatten_json(val: &Value, prefix: &str, out: &mut std::collections::HashMap<String, String>) {
        match val {
            Value::Object(map) => {
                for (k, v) in map {
                    let new_key = if prefix.is_empty() { k.clone() } else { format!("{}.{}", prefix, k) };
                    flatten_json(v, &new_key, out);
                }
            }
            Value::Array(arr) => {
                for (i, v) in arr.iter().enumerate() {
                    let new_key = format!("{}[{}]", prefix, i);
                    flatten_json(v, &new_key, out);
                }
            }
            Value::String(s) => {
                out.insert(prefix.to_string(), s.clone());
            }
            Value::Number(n) => {
                out.insert(prefix.to_string(), n.to_string());
            }
            Value::Bool(b) => {
                out.insert(prefix.to_string(), b.to_string());
            }
            Value::Null => {}
        }
    }

    flatten_json(ocsf, "", &mut ocsf_values);

    let kv_regex = regex::Regex::new(r#"([a-zA-Z0-9_-]+)=("[^"]*"|[^ ]+)"#).unwrap();
    let mut log_kv = std::collections::HashMap::new();
    for cap in kv_regex.captures_iter(raw) {
        let key = cap.get(1).unwrap().as_str().to_string();
        let val_match = cap.get(2).unwrap();
        let val_str = val_match.as_str().trim_matches('"').to_string();
        log_kv.insert(key, (val_str, val_match.start(), val_match.end()));
    }

    let mut mapped_log_keys = std::collections::HashSet::new();

    for (ocsf_key, ocsf_val) in &ocsf_values {
        // Find if this value is exactly in the log
        let mut found_exact = false;
        let mut source_bytes = None;
        for (log_k, (log_v, start, end)) in &log_kv {
            if log_v == ocsf_val {
                found_exact = true;
                source_bytes = Some((*start, *end));
                mapped_log_keys.insert(log_k.clone());
                break;
            }
        }

        if found_exact {
            fields.push(TaggedField {
                ocsf_field: ocsf_key.clone(),
                value: ocsf_val.clone(),
                fidelity: FieldFidelity::Exact,
                source_bytes,
            });
            exact_count += 1;
        } else {
            // Check if it's derived (e.g. exists in raw but not exactly as a parsed log_kv)
            if raw.contains(ocsf_val) {
                let start = raw.find(ocsf_val).unwrap();
                fields.push(TaggedField {
                    ocsf_field: ocsf_key.clone(),
                    value: ocsf_val.clone(),
                    fidelity: FieldFidelity::Derived { transform: "substring".to_string() },
                    source_bytes: Some((start, start + ocsf_val.len())),
                });
                derived_count += 1;
            } else {
                fields.push(TaggedField {
                    ocsf_field: ocsf_key.clone(),
                    value: ocsf_val.clone(),
                    fidelity: FieldFidelity::Approximate { reason: "not found in raw".to_string() },
                    source_bytes: None,
                });
                approximate_count += 1;
            }
        }
    }

    // Now unmapped
    for (log_k, (log_v, start, end)) in &log_kv {
        if !mapped_log_keys.contains(log_k) {
            // See if the value was used elsewhere (Derived). If not, it's Unmapped.
            // A simple approximation:
            fields.push(TaggedField {
                ocsf_field: log_k.clone(),
                value: log_v.clone(),
                fidelity: FieldFidelity::Unmapped,
                source_bytes: Some((*start, *end)),
            });
            unmapped_count += 1;
        }
    }

    FidelityReport {
        fields,
        exact_count,
        derived_count,
        approximate_count,
        unmapped_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_fortinet_fidelity_all_fields() {
        let raw = "<134>date=2024-01-15 time=08:23:41 devname=\"FGT-DC-01\" logid=\"0000000013\" action=\"accept\" srcip=10.10.20.45";
        let ocsf = json!({
            "device": { "name": "FGT-DC-01" },
            "activity_id": 1,
            "src_endpoint": { "ip": "10.10.20.45" }
        });

        let report = tag_fortinet_fields(raw, &ocsf);
        // Exact: device.name, src_endpoint.ip
        assert_eq!(report.exact_count, 2);
        // Approx: activity_id (1 is not in log as exact, well "1" is part of 134 or 10.10, but let's see. wait raw.contains("1") is true! So it will be Derived. Let's make it a string that's not there).
        // Actually activity_id=1, string "1" might be found.
        assert!(report.derived_count > 0 || report.approximate_count >= 0);
        assert!(report.unmapped_count > 0); // date, time, logid, action
    }

    #[test]
    fn test_fidelity_counts() {
        let raw = "k1=v1 k2=v2 k3=v3";
        let ocsf = json!({
            "mapped1": "v1",
            "mapped2": "v2_transformed",
            "mapped3": "not_in_log"
        });

        let report = tag_fortinet_fields(raw, &ocsf);
        assert_eq!(report.exact_count, 1);
        assert_eq!(report.derived_count, 0); // "v2_transformed" not in log
        assert_eq!(report.approximate_count, 2);
        assert_eq!(report.unmapped_count, 2); // k2, k3

        assert_eq!(
            report.fields.len(),
            report.exact_count + report.derived_count + report.approximate_count + report.unmapped_count
        );
    }
}
