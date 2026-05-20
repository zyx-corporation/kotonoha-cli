//! M2 integration: MeaningDelta → `rde attach` (metadata) → `export --format m2`.
//!
//! Requires `DATABASE_URL` and `git` (same as CI). Skips when `DATABASE_URL` is unset.

use assert_cmd::Command;
use serde_json::Value;

const M2_EXPORT_FORMAT: &str = "kotonoha.m2_export.v0.1";

fn database_url() -> Option<String> {
    std::env::var("DATABASE_URL").ok()
}

fn kotonoha_cmd() -> Command {
    Command::cargo_bin("kotonoha").unwrap()
}

fn git_available() -> bool {
    std::process::Command::new("git")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn init_git_repo(dir: &std::path::Path) {
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .status()
        .expect("git init");
    std::fs::write(dir.join("note.md"), "# m2 integration\n").expect("write");
    std::process::Command::new("git")
        .args(["add", "note.md"])
        .current_dir(dir)
        .status()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "init"])
        .env("GIT_AUTHOR_NAME", "test")
        .env("GIT_AUTHOR_EMAIL", "test@example.invalid")
        .env("GIT_COMMITTER_NAME", "test")
        .env("GIT_COMMITTER_EMAIL", "test@example.invalid")
        .current_dir(dir)
        .status()
        .expect("git commit");
}

#[test]
fn m2_export_contract_after_attach_with_source_kind() {
    let Some(database_url) = database_url() else {
        eprintln!("skip m2_integration: DATABASE_URL not set");
        return;
    };
    if !git_available() {
        eprintln!("skip m2_integration: git not available");
        return;
    }

    let tmp = tempfile::tempdir().expect("tempdir");
    init_git_repo(tmp.path());

    let obs_path = tmp.path().join("obs.json");
    std::fs::write(&obs_path, r#"{"preserved":["intent"],"lost":[]}"#).expect("obs");

    kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .arg("db")
        .arg("migrate")
        .current_dir(tmp.path())
        .assert()
        .success();

    let delta_out = kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .args(["delta", "create", "note.md", obs_path.to_str().unwrap()])
        .current_dir(tmp.path())
        .assert()
        .success();
    let delta_id = String::from_utf8(delta_out.get_output().stdout.clone())
        .expect("utf8")
        .trim()
        .to_string();
    assert!(!delta_id.is_empty());

    let emit = kotonoha_cmd().args(["rde", "emit"]).assert().success();
    let rde_json = emit.get_output().stdout.clone();

    kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .args([
            "rde",
            "attach",
            "--delta-id",
            &delta_id,
            "--source-kind",
            "llm",
        ])
        .write_stdin(rde_json)
        .assert()
        .success();

    let export_out = kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .args(["export", "--delta-id", &delta_id, "--format", "m2"])
        .assert()
        .success();
    let exported: Value =
        serde_json::from_slice(&export_out.get_output().stdout).expect("export json");

    assert_eq!(
        exported.get("format").and_then(|v| v.as_str()),
        Some(M2_EXPORT_FORMAT)
    );
    let hints = exported
        .get("observation_rde_hints")
        .and_then(|v| v.get("hints"))
        .and_then(|v| v.as_array())
        .expect("observation_rde_hints.hints");
    assert!(!hints.is_empty(), "expected observation mapping hints");

    let assessments = exported
        .get("rde_assessments")
        .and_then(|v| v.as_array())
        .expect("rde_assessments");
    assert_eq!(assessments.len(), 1);
    let a = &assessments[0];
    assert_eq!(a.get("source_kind").and_then(|v| v.as_str()), Some("llm"));
    assert_eq!(
        a.get("payload_schema_version").and_then(|v| v.as_str()),
        Some("0.1")
    );
    assert!(a.get("validation_report").is_some());
}
