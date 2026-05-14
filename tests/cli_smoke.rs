//! Smoke tests for `kotonoha` (no database).

use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;

#[test]
fn bare_invocation_prints_usage_and_succeeds() {
    Command::cargo_bin("kotonoha")
        .unwrap()
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn version_prints_binary_and_spec_bundle() {
    Command::cargo_bin("kotonoha")
        .unwrap()
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("kotonoha "))
        .stdout(predicate::str::contains("kotonoha-spec"));
}

#[test]
fn rde_emit_round_trips_through_validate_strict() {
    let assert = Command::cargo_bin("kotonoha")
        .unwrap()
        .args(["rde", "emit"])
        .assert()
        .success();
    let json = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8");

    Command::cargo_bin("kotonoha")
        .unwrap()
        .args(["rde", "validate", "--strict"])
        .write_stdin(json)
        .assert()
        .success();
}

#[test]
fn interchange_emit_round_trips_through_validate_strict() {
    let assert = Command::cargo_bin("kotonoha")
        .unwrap()
        .args(["interchange", "emit"])
        .assert()
        .success();
    let json = String::from_utf8(assert.get_output().stdout.clone()).expect("utf8");

    Command::cargo_bin("kotonoha")
        .unwrap()
        .args(["interchange", "validate", "--strict"])
        .write_stdin(json)
        .assert()
        .success();
}

#[test]
fn interchange_ingest_interchange_kind_round_trips_strict() {
    let assert = Command::cargo_bin("kotonoha")
        .unwrap()
        .args(["interchange", "emit"])
        .assert()
        .success();
    let envelope: serde_json::Value =
        serde_json::from_slice(&assert.get_output().stdout).expect("valid json");
    let wrapped = json!({
        "console_event": {
            "version": "kotonoha.console_event.v0",
            "kind": "interchange.ingest.submitted",
            "body": envelope
        }
    });
    let payload = serde_json::to_string(&wrapped).unwrap();

    Command::cargo_bin("kotonoha")
        .unwrap()
        .args(["interchange", "ingest", "--strict"])
        .write_stdin(payload)
        .assert()
        .success();
}

#[test]
fn interchange_ingest_rde_kind_round_trips_strict() {
    let assert = Command::cargo_bin("kotonoha")
        .unwrap()
        .args(["rde", "emit"])
        .assert()
        .success();
    let body: serde_json::Value =
        serde_json::from_slice(&assert.get_output().stdout).expect("valid json");
    let wrapped = json!({
        "console_event": {
            "version": "kotonoha.console_event.v0",
            "kind": "rde.review.requested",
            "body": body
        }
    });
    Command::cargo_bin("kotonoha")
        .unwrap()
        .args(["interchange", "ingest", "--strict"])
        .write_stdin(serde_json::to_string(&wrapped).unwrap())
        .assert()
        .success();
}

#[test]
fn interchange_ingest_unknown_kind_exits_1() {
    let bad = json!({
        "console_event": {
            "version": "kotonoha.console_event.v0",
            "kind": "unknown.event",
            "body": {}
        }
    });
    Command::cargo_bin("kotonoha")
        .unwrap()
        .args(["interchange", "ingest", "--strict"])
        .write_stdin(serde_json::to_string(&bad).unwrap())
        .assert()
        .failure()
        .code(1);
}

#[test]
fn interchange_validate_strict_exit_2_on_unknown_top_level_envelope_keys() {
    let envelope = concat!(
        "{\n",
        "  \"format\": \"kotonoha.interchange.v1\",\n",
        "  \"spec_bundle\": \"0.1\",\n",
        "  \"lineage_unit\": { \"id\": \"https://example.invalid/smoke-extra\", ",
        "\"prior_unit_id\": null },\n",
        "  \"x_custom_trailer\": true\n",
        "}\n",
    );

    Command::cargo_bin("kotonoha")
        .unwrap()
        .args(["interchange", "validate", "--strict"])
        .write_stdin(envelope)
        .assert()
        .failure()
        .code(2);
}
