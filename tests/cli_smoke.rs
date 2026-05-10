//! Smoke tests for `kotonoha` (no database).

use assert_cmd::Command;
use predicates::prelude::*;

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
