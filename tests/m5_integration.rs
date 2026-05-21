//! M5 integration: context export → agent record → delta → capability deny on review.
//!
//! Requires `DATABASE_URL` and `git`. Skips when `DATABASE_URL` is unset.

use assert_cmd::Command;
use predicates::prelude::*;

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

    let run_out = kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
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

    kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
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
