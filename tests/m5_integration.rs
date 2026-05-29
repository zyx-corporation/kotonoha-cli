//! M5 integration: context export → agent record → delta → capability deny on review.
//!
//! Requires `DATABASE_URL` and `git`. Skips when `DATABASE_URL` is unset.

mod common;

use assert_cmd::Command;
use common::rbac::set_legacy_member_role;
use kotonoha_core::store::postgres::PgStore;
use kotonoha_core::store::principals::LegacyDefaults;
use predicates::prelude::*;
use uuid::Uuid;

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
    std::fs::write(dir.join("note.md"), "# m5 integration\n").expect("write");
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
fn m5_agent_review_approve_denied_with_agent_run_id() {
    let Some(database_url) = database_url() else {
        eprintln!("skip m5_integration: DATABASE_URL not set");
        return;
    };
    if !git_available() {
        eprintln!("skip m5_integration: git not available");
        return;
    }

    let tmp = tempfile::tempdir().expect("tempdir");
    init_git_repo(tmp.path());

    kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .args(["db", "migrate"])
        .current_dir(tmp.path())
        .assert()
        .success();

    set_legacy_member_role(&database_url, "agent_runner");

    let run_out = kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .env(
            "KOTONOHA_PRINCIPAL_ID",
            LegacyDefaults::PRINCIPAL_ID.to_string(),
        )
        .env(
            "KOTONOHA_PROJECT_ID",
            LegacyDefaults::PROJECT_ID.to_string(),
        )
        .args(["agent", "record", "start", "--agent-kind", "m5-test"])
        .current_dir(tmp.path())
        .assert()
        .success();
    let run_id = String::from_utf8(run_out.get_output().stdout.clone())
        .expect("utf8")
        .trim()
        .to_string();

    kotonoha_cmd()
        .args(["context", "export", "note.md"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("kotonoha.context_pack.v0.1"));

    let delta_out = kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .env(
            "KOTONOHA_PRINCIPAL_ID",
            LegacyDefaults::PRINCIPAL_ID.to_string(),
        )
        .env(
            "KOTONOHA_PROJECT_ID",
            LegacyDefaults::PROJECT_ID.to_string(),
        )
        .args([
            "agent",
            "delta",
            "create",
            "note.md",
            "--agent-run-id",
            &run_id,
        ])
        .current_dir(tmp.path())
        .assert()
        .success();
    let delta_id = String::from_utf8(delta_out.get_output().stdout.clone())
        .expect("utf8")
        .trim()
        .to_string();

    kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .args([
            "review",
            "approve",
            "--delta-id",
            &delta_id,
            "--agent-run-id",
            &run_id,
            "--decided-by",
            "agent-bot",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("denied_actions"))
        .stderr(predicate::str::contains("review.approve"));

    set_legacy_member_role(&database_url, "reviewer");

    kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .env(
            "KOTONOHA_PRINCIPAL_ID",
            LegacyDefaults::PRINCIPAL_ID.to_string(),
        )
        .env(
            "KOTONOHA_PROJECT_ID",
            LegacyDefaults::PROJECT_ID.to_string(),
        )
        .args([
            "review",
            "approve",
            "--delta-id",
            &delta_id,
            "--decided-by",
            "human-reviewer",
        ])
        .assert()
        .success();
}

fn denied_actions_contain(run_id: Uuid, action: &str, database_url: &str) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    rt.block_on(async {
        let store = PgStore::connect(database_url).await.expect("connect");
        let run = store
            .get_agent_run(run_id)
            .await
            .expect("query")
            .expect("run row");
        let arr = run.denied_actions.as_array().expect("denied_actions array");
        assert!(
            arr.iter()
                .any(|e| e.get("action").and_then(|v| v.as_str()) == Some(action)),
            "expected denied_actions to contain {action:?}, got {arr:?}"
        );
    });
}

#[test]
fn m5_agent_capability_check_denies_git_and_shell() {
    let Some(database_url) = database_url() else {
        eprintln!("skip m5_integration: DATABASE_URL not set");
        return;
    };

    let run_out = kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .args(["db", "migrate"])
        .assert()
        .success();
    let _ = run_out;

    let run_out = kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .args(["agent", "record", "start", "--agent-kind", "m5-cap-check"])
        .assert()
        .success();
    let run_id = Uuid::parse_str(
        std::str::from_utf8(run_out.get_output().stdout.as_slice())
            .expect("utf8")
            .trim(),
    )
    .expect("uuid");

    for action in ["git.push", "git.commit", "shell"] {
        kotonoha_cmd()
            .env("DATABASE_URL", &database_url)
            .args([
                "agent",
                "capability",
                "check",
                "--action",
                action,
                "--agent-run-id",
                &run_id.to_string(),
            ])
            .assert()
            .failure()
            .code(2)
            .stderr(predicate::str::contains("denied_actions"))
            .stderr(predicate::str::contains(action));
        denied_actions_contain(run_id, action, &database_url);
    }
}

#[test]
fn m5_agent_capability_check_allows_readonly_action() {
    let Some(database_url) = database_url() else {
        eprintln!("skip m5_integration: DATABASE_URL not set");
        return;
    };

    kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .args(["db", "migrate"])
        .assert()
        .success();

    let run_out = kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .args(["agent", "record", "start", "--agent-kind", "m5-cap-allow"])
        .assert()
        .success();
    let run_id = run_out.get_output().stdout.clone();

    kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .args([
            "agent",
            "capability",
            "check",
            "--action",
            "context.export",
            "--agent-run-id",
            std::str::from_utf8(run_id.as_slice()).unwrap().trim(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "kotonoha.agent_capability_check.v0.1",
        ))
        .stdout(predicate::str::contains("\"allowed\": true"));
}
