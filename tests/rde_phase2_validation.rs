//! Phase 2 (SLS-9) CLI smoke: `source_context_status` via `kotonoha-core` delegation.
//!
//! Issue: <https://github.com/zyx-corporation/kotonoha-cli/issues/30>

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;

fn minimal_rde_with_source_context_status(status: serde_json::Value) -> String {
    let mut item = json!({
        "summary": "Phase 2 validation smoke test."
    });
    if !status.is_null() {
        item["source_context_status"] = status;
    }
    let doc = json!({
        "rde_review_output": {
            "spec_version": "0.1",
            "subject_ref": "https://example.invalid/subject/smoke",
            "categories": {
                "preserved": [item.clone()],
                "transformed": [],
                "complemented": [],
                "intentionally_unresolved": [],
                "lost": [],
                "deviation_risk": [],
                "next_update_policy": []
            }
        }
    });
    serde_json::to_string(&doc).expect("json")
}

#[test]
fn rde_validate_accepts_closed_source_context_status() {
    let payload = minimal_rde_with_source_context_status(json!("supplied"));
    Command::cargo_bin("kotonoha")
        .unwrap()
        .args(["rde", "validate", "--strict"])
        .write_stdin(payload)
        .assert()
        .success();
}

#[test]
fn rde_validate_rejects_unknown_source_context_status() {
    let payload = minimal_rde_with_source_context_status(json!("guessed"));
    Command::cargo_bin("kotonoha")
        .unwrap()
        .args(["rde", "validate", "--strict"])
        .write_stdin(payload)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("source_context_status"));
}

#[test]
fn rde_validate_rejects_non_string_source_context_status() {
    let payload = minimal_rde_with_source_context_status(json!(true));
    Command::cargo_bin("kotonoha")
        .unwrap()
        .args(["rde", "validate", "--strict"])
        .write_stdin(payload)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("source_context_status"));
}

#[test]
fn interchange_validate_rejects_nested_invalid_source_context_status() {
    let rde = minimal_rde_with_source_context_status(json!("not-in-vocabulary"));
    let rde_value: serde_json::Value = serde_json::from_str(&rde).expect("rde json");
    let envelope = json!({
        "format": "kotonoha.interchange.v1",
        "spec_bundle": "0.1",
        "lineage_unit": { "id": "00000000-0000-4000-8000-000000000099" },
        "rde_document": rde_value
    });
    Command::cargo_bin("kotonoha")
        .unwrap()
        .args(["interchange", "validate", "--strict"])
        .write_stdin(serde_json::to_string(&envelope).unwrap())
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("source_context_status"));
}
