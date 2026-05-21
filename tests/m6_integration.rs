//! M6-f: two projects on the same git commit — project-scoped export isolation.
//!
//! Requires `DATABASE_URL` and `git`. Skips when unset.

use assert_cmd::Command;
use kotonoha_core::store::postgres::PgStore;
use kotonoha_core::store::principals::LegacyDefaults;
use serde_json::Value;
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

fn init_git_repo(dir: &std::path::Path) -> String {
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .status()
        .expect("git init");
    std::fs::write(dir.join("note.md"), "# m6 integration\n").expect("write");
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
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .expect("rev-parse");
    String::from_utf8(out.stdout).expect("utf8").trim().to_string()
}

#[test]
fn m6_export_isolates_two_projects_on_same_commit() {
    let Some(database_url) = database_url() else {
        eprintln!("skip m6_integration: DATABASE_URL not set");
        return;
    };
    if !git_available() {
        eprintln!("skip m6_integration: git not available");
        return;
    }

    let tmp = tempfile::tempdir().expect("tempdir");
    let commit = init_git_repo(tmp.path());

    let slug_suffix = Uuid::new_v4().simple().to_string();
    let project_slug = format!("m6-cli-team-b-{slug_suffix}");
    let ext_ref = format!("test.m6.cli-runner-b.{slug_suffix}");

    let rt = tokio::runtime::Runtime::new().expect("runtime");
    let (project_b_id, runner_b_id) = rt.block_on(async {
        let store = PgStore::connect(&database_url).await.expect("connect");
        store.migrate().await.expect("migrate");

        let project_b_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO projects (slug, name) VALUES ($1, 'CLI Team B') RETURNING id"#,
        )
        .bind(&project_slug)
        .fetch_one(store.pool())
        .await
        .expect("project");

        let runner_b_id: Uuid = sqlx::query_scalar(
            r#"INSERT INTO principals (kind, display_name, external_ref)
               VALUES ('human', 'CLI Runner B', $1) RETURNING id"#,
        )
        .bind(&ext_ref)
        .fetch_one(store.pool())
        .await
        .expect("principal");

        sqlx::query(
            r#"INSERT INTO project_members (project_id, principal_id, role)
               VALUES ($1, $2, 'owner')"#,
        )
        .bind(project_b_id)
        .bind(runner_b_id)
        .execute(store.pool())
        .await
        .expect("owner");

        (project_b_id, runner_b_id)
    });

    kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .args(["db", "migrate"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let obs_default = tmp.path().join("obs-default.json");
    let obs_b = tmp.path().join("obs-b.json");
    std::fs::write(&obs_default, r#"{"note":"default project"}"#).expect("obs default");
    std::fs::write(&obs_b, r#"{"note":"team b"}"#).expect("obs b");

    kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .env(
            "KOTONOHA_PROJECT_ID",
            LegacyDefaults::PROJECT_ID.to_string(),
        )
        .env(
            "KOTONOHA_PRINCIPAL_ID",
            LegacyDefaults::PRINCIPAL_ID.to_string(),
        )
        .args([
            "delta",
            "create",
            "note.md",
            "--observation",
            obs_default.to_str().expect("path"),
        ])
        .current_dir(tmp.path())
        .assert()
        .success();

    kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .env("KOTONOHA_PROJECT_ID", project_b_id.to_string())
        .env("KOTONOHA_PRINCIPAL_ID", runner_b_id.to_string())
        .args([
            "delta",
            "create",
            "note.md",
            "--observation",
            obs_b.to_str().expect("path"),
        ])
        .current_dir(tmp.path())
        .assert()
        .success();

    let export_b = kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .env("KOTONOHA_PRINCIPAL_ID", runner_b_id.to_string())
        .args([
            "export",
            "--format",
            "m6",
            "--project-id",
            &project_b_id.to_string(),
            "--git-commit",
            &commit,
        ])
        .current_dir(tmp.path())
        .assert()
        .success();
    let json_b: Value =
        serde_json::from_slice(&export_b.get_output().stdout).expect("json b");
    assert_eq!(
        json_b.get("format").and_then(|v| v.as_str()),
        Some("kotonoha.m6_project_audit_export.v0.1")
    );
    assert_eq!(json_b.get("export_count").and_then(|v| v.as_u64()), Some(1));
    let obs_b = json_b["exports"][0]["meaning_delta"]["observation"]["note"]
        .as_str()
        .unwrap_or("");
    assert_eq!(obs_b, "team b");

    let export_default = kotonoha_cmd()
        .env("DATABASE_URL", &database_url)
        .env(
            "KOTONOHA_PRINCIPAL_ID",
            LegacyDefaults::PRINCIPAL_ID.to_string(),
        )
        .args([
            "export",
            "--format",
            "m6",
            "--project-id",
            &LegacyDefaults::PROJECT_ID.to_string(),
            "--git-commit",
            &commit,
        ])
        .current_dir(tmp.path())
        .assert()
        .success();
    let json_d: Value =
        serde_json::from_slice(&export_default.get_output().stdout).expect("json default");
    assert_eq!(json_d.get("export_count").and_then(|v| v.as_u64()), Some(1));
    let obs_d = json_d["exports"][0]["meaning_delta"]["observation"]["note"]
        .as_str()
        .unwrap_or("");
    assert_eq!(obs_d, "default project");
}
