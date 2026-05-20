//! M4 GitHub integration — `gh` CLI + PostgreSQL link tables ([`kotonoha_core::store::github_links`]).
//!
//! Issue: <https://github.com/zyx-corporation/kotonoha-cli/issues/21>

use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Parser, Subcommand};
use serde_json::Value;

use kotonoha_core::store::github_links::GithubRepoRef;
use kotonoha_core::store::postgres::{PgStore, StoreError};

#[derive(Parser)]
#[command(
    about = "GitHub Issue/PR correlation (requires DATABASE_URL, `kotonoha db migrate`, and `gh auth` for live GH API)"
)]
pub struct GithubCli {
    #[command(subcommand)]
    pub action: GithubAction,
}

#[derive(Subcommand)]
pub enum GithubAction {
    /// Check whether `gh` is installed and authenticated.
    GhStatus,
    /// Link repository / issue / pull request records in Kotonoha DB.
    Link {
        #[command(subcommand)]
        target: GithubLinkTarget,
    },
    /// List MeaningDelta rows correlated to a PR (DB links + optional head SHA).
    ListPr {
        #[arg(long)]
        pr_number: i32,
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        head_sha: Option<String>,
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Emit a Markdown section for PR description or review comment (stdout).
    PrSummary {
        #[arg(long)]
        pr_number: i32,
        /// Single delta; omit to include all deltas linked to the PR.
        #[arg(long)]
        delta_id: Option<uuid::Uuid>,
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        head_sha: Option<String>,
        #[arg(long, default_value = "en")]
        locale: String,
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Subcommand)]
pub enum GithubLinkTarget {
    /// Upsert `github_repository_links` for owner/repo (from args or `origin` remote).
    Repo {
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Link a MeaningDelta to a GitHub Issue number.
    Issue {
        #[arg(long)]
        delta_id: uuid::Uuid,
        #[arg(long)]
        issue_number: i32,
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        issue_url: Option<String>,
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Link a MeaningDelta to a Pull Request (optional `head_sha` from `gh pr view`).
    Pr {
        #[arg(long)]
        delta_id: uuid::Uuid,
        #[arg(long)]
        pr_number: i32,
        #[arg(long)]
        owner: Option<String>,
        #[arg(long)]
        repo: Option<String>,
        #[arg(long)]
        head_sha: Option<String>,
        #[arg(long)]
        pr_url: Option<String>,
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
}

#[derive(Debug, Clone, Copy)]
enum SummaryLocale {
    En,
    Ja,
}

impl SummaryLocale {
    fn parse(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "en" | "en-us" => Some(SummaryLocale::En),
            "ja" | "ja-jp" => Some(SummaryLocale::Ja),
            _ => None,
        }
    }
}

pub async fn run(gh: GithubCli) -> i32 {
    match gh.action {
        GithubAction::GhStatus => cmd_gh_status(),
        GithubAction::Link { target } => match target {
            GithubLinkTarget::Repo { owner, repo, path } => {
                cmd_link_repo(&path, owner.as_deref(), repo.as_deref()).await
            }
            GithubLinkTarget::Issue {
                delta_id,
                issue_number,
                owner,
                repo,
                issue_url,
                path,
            } => {
                cmd_link_issue(
                    &path,
                    delta_id,
                    issue_number,
                    owner.as_deref(),
                    repo.as_deref(),
                    issue_url.as_deref(),
                )
                .await
            }
            GithubLinkTarget::Pr {
                delta_id,
                pr_number,
                owner,
                repo,
                head_sha,
                pr_url,
                path,
            } => {
                cmd_link_pr(
                    &path,
                    delta_id,
                    pr_number,
                    owner.as_deref(),
                    repo.as_deref(),
                    head_sha.as_deref(),
                    pr_url.as_deref(),
                )
                .await
            }
        },
        GithubAction::ListPr {
            pr_number,
            owner,
            repo,
            head_sha,
            path,
            json,
        } => cmd_list_pr(&path, pr_number, owner.as_deref(), repo.as_deref(), head_sha.as_deref(), json).await,
        GithubAction::PrSummary {
            pr_number,
            delta_id,
            owner,
            repo,
            head_sha,
            locale,
            path,
        } => {
            cmd_pr_summary(
                &path,
                pr_number,
                delta_id,
                owner.as_deref(),
                repo.as_deref(),
                head_sha.as_deref(),
                &locale,
            )
            .await
        }
    }
}

fn cmd_gh_status() -> i32 {
    match which_gh() {
        None => {
            eprintln!("gh: not found on PATH (install GitHub CLI or skip GitHub commands)");
            1
        }
        Some(gh) => {
            println!("gh: {}", gh.display());
            match gh_auth_status() {
                Ok(msg) => {
                    println!("auth: {msg}");
                    0
                }
                Err(code) => code,
            }
        }
    }
}

async fn cmd_link_repo(path: &Path, owner: Option<&str>, repo: Option<&str>) -> i32 {
    let store = match pg_store().await {
        Ok(s) => s,
        Err(c) => return c,
    };
    if !store.m4_schema_present().await.unwrap_or(false) {
        eprintln!("M4 schema missing — run `kotonoha db migrate`");
        return 1;
    }
    let (owner, repo) = match resolve_owner_repo(path, owner, repo) {
        Ok(x) => x,
        Err(c) => return c,
    };
    match store
        .upsert_github_repository(&GithubRepoRef {
            owner: owner.clone(),
            repo: repo.clone(),
            project_id: None,
            default_branch: None,
        })
        .await
    {
        Ok(row) => {
            println!("{}", row.id);
            0
        }
        Err(e) => {
            eprintln!("{e}");
            store_error_code(&e)
        }
    }
}

async fn cmd_link_issue(
    path: &Path,
    delta_id: uuid::Uuid,
    issue_number: i32,
    owner: Option<&str>,
    repo: Option<&str>,
    issue_url: Option<&str>,
) -> i32 {
    let store = match pg_store().await {
        Ok(s) => s,
        Err(c) => return c,
    };
    if !m4_ready(&store).await {
        return 1;
    }
    let (owner, repo) = match resolve_owner_repo(path, owner, repo) {
        Ok(x) => x,
        Err(c) => return c,
    };
    let repo_row = match store
        .upsert_github_repository(&GithubRepoRef {
            owner,
            repo,
            project_id: None,
            default_branch: None,
        })
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return store_error_code(&e);
        }
    };
    let url = issue_url.map(str::to_string).or_else(|| {
        gh_issue_url(&repo_row.owner, &repo_row.repo, issue_number).ok()
    });
    match store
        .link_meaning_delta_to_github_issue(repo_row.id, delta_id, issue_number, url.as_deref())
        .await
    {
        Ok(link) => {
            println!("{}", link.id);
            0
        }
        Err(e) => {
            eprintln!("{e}");
            store_error_code(&e)
        }
    }
}

async fn cmd_link_pr(
    path: &Path,
    delta_id: uuid::Uuid,
    pr_number: i32,
    owner: Option<&str>,
    repo: Option<&str>,
    head_sha: Option<&str>,
    pr_url: Option<&str>,
) -> i32 {
    let store = match pg_store().await {
        Ok(s) => s,
        Err(c) => return c,
    };
    if !m4_ready(&store).await {
        return 1;
    }
    let (owner, repo) = match resolve_owner_repo(path, owner, repo) {
        Ok(x) => x,
        Err(c) => return c,
    };
    let repo_row = match store
        .upsert_github_repository(&GithubRepoRef {
            owner: owner.clone(),
            repo: repo.clone(),
            project_id: None,
            default_branch: None,
        })
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return store_error_code(&e);
        }
    };
    let head = match head_sha.map(str::to_string) {
        Some(s) => Some(s),
        None => gh_pr_head_sha(&owner, &repo, pr_number).ok(),
    };
    let url = pr_url.map(str::to_string).or_else(|| gh_pr_url(&owner, &repo, pr_number).ok());
    match store
        .link_meaning_delta_to_github_pr(
            repo_row.id,
            delta_id,
            pr_number,
            url.as_deref(),
            head.as_deref(),
        )
        .await
    {
        Ok(link) => {
            println!("{}", link.id);
            0
        }
        Err(e) => {
            eprintln!("{e}");
            store_error_code(&e)
        }
    }
}

async fn cmd_list_pr(
    path: &Path,
    pr_number: i32,
    owner: Option<&str>,
    repo: Option<&str>,
    head_sha: Option<&str>,
    json: bool,
) -> i32 {
    let store = match pg_store().await {
        Ok(s) => s,
        Err(c) => return c,
    };
    if !m4_ready(&store).await {
        return 1;
    }
    let (owner, repo) = match resolve_owner_repo(path, owner, repo) {
        Ok(x) => x,
        Err(c) => return c,
    };
    let repo_row = match store
        .upsert_github_repository(&GithubRepoRef {
            owner,
            repo,
            project_id: None,
            default_branch: None,
        })
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return store_error_code(&e);
        }
    };
    let head = match head_sha.map(str::to_string) {
        Some(s) => Some(s),
        None => gh_pr_head_sha(&repo_row.owner, &repo_row.repo, pr_number).ok(),
    };
    let deltas = match store
        .list_meaning_deltas_for_github_pr(repo_row.id, pr_number, head.as_deref())
        .await
    {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{e}");
            return store_error_code(&e);
        }
    };
    if json {
        let ids: Vec<uuid::Uuid> = deltas.iter().map(|d| d.id).collect();
        match serde_json::to_string_pretty(&serde_json::json!({
            "owner": repo_row.owner,
            "repo": repo_row.repo,
            "pr_number": pr_number,
            "head_sha": head,
            "meaning_delta_ids": ids,
        })) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("json encode: {e}");
                return 3;
            }
        }
    } else {
        for d in &deltas {
            println!("{} {} {}", d.id, d.git_commit, d.file_path);
        }
    }
    0
}

async fn cmd_pr_summary(
    path: &Path,
    pr_number: i32,
    delta_id: Option<uuid::Uuid>,
    owner: Option<&str>,
    repo: Option<&str>,
    head_sha: Option<&str>,
    locale: &str,
) -> i32 {
    let loc = match SummaryLocale::parse(locale) {
        Some(l) => l,
        None => {
            eprintln!("unknown --locale {locale:?} (use en or ja)");
            return 1;
        }
    };
    let store = match pg_store().await {
        Ok(s) => s,
        Err(c) => return c,
    };
    if !m4_ready(&store).await {
        return 1;
    }
    let (owner, repo) = match resolve_owner_repo(path, owner, repo) {
        Ok(x) => x,
        Err(c) => return c,
    };
    let repo_row = match store
        .upsert_github_repository(&GithubRepoRef {
            owner,
            repo,
            project_id: None,
            default_branch: None,
        })
        .await
    {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            return store_error_code(&e);
        }
    };
    let head = match head_sha.map(str::to_string) {
        Some(s) => Some(s),
        None => gh_pr_head_sha(&repo_row.owner, &repo_row.repo, pr_number).ok(),
    };

    let delta_ids: Vec<uuid::Uuid> = if let Some(id) = delta_id {
        vec![id]
    } else {
        match store
            .list_meaning_deltas_for_github_pr(repo_row.id, pr_number, head.as_deref())
            .await
        {
            Ok(rows) => rows.into_iter().map(|r| r.id).collect(),
            Err(e) => {
                eprintln!("{e}");
                return store_error_code(&e);
            }
        }
    };

    if delta_ids.is_empty() {
        eprintln!("no meaning deltas linked to PR #{pr_number}");
        return 2;
    }

    let mut sections = Vec::new();
    for id in delta_ids {
        let export = match build_m2_export_for_summary(&store, id).await {
            Ok(v) => v,
            Err(c) => return c,
        };
        sections.push(export_section_from_m2(&export, loc));
    }

    print!("{}", render_pr_summary_markdown(loc, &repo_row.owner, &repo_row.repo, pr_number, &sections));
    0
}

async fn build_m2_export_for_summary(store: &PgStore, delta_id: uuid::Uuid) -> Result<Value, i32> {
    let row = match store.get_meaning_delta(delta_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            eprintln!("meaning delta not found: {delta_id}");
            return Err(2);
        }
        Err(e) => {
            eprintln!("{e}");
            return Err(store_error_code(&e));
        }
    };
    let assessments = store
        .list_rde_assessments_for_meaning_delta(delta_id)
        .await
        .map_err(|e| {
            eprintln!("{e}");
            store_error_code(&e)
        })?;
    let decisions = store
        .list_review_decisions_for_meaning_delta(delta_id)
        .await
        .map_err(|e| {
            eprintln!("{e}");
            store_error_code(&e)
        })?;
    Ok(crate::export_fmt::m2_export_value(&row, &assessments, &decisions))
}

fn export_section_from_m2(export: &Value, loc: SummaryLocale) -> String {
    let summary = export
        .get("summary_paragraph")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let delta = export.get("meaning_delta").cloned().unwrap_or(Value::Null);
    let id = delta.get("id").and_then(|v| v.as_str()).unwrap_or("?");
    let file = delta.get("file_path").and_then(|v| v.as_str()).unwrap_or("?");
    let commit = delta.get("git_commit").and_then(|v| v.as_str()).unwrap_or("?");
    let n_rde = export
        .get("rde_assessments")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let decision = export
        .get("review_decisions")
        .and_then(|v| v.as_array())
        .and_then(|a| a.first())
        .and_then(|d| d.get("decision"))
        .and_then(|v| v.as_str());

    match loc {
        SummaryLocale::En => format!(
            "- **ΔM `{id}`** — `{file}` @ `{commit}` · {n_rde} RDE assessment(s){}\n  {summary}",
            decision
                .map(|d| format!(" · latest review `{d}`"))
                .unwrap_or_default()
        ),
        SummaryLocale::Ja => format!(
            "- **ΔM `{id}`** — `{file}` @ `{commit}` · RDE 評価 {n_rde} 件{}\n  {summary}",
            decision
                .map(|d| format!(" · 最新レビュー `{d}`"))
                .unwrap_or_default()
        ),
    }
}

fn render_pr_summary_markdown(
    loc: SummaryLocale,
    owner: &str,
    repo: &str,
    pr_number: i32,
    sections: &[String],
) -> String {
    let (title, banner, repo_line) = match loc {
        SummaryLocale::En => (
            "## Kotonoha semantic summary",
            "> RDE assessments support review; they do **not** replace human judgment.",
            format!("Repository: `{owner}/{repo}` · PR #{pr_number}"),
        ),
        SummaryLocale::Ja => (
            "## Kotonoha 意味サマリー",
            "> RDE はレビューを支援します。**最終判断の代替ではありません**（人間責任）。",
            format!("リポジトリ: `{owner}/{repo}` · PR #{pr_number}"),
        ),
    };
    let mut out = format!("{title}\n\n{banner}\n\n{repo_line}\n\n");
    for s in sections {
        out.push_str(s);
        out.push('\n');
    }
    out
}

async fn m4_ready(store: &PgStore) -> bool {
    match store.m4_schema_present().await {
        Ok(true) => true,
        Ok(false) => {
            eprintln!("M4 schema missing — run `kotonoha db migrate`");
            false
        }
        Err(e) => {
            eprintln!("{e}");
            false
        }
    }
}

fn resolve_owner_repo(path: &Path, owner: Option<&str>, repo: Option<&str>) -> Result<(String, String), i32> {
    if let (Some(o), Some(r)) = (owner, repo) {
        return Ok((o.to_string(), r.to_string()));
    }
    let git = match kotonoha_core::git::discover_repo(Some(path)) {
        Ok(c) => c,
        Err(kotonoha_core::git::GitError::NotARepository) => {
            eprintln!("requires a Git repository (or pass --owner and --repo)");
            return Err(1);
        }
        Err(e) => {
            eprintln!("{e}");
            return Err(3);
        }
    };
    parse_origin_remote(&git.root).map_err(|_| {
        eprintln!("could not parse owner/repo from git remote (pass --owner and --repo)");
        1
    })
}

fn parse_origin_remote(repo_root: &Path) -> Result<(String, String), ()> {
    let out = Command::new("git")
        .current_dir(repo_root)
        .args(["remote", "get-url", "origin"])
        .output()
        .map_err(|_| ())?;
    if !out.status.success() {
        return Err(());
    }
    let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
    parse_github_remote_url(&url).ok_or(())
}

fn parse_github_remote_url(url: &str) -> Option<(String, String)> {
    let url = url.trim();
    // https://github.com/owner/repo.git
    if let Some(rest) = url.strip_prefix("https://github.com/") {
        return split_owner_repo(rest);
    }
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        return split_owner_repo(rest);
    }
    if let Some(rest) = url.strip_prefix("ssh://git@github.com/") {
        return split_owner_repo(rest);
    }
    None
}

fn split_owner_repo(s: &str) -> Option<(String, String)> {
    let s = s.trim_end_matches(".git");
    let mut parts = s.split('/');
    let owner = parts.next()?.to_string();
    let repo = parts.next()?.to_string();
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some((owner, repo))
}

fn which_gh() -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(if cfg!(windows) { "gh.exe" } else { "gh" });
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

fn gh_auth_status() -> Result<String, i32> {
    let out = Command::new("gh")
        .args(["auth", "status"])
        .output()
        .map_err(|e| {
            eprintln!("gh auth status: {e}");
            3
        })?;
    if out.status.success() {
        let text = String::from_utf8_lossy(&out.stdout);
        let first = text.lines().next().unwrap_or("ok").trim();
        Ok(first.to_string())
    } else {
        eprintln!("{}", String::from_utf8_lossy(&out.stderr));
        Err(1)
    }
}

fn gh_pr_head_sha(owner: &str, repo: &str, pr_number: i32) -> Result<String, i32> {
    if which_gh().is_none() {
        return Err(1);
    }
    let out = Command::new("gh")
        .args([
            "pr",
            "view",
            &pr_number.to_string(),
            "--repo",
            &format!("{owner}/{repo}"),
            "--json",
            "headRefOid",
            "-q",
            ".headRefOid",
        ])
        .output()
        .map_err(|e| {
            eprintln!("gh pr view: {e}");
            3
        })?;
    if !out.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&out.stderr));
        return Err(1);
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

fn gh_pr_url(owner: &str, repo: &str, pr_number: i32) -> Result<String, i32> {
    Ok(format!("https://github.com/{owner}/{repo}/pull/{pr_number}"))
}

fn gh_issue_url(owner: &str, repo: &str, issue_number: i32) -> Result<String, i32> {
    Ok(format!(
        "https://github.com/{owner}/{repo}/issues/{issue_number}"
    ))
}

async fn pg_store() -> Result<PgStore, i32> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!("DATABASE_URL is not set");
            return Err(1);
        }
    };
    PgStore::connect(&url).await.map_err(|e| {
        eprintln!("database connection failed: {e}");
        3
    })
}

fn store_error_code(e: &StoreError) -> i32 {
    match e {
        StoreError::InterchangeValidation(_)
        | StoreError::RdeValidation(_)
        | StoreError::Lineage(_)
        | StoreError::SemanticLineage(_)
        | StoreError::MissingField(_)
        | StoreError::Json(_) => 2,
        StoreError::Sql(_) | StoreError::Migrate(_) => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_https_remote() {
        let (o, r) = parse_github_remote_url("https://github.com/zyx-corporation/kotonoha-cli.git").unwrap();
        assert_eq!(o, "zyx-corporation");
        assert_eq!(r, "kotonoha-cli");
    }

    #[test]
    fn parse_ssh_remote() {
        let (o, r) = parse_github_remote_url("git@github.com:zyx-corporation/kotonoha-core.git").unwrap();
        assert_eq!(o, "zyx-corporation");
        assert_eq!(r, "kotonoha-core");
    }
}
