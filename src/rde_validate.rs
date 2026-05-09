//! Transitional in-repo validation for RDE interchange (Phase 1).
//! Replace with `kotonoha-core` when a shared library exists.

use serde_json::Value;

const CATEGORY_KEYS: [&str; 7] = [
    "preserved",
    "transformed",
    "complemented",
    "intentionally_unresolved",
    "lost",
    "deviation_risk",
    "next_update_policy",
];

/// Returns warnings (stderr) when `strict` is false; errors fail validation.
pub fn validate_json(text: &str, strict: bool) -> Result<Vec<String>, String> {
    let root: Value = serde_json::from_str(text).map_err(|e| format!("invalid JSON: {e}"))?;

    let inner = root.get("rde_review_output").ok_or_else(|| {
        "missing top-level key \"rde_review_output\" (see kotonoha-spec docs/rde-review-output.md)"
            .to_string()
    })?;

    if !inner.is_object() {
        return Err("\"rde_review_output\" must be a JSON object".to_string());
    }

    let spec_version = inner
        .get("spec_version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing string \"spec_version\" under \"rde_review_output\"".to_string())?;

    if spec_version != crate::TARGET_SPEC_BUNDLE {
        return Err(format!(
            "\"spec_version\" must be \"{}\" for Phase 1 interchange validation (got {:?})",
            crate::TARGET_SPEC_BUNDLE,
            spec_version
        ));
    }

    inner
        .get("subject_ref")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            "missing non-empty string \"subject_ref\" under \"rde_review_output\"".to_string()
        })?;

    let categories = inner
        .get("categories")
        .ok_or_else(|| "missing \"categories\" object under \"rde_review_output\"".to_string())?;

    let cat_obj = categories
        .as_object()
        .ok_or_else(|| "\"categories\" must be a JSON object".to_string())?;

    for key in CATEGORY_KEYS {
        if !cat_obj.contains_key(key) {
            return Err(format!(
                "missing category key {:?} under \"categories\"",
                key
            ));
        }
    }

    for (key, val) in cat_obj.iter() {
        if !CATEGORY_KEYS.contains(&key.as_str()) {
            return Err(format!(
                "unknown category key {:?} (Phase 1 allows only the seven normative keys)",
                key
            ));
        }
        if !val.is_array() {
            return Err(format!("category {:?} must be a JSON array", key));
        }
    }

    let mut warnings = Vec::new();

    for key in CATEGORY_KEYS {
        let arr = cat_obj[key].as_array().unwrap();
        for (i, item) in arr.iter().enumerate() {
            let obj = item.as_object().ok_or_else(|| {
                format!("category {:?}[{}]: each item must be a JSON object", key, i)
            })?;
            let summary = obj.get("summary").and_then(|v| v.as_str());
            if summary.map(|s| s.trim().is_empty()).unwrap_or(true) {
                let msg = format!(
                    "category {:?}[{}]: item SHOULD include non-empty \"summary\" (kotonoha-spec)",
                    key, i
                );
                if strict {
                    return Err(msg);
                }
                warnings.push(msg);
            }
        }
    }

    Ok(warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn minimal_valid() -> String {
        r#"{
            "rde_review_output": {
                "spec_version": "0.1",
                "subject_ref": "https://example.invalid/x",
                "categories": {
                    "preserved": [],
                    "transformed": [],
                    "complemented": [],
                    "intentionally_unresolved": [],
                    "lost": [],
                    "deviation_risk": [],
                    "next_update_policy": []
                }
            }
        }"#
        .to_string()
    }

    #[test]
    fn accepts_minimal() {
        assert!(validate_json(&minimal_valid(), false).unwrap().is_empty());
    }

    #[test]
    fn rejects_bad_spec_version() {
        let mut s = minimal_valid();
        s = s.replace("\"0.1\"", "\"0.2\"");
        assert!(validate_json(&s, false).is_err());
    }

    #[test]
    fn warns_without_summary_non_strict() {
        let j = r#"{
            "rde_review_output": {
                "spec_version": "0.1",
                "subject_ref": "https://example.invalid/x",
                "categories": {
                    "preserved": [{}],
                    "transformed": [],
                    "complemented": [],
                    "intentionally_unresolved": [],
                    "lost": [],
                    "deviation_risk": [],
                    "next_update_policy": []
                }
            }
        }"#;
        let w = validate_json(j, false).unwrap();
        assert!(!w.is_empty());
    }

    #[test]
    fn strict_fails_without_summary() {
        let j = r#"{
            "rde_review_output": {
                "spec_version": "0.1",
                "subject_ref": "https://example.invalid/x",
                "categories": {
                    "preserved": [{}],
                    "transformed": [],
                    "complemented": [],
                    "intentionally_unresolved": [],
                    "lost": [],
                    "deviation_risk": [],
                    "next_update_policy": []
                }
            }
        }"#;
        assert!(validate_json(j, true).is_err());
    }
}
