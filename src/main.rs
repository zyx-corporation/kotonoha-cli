//! `kotonoha` — CLI entry (argument parsing and UX). Domain logic comes from [`kotonoha_core`].

mod project;

use std::io::{self, Read, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process;

use clap::{CommandFactory, Parser, Subcommand};
use serde_json::Value;

#[derive(Parser)]
#[command(name = "kotonoha")]
#[command(
    about = "Kotonoha / SLS developer CLI (see docs/cli-definition.md)",
    version
)]
struct Cli {
    /// Omitted subcommand prints full help (same as `--help`; see docs/cli-definition.md §4).
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Print CLI build identity and targeted specification bundle version.
    Version,
    /// PostgreSQL operations (`DATABASE_URL`; migrations ship with `kotonoha-core`).
    Db {
        #[command(subcommand)]
        action: DbAction,
    },
    /// RDE review output interchange (validate / emit skeleton).
    Rde {
        #[command(subcommand)]
        action: RdeAction,
    },
    /// Bundled lineage +/or RDE JSON envelope (`kotonoha.interchange.v1`, core-supported).
    Interchange {
        #[command(subcommand)]
        action: InterchangeAction,
    },
    /// Initialize `.kotonoha/config.toml` in a Git repository (M1).
    Init {
        /// Project id (defaults to directory name).
        #[arg(long)]
        project_id: Option<String>,
        /// Directory to treat as repository root (default: `.`).
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Show Git + database + project context (M1).
    Status {
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Show Git unified diff for working tree changes (M1).
    Diff {
        #[arg(long, default_value = ".")]
        path: PathBuf,
        /// Limit diff to this file (repo-relative or absolute).
        file: Option<PathBuf>,
    },
    /// MeaningDelta (ΔM) operations (M1; requires `DATABASE_URL` + `db migrate`).
    Delta {
        #[command(subcommand)]
        action: DeltaAction,
    },
    /// Record human review decisions (M1; does not substitute for institutional authority).
    Review {
        #[command(subcommand)]
        action: ReviewAction,
    },
    /// Export MeaningDelta + RDE + review decisions as JSON (M1 audit report).
    Export {
        /// Meaning delta UUID (`meaning_deltas.id`).
        #[arg(long, group = "target")]
        delta_id: Option<uuid::Uuid>,
        /// Git commit hash (exports all deltas for that commit).
        #[arg(long, group = "target")]
        git_commit: Option<String>,
        /// Write JSON to file instead of stdout.
        #[arg(long)]
        out: Option<PathBuf>,
    },
}

const M1_EXPORT_FORMAT: &str = "kotonoha.m1_export.v0.1";
const M1_EXPORT_BUNDLE_FORMAT: &str = "kotonoha.m1_export_bundle.v0.1";

#[derive(clap::Args)]
struct ReviewRecordArgs {
    #[arg(long)]
    delta_id: uuid::Uuid,
    #[arg(long)]
    assessment_id: Option<uuid::Uuid>,
    /// Reviewer identity (defaults: `KOTONOHA_DECIDED_BY`, `git config user.email`, `$USER`).
    #[arg(long)]
    decided_by: Option<String>,
    /// Rationale JSON file (omit for `{}`).
    #[arg(long)]
    rationale: Option<PathBuf>,
}

#[derive(Subcommand)]
enum ReviewAction {
    /// Record approval (human-in-the-loop; RDE does not replace judgment).
    Approve(ReviewRecordArgs),
    /// Record hold (pending further review).
    Hold(ReviewRecordArgs),
    /// Record rejection (send back for revision).
    Reject(ReviewRecordArgs),
}

#[derive(Subcommand)]
enum DeltaAction {
    /// Register a meaning change anchored to a file in the current Git repo.
    Create {
        /// Changed file (repo-relative or absolute).
        file: PathBuf,
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        line_start: Option<i32>,
        #[arg(long)]
        line_end: Option<i32>,
        /// Alternative to line range (e.g. `unstaged:docs/foo.md`).
        #[arg(long)]
        diff_ref: Option<String>,
        /// Observation JSON (`preserved`, `lost`, …). Omit for `{}`; use `-` file or stdin when wired via shell.
        observation: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum DbAction {
    /// Apply DDL migrations from `kotonoha-core/migrations` via SQLx.
    Migrate,
}

#[derive(Subcommand)]
enum RdeAction {
    /// Validate JSON against Phase 1 RDE interchange (`docs/rde-review-output.md` in kotonoha-spec).
    Validate {
        /// Fail if category items omit `summary` (spec SHOULD).
        #[arg(long)]
        strict: bool,
        /// JSON file, or `-` / omit for stdin.
        path: Option<PathBuf>,
    },
    /// Emit a minimal compliant JSON skeleton (stdout).
    Emit,
    /// Attach validated RDE JSON to an existing MeaningDelta (`rde_assessments` row).
    Attach {
        /// Target `meaning_deltas.id` from `delta create`.
        #[arg(long)]
        delta_id: uuid::Uuid,
        #[arg(long)]
        strict: bool,
        /// Also insert spec-shaped row into `rde_documents` and link FK.
        #[arg(long)]
        materialize_document: bool,
        /// Audit correlation (defaults to RDE `subject_ref` when present).
        #[arg(long)]
        audit_correlation_id: Option<String>,
        /// RDE JSON file, or stdin when omitted / `-`.
        path: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum InterchangeAction {
    /// Validate `kotonoha.interchange.v1` envelope JSON (optional nested RDE document).
    Validate {
        #[arg(long)]
        strict: bool,
        path: Option<PathBuf>,
    },
    /// Persist a validated envelope to PostgreSQL (`interchange_documents`; requires `DATABASE_URL`).
    Store {
        #[arg(long)]
        strict: bool,
        path: Option<PathBuf>,
    },
    /// Ingest a **Phase 3** `kotonoha.console_event.v0` JSON wrapper (see docs/cli-definition.md §4.1).
    Ingest {
        #[arg(long)]
        strict: bool,
        /// After validation, persist interchange body (same as `interchange store`; `interchange.ingest.submitted` only).
        #[arg(long)]
        persist: bool,
        path: Option<PathBuf>,
    },
    Emit,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let code = match cli.command {
        None => cmd_help(),
        Some(Commands::Version) => cmd_version(),
        Some(Commands::Db { action }) => match action {
            DbAction::Migrate => cmd_db_migrate().await,
        },
        Some(Commands::Rde { action }) => match action {
            RdeAction::Validate { strict, path } => cmd_rde_validate(strict, path.as_deref()),
            RdeAction::Emit => cmd_rde_emit(),
            RdeAction::Attach {
                delta_id,
                strict,
                materialize_document,
                audit_correlation_id,
                path,
            } => {
                cmd_rde_attach(
                    delta_id,
                    strict,
                    materialize_document,
                    audit_correlation_id.as_deref(),
                    path.as_deref(),
                )
                .await
            }
        },
        Some(Commands::Interchange { action }) => match action {
            InterchangeAction::Validate { strict, path } => {
                cmd_interchange_validate(strict, path.as_deref())
            }
            InterchangeAction::Store { strict, path } => {
                cmd_interchange_store(strict, path.as_deref()).await
            }
            InterchangeAction::Emit => cmd_interchange_emit(),
            InterchangeAction::Ingest {
                strict,
                persist,
                path,
            } => cmd_interchange_ingest(strict, persist, path.as_deref()).await,
        },
        Some(Commands::Init { project_id, path }) => cmd_init(project_id.as_deref(), &path),
        Some(Commands::Status { path }) => cmd_status(&path).await,
        Some(Commands::Diff { path, file }) => cmd_diff(&path, file.as_deref()),
        Some(Commands::Delta { action }) => match action {
            DeltaAction::Create {
                file,
                path,
                line_start,
                line_end,
                diff_ref,
                observation,
            } => {
                cmd_delta_create(
                    &path,
                    &file,
                    line_start,
                    line_end,
                    diff_ref,
                    observation.as_deref(),
                )
                .await
            }
        },
        Some(Commands::Review { action }) => match action {
            ReviewAction::Approve(args) => {
                cmd_review_record(
                    kotonoha_core::semantic_lineage::ReviewDecisionKind::Approve,
                    &args,
                )
                .await
            }
            ReviewAction::Hold(args) => {
                cmd_review_record(
                    kotonoha_core::semantic_lineage::ReviewDecisionKind::Hold,
                    &args,
                )
                .await
            }
            ReviewAction::Reject(args) => {
                cmd_review_record(
                    kotonoha_core::semantic_lineage::ReviewDecisionKind::Reject,
                    &args,
                )
                .await
            }
        },
        Some(Commands::Export {
            delta_id,
            git_commit,
            out,
        }) => cmd_export(delta_id, git_commit.as_deref(), out.as_deref()).await,
    };
    process::exit(code);
}

fn store_error_code(e: &kotonoha_core::store::postgres::StoreError) -> i32 {
    use kotonoha_core::store::postgres::StoreError;
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

async fn pg_store() -> Result<kotonoha_core::store::postgres::PgStore, i32> {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!("DATABASE_URL is not set");
            return Err(1);
        }
    };
    kotonoha_core::store::postgres::PgStore::connect(&url)
        .await
        .map_err(|e| {
            eprintln!("database connection failed: {e}");
            3
        })
}

async fn cmd_delta_create(
    repo_path: &Path,
    file: &Path,
    line_start: Option<i32>,
    line_end: Option<i32>,
    diff_ref: Option<String>,
    observation_path: Option<&Path>,
) -> i32 {
    let git = match kotonoha_core::git::discover_repo(Some(repo_path)) {
        Ok(c) => c,
        Err(kotonoha_core::git::GitError::NotARepository) => {
            eprintln!("delta create requires a Git repository");
            return 1;
        }
        Err(e) => {
            eprintln!("{}", e);
            return 3;
        }
    };
    let rel = match kotonoha_core::git::path_relative_to_root(&git, file) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let observation = match read_observation_json(observation_path) {
        Ok(v) => v,
        Err(code) => return code,
    };
    let diff_ref = diff_ref.or_else(|| {
        if line_start.is_none() && line_end.is_none() {
            Some(format!("unstaged:{rel}"))
        } else {
            None
        }
    });
    let input = kotonoha_core::semantic_lineage::MeaningDeltaInput {
        document_object_id: None,
        prior_meaning_state_id: None,
        new_meaning_state_id: None,
        agent_run_id: None,
        git_anchor: kotonoha_core::semantic_lineage::GitAnchor {
            git_commit: git.commit,
            file_path: rel,
            line_range_start: line_start,
            line_range_end: line_end,
            diff_ref,
        },
        observation,
        source_context: Value::Object(Default::default()),
    };
    let store = match pg_store().await {
        Ok(s) => s,
        Err(c) => return c,
    };
    match store.create_meaning_delta(&input).await {
        Ok(id) => {
            println!("{}", id);
            0
        }
        Err(e) => {
            eprintln!("{}", e);
            store_error_code(&e)
        }
    }
}

fn read_observation_json(path: Option<&Path>) -> Result<Value, i32> {
    match path {
        None => Ok(Value::Object(Default::default())),
        Some(p) => {
            let text = match read_input_text(Some(p)) {
                Ok(t) => t,
                Err((code, msg)) => {
                    if let Some(m) = msg {
                        eprintln!("{}", m);
                    }
                    return Err(code);
                }
            };
            serde_json::from_str(&text).map_err(|e| {
                eprintln!("observation JSON: {e}");
                2
            })
        }
    }
}

async fn cmd_review_record(
    decision: kotonoha_core::semantic_lineage::ReviewDecisionKind,
    args: &ReviewRecordArgs,
) -> i32 {
    let decided_by = match resolve_decided_by(args.decided_by.as_deref()) {
        Ok(s) => s,
        Err(c) => return c,
    };
    let rationale = match read_observation_json(args.rationale.as_deref()) {
        Ok(v) => v,
        Err(code) => return code,
    };
    let input = kotonoha_core::semantic_lineage::RecordReviewDecisionInput {
        meaning_delta_id: args.delta_id,
        rde_assessment_id: args.assessment_id,
        decision,
        decided_by,
        rationale,
    };
    let store = match pg_store().await {
        Ok(s) => s,
        Err(c) => return c,
    };
    match store.record_review_decision(&input).await {
        Ok(id) => {
            println!("{}", id);
            0
        }
        Err(e) => {
            eprintln!("{}", e);
            store_error_code(&e)
        }
    }
}

fn resolve_decided_by(override_: Option<&str>) -> Result<String, i32> {
    if let Some(s) = override_ {
        let t = s.trim();
        if !t.is_empty() {
            return Ok(t.to_string());
        }
    }
    if let Ok(s) = std::env::var("KOTONOHA_DECIDED_BY") {
        let t = s.trim();
        if !t.is_empty() {
            return Ok(t.to_string());
        }
    }
    if let Ok(out) = std::process::Command::new("git")
        .args(["config", "user.email"])
        .output()
    {
        if out.status.success() {
            let email = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !email.is_empty() {
                return Ok(email);
            }
        }
    }
    if let Ok(u) = std::env::var("USER") {
        let t = u.trim();
        if !t.is_empty() {
            return Ok(t.to_string());
        }
    }
    eprintln!(
        "decided_by is required: pass --decided-by, set KOTONOHA_DECIDED_BY, or configure git user.email"
    );
    Err(1)
}

async fn cmd_export(
    delta_id: Option<uuid::Uuid>,
    git_commit: Option<&str>,
    out_path: Option<&Path>,
) -> i32 {
    match (delta_id, git_commit) {
        (Some(_), Some(_)) => {
            eprintln!("export: specify only one of --delta-id or --git-commit");
            return 1;
        }
        (None, None) => {
            eprintln!("export: --delta-id or --git-commit is required");
            return 1;
        }
        _ => {}
    }
    let store = match pg_store().await {
        Ok(s) => s,
        Err(c) => return c,
    };
    let json = match (delta_id, git_commit) {
        (Some(id), None) => match build_m1_export(&store, id).await {
            Ok(v) => v,
            Err(c) => return c,
        },
        (None, Some(commit)) => match build_m1_export_bundle(&store, commit).await {
            Ok(v) => v,
            Err(c) => return c,
        },
        _ => unreachable!(),
    };
    let pretty = match serde_json::to_string_pretty(&json) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("export JSON encode: {e}");
            return 3;
        }
    };
    if let Some(path) = out_path {
        if let Err(e) = std::fs::write(path, format!("{pretty}\n")) {
            eprintln!("write {}: {e}", path.display());
            return 3;
        }
    } else {
        println!("{pretty}");
    }
    0
}

async fn build_m1_export(
    store: &kotonoha_core::store::postgres::PgStore,
    delta_id: uuid::Uuid,
) -> Result<Value, i32> {
    use kotonoha_core::store::postgres::StoreError;
    let row = match store.get_meaning_delta(delta_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            eprintln!("meaning delta not found: {delta_id}");
            return Err(2);
        }
        Err(StoreError::Sql(e)) => {
            eprintln!("{}", e);
            return Err(3);
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(store_error_code(&e));
        }
    };
    let assessments = store
        .list_rde_assessments_for_meaning_delta(delta_id)
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            store_error_code(&e)
        })?;
    let decisions = store
        .list_review_decisions_for_meaning_delta(delta_id)
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            store_error_code(&e)
        })?;
    Ok(m1_export_value(&row, &assessments, &decisions))
}

async fn build_m1_export_bundle(
    store: &kotonoha_core::store::postgres::PgStore,
    git_commit: &str,
) -> Result<Value, i32> {
    let deltas = store
        .list_meaning_deltas_by_git_commit(git_commit)
        .await
        .map_err(|e| {
            eprintln!("{}", e);
            store_error_code(&e)
        })?;
    let mut exports = Vec::with_capacity(deltas.len());
    for row in &deltas {
        let assessments = store
            .list_rde_assessments_for_meaning_delta(row.id)
            .await
            .map_err(|e| {
                eprintln!("{}", e);
                store_error_code(&e)
            })?;
        let decisions = store
            .list_review_decisions_for_meaning_delta(row.id)
            .await
            .map_err(|e| {
                eprintln!("{}", e);
                store_error_code(&e)
            })?;
        exports.push(m1_export_value(row, &assessments, &decisions));
    }
    Ok(serde_json::json!({
        "format": M1_EXPORT_BUNDLE_FORMAT,
        "git_commit": git_commit,
        "exports": exports,
    }))
}

fn m1_export_value(
    row: &kotonoha_core::store::postgres::MeaningDeltaRow,
    assessments: &[kotonoha_core::store::postgres::RdeAssessmentRow],
    decisions: &[kotonoha_core::store::postgres::ReviewDecisionRow],
) -> Value {
    let generated_at_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let summary = build_summary_paragraph(row, assessments, decisions);
    serde_json::json!({
        "format": M1_EXPORT_FORMAT,
        "generated_at_unix": generated_at_unix,
        "meaning_delta": {
            "id": row.id,
            "git_commit": row.git_commit,
            "file_path": row.file_path,
            "line_range_start": row.line_range_start,
            "line_range_end": row.line_range_end,
            "diff_ref": row.diff_ref,
            "observation": row.observation,
            "source_context": row.source_context,
        },
        "rde_assessments": assessments.iter().map(|a| serde_json::json!({
            "id": a.id,
            "meaning_delta_id": a.meaning_delta_id,
            "payload": a.payload,
            "audit_correlation_id": a.audit_correlation_id,
            "rde_document_id": a.rde_document_id,
        })).collect::<Vec<_>>(),
        "review_decisions": decisions.iter().map(|d| serde_json::json!({
            "id": d.id,
            "meaning_delta_id": d.meaning_delta_id,
            "rde_assessment_id": d.rde_assessment_id,
            "decision": d.decision,
            "decided_by": d.decided_by,
            "rationale": d.rationale,
        })).collect::<Vec<_>>(),
        "summary_paragraph": summary,
    })
}

fn build_summary_paragraph(
    row: &kotonoha_core::store::postgres::MeaningDeltaRow,
    assessments: &[kotonoha_core::store::postgres::RdeAssessmentRow],
    decisions: &[kotonoha_core::store::postgres::ReviewDecisionRow],
) -> String {
    let latest = decisions.first();
    let decision_part = latest.map_or_else(
        || "no review decision recorded yet".to_string(),
        |d| format!("latest decision `{}` by {}", d.decision, d.decided_by),
    );
    format!(
        "Meaning change in `{}` at commit {}: {} RDE assessment(s); {}.",
        row.file_path,
        row.git_commit,
        assessments.len(),
        decision_part
    )
}

async fn cmd_rde_attach(
    delta_id: uuid::Uuid,
    strict: bool,
    materialize_document: bool,
    audit_correlation_id: Option<&str>,
    path: Option<&Path>,
) -> i32 {
    let text = match read_input_text(path) {
        Ok(t) => t,
        Err((code, msg)) => {
            if let Some(m) = msg {
                eprintln!("{}", m);
            }
            return code;
        }
    };
    let payload: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("invalid JSON: {e}");
            return 1;
        }
    };
    let correlation = audit_correlation_id.map(str::to_string).or_else(|| {
        payload
            .get("rde_review_output")
            .and_then(|o| o.get("subject_ref"))
            .and_then(|s| s.as_str())
            .map(str::to_string)
    });
    let store = match pg_store().await {
        Ok(s) => s,
        Err(c) => return c,
    };
    match store
        .attach_rde_assessment(
            delta_id,
            payload,
            strict,
            correlation.as_deref(),
            materialize_document,
        )
        .await
    {
        Ok(id) => {
            println!("{}", id);
            0
        }
        Err(e) => {
            eprintln!("{}", e);
            store_error_code(&e)
        }
    }
}

fn cmd_help() -> i32 {
    let mut c = Cli::command();
    if let Err(e) = c.print_help() {
        eprintln!("{}", e);
        return 3;
    }
    println!();
    0
}

fn cmd_init(project_id: Option<&str>, path: &Path) -> i32 {
    let git = match kotonoha_core::git::discover_repo(Some(path)) {
        Ok(c) => c,
        Err(kotonoha_core::git::GitError::NotARepository) => {
            eprintln!("init requires a Git repository (run from repo root or use --path)");
            return 1;
        }
        Err(e) => {
            eprintln!("{}", e);
            return 3;
        }
    };
    match project::init_config(&git.root, project_id) {
        Ok(cfg) => {
            println!("initialized {}", project::config_path(&git.root).display());
            println!("project_id: {}", cfg.project_id);
            0
        }
        Err(e) => {
            eprintln!("{}", e);
            3
        }
    }
}

async fn cmd_status(path: &Path) -> i32 {
    let git = match kotonoha_core::git::discover_repo(Some(path)) {
        Ok(c) => c,
        Err(kotonoha_core::git::GitError::NotARepository) => {
            eprintln!("status requires a Git repository");
            return 1;
        }
        Err(e) => {
            eprintln!("{}", e);
            return 3;
        }
    };
    let wt = match kotonoha_core::git::working_tree_status(&git) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{}", e);
            return 3;
        }
    };
    let cfg = match project::load_config(&git.root) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return 3;
        }
    };

    println!("repository: {}", git.root.display());
    if let Some(ref b) = git.branch {
        println!("branch: {b}");
    } else if git.detached {
        println!("branch: (detached HEAD)");
    }
    println!("commit: {}", git.commit);
    println!("working tree: {}", if wt.dirty { "dirty" } else { "clean" });
    if wt.dirty {
        println!(
            "changes: staged={} unstaged={} untracked={}",
            wt.staged_count, wt.unstaged_count, wt.untracked_count
        );
    }
    match &cfg {
        Some(c) => println!("kotonoha project_id: {}", c.project_id),
        None => println!("kotonoha: not initialized (run `kotonoha init`)"),
    }

    match std::env::var("DATABASE_URL") {
        Ok(_) => {
            if let Some(db) = db_status_summary().await {
                println!("database: connected");
                println!("migrations: {}", db.migrations);
                println!("meaning_deltas: {}", db.meaning_delta_count);
            } else {
                println!("database: connection or query failed");
            }
        }
        Err(_) => println!("database: DATABASE_URL not set"),
    }
    0
}

struct DbStatusSummary {
    migrations: String,
    meaning_delta_count: i64,
}

async fn db_status_summary() -> Option<DbStatusSummary> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let store = kotonoha_core::store::postgres::PgStore::connect(&url)
        .await
        .ok()?;
    let has_m1 = store.m1_schema_present().await.ok()?;
    if !has_m1 {
        return Some(DbStatusSummary {
            migrations: "v0 only (run `kotonoha db migrate` for M1 tables)".into(),
            meaning_delta_count: 0,
        });
    }
    let count = store.count_meaning_deltas().await.ok()?;
    Some(DbStatusSummary {
        migrations: "applied (meaning_deltas present)".into(),
        meaning_delta_count: count,
    })
}

fn cmd_diff(path: &Path, file: Option<&Path>) -> i32 {
    let git = match kotonoha_core::git::discover_repo(Some(path)) {
        Ok(c) => c,
        Err(kotonoha_core::git::GitError::NotARepository) => {
            eprintln!("diff requires a Git repository");
            return 1;
        }
        Err(e) => {
            eprintln!("{}", e);
            return 3;
        }
    };
    let diff = match kotonoha_core::git::diff_unstaged(&git, file) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("{}", e);
            return 3;
        }
    };
    if diff.text.is_empty() {
        println!("(no unstaged diff)");
        return 0;
    }
    let mut out = io::stdout();
    if let Err(e) = out.write_all(diff.text.as_bytes()) {
        eprintln!("write stdout: {e}");
        return 3;
    }
    if !diff.text.ends_with('\n') {
        let _ = writeln!(out);
    }
    0
}

async fn cmd_db_migrate() -> i32 {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!("DATABASE_URL is not set");
            return 1;
        }
    };
    let store = match kotonoha_core::store::postgres::PgStore::connect(&url).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("database connection failed: {e}");
            return 3;
        }
    };
    match store.migrate().await {
        Ok(()) => {
            println!("migrations applied successfully");
            0
        }
        Err(e) => {
            eprintln!("migration failed: {e}");
            3
        }
    }
}

fn cmd_version() -> i32 {
    println!("kotonoha {}", env!("CARGO_PKG_VERSION"));
    println!(
        "kotonoha-spec (target bundle): {}",
        kotonoha_core::TARGET_SPEC_BUNDLE
    );
    0
}

fn cmd_rde_emit() -> i32 {
    let skeleton = serde_json::json!({
        "rde_review_output": {
            "spec_version": kotonoha_core::TARGET_SPEC_BUNDLE,
            "subject_ref": "https://example.invalid/subject/REPLACE",
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
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&skeleton).unwrap_or_else(|_| "{}".to_string())
    );
    0
}

fn cmd_interchange_emit() -> i32 {
    let skeleton = serde_json::json!({
        "format": kotonoha_core::interchange::INTERCHANGE_FORMAT_V1,
        "spec_bundle": kotonoha_core::TARGET_SPEC_BUNDLE,
        "lineage_unit": {
            "id": "https://example.invalid/lineage/REPLACE",
            "prior_unit_id": null
        }
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&skeleton).unwrap_or_else(|_| "{}".to_string())
    );
    0
}

fn cmd_rde_validate(strict: bool, path: Option<&Path>) -> i32 {
    let text = match read_input_text(path) {
        Ok(t) => t,
        Err((code, msg)) => {
            if let Some(m) = msg {
                eprintln!("{}", m);
            }
            return code;
        }
    };
    match kotonoha_core::rde::validate_json(&text, strict) {
        Ok(warnings) => {
            for w in warnings {
                eprintln!("warning: {}", w);
            }
            0
        }
        Err(e) => {
            eprintln!("{}", e);
            2
        }
    }
}

async fn cmd_interchange_store(strict: bool, path: Option<&Path>) -> i32 {
    let text = match read_input_text(path) {
        Ok(t) => t,
        Err((code, msg)) => {
            if let Some(m) = msg {
                eprintln!("{}", m);
            }
            return code;
        }
    };
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!("DATABASE_URL is not set");
            return 1;
        }
    };
    let store = match kotonoha_core::store::postgres::PgStore::connect(&url).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("database connection failed: {e}");
            return 3;
        }
    };
    use kotonoha_core::store::postgres::StoreError;

    match store.insert_interchange_document_json(&text, strict).await {
        Ok(id) => {
            println!("{}", id);
            0
        }
        Err(e) => {
            let code = match &e {
                StoreError::InterchangeValidation(_)
                | StoreError::RdeValidation(_)
                | StoreError::Lineage(_)
                | StoreError::SemanticLineage(_)
                | StoreError::MissingField(_)
                | StoreError::Json(_) => 2,
                StoreError::Sql(_) | StoreError::Migrate(_) => 3,
            };
            eprintln!("{}", e);
            code
        }
    }
}

fn cmd_interchange_validate(strict: bool, path: Option<&Path>) -> i32 {
    let text = match read_input_text(path) {
        Ok(t) => t,
        Err((code, msg)) => {
            if let Some(m) = msg {
                eprintln!("{}", m);
            }
            return code;
        }
    };
    match kotonoha_core::interchange::validate_interchange_json(&text, strict) {
        Ok(warnings) => {
            for w in warnings {
                eprintln!("warning: {}", w);
            }
            0
        }
        Err(e) => {
            eprintln!("{}", e);
            2
        }
    }
}

/// [`docs/cli-definition.md`] §4.1 — `kotonoha.console_event.v0` wrapper for console-equivalent ingest (Phase 3).
async fn cmd_interchange_ingest(strict: bool, persist: bool, path: Option<&Path>) -> i32 {
    let text = match read_input_text(path) {
        Ok(t) => t,
        Err((code, msg)) => {
            if let Some(m) = msg {
                eprintln!("{}", m);
            }
            return code;
        }
    };
    let (kind, body_json) = match parse_console_event_v0(&text) {
        Ok(x) => x,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };

    match kind.as_str() {
        "interchange.ingest.submitted" => {
            match kotonoha_core::interchange::validate_interchange_json(&body_json, strict) {
                Ok(warnings) => {
                    for w in warnings {
                        eprintln!("warning: {}", w);
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                    return 2;
                }
            }
            if persist {
                return cmd_interchange_store_from_envelope(&body_json, strict).await;
            }
            0
        }
        "rde.review.requested" => match kotonoha_core::rde::validate_json(&body_json, strict) {
            Ok(warnings) => {
                for w in warnings {
                    eprintln!("warning: {}", w);
                }
                0
            }
            Err(e) => {
                eprintln!("{}", e);
                2
            }
        },
        other => {
            eprintln!("unsupported console_event.kind for ingest: {other}");
            1
        }
    }
}

fn parse_console_event_v0(text: &str) -> Result<(String, String), String> {
    let root: Value = serde_json::from_str(text).map_err(|e| format!("invalid JSON: {e}"))?;
    let inner = root
        .get("console_event")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            "missing object \"console_event\" (see docs/cli-definition.md §4.1)".to_string()
        })?;
    let version = inner
        .get("version")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "console_event.version must be a string".to_string())?;
    if version != "kotonoha.console_event.v0" {
        return Err(format!(
            "unsupported console_event.version (expected \"kotonoha.console_event.v0\", got {version:?})"
        ));
    }
    let kind = inner
        .get("kind")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "console_event.kind must be a string".to_string())?
        .to_string();
    let body = inner
        .get("body")
        .ok_or_else(|| "missing console_event.body".to_string())?;
    let body_json = serde_json::to_string(body).map_err(|e| format!("console_event.body: {e}"))?;
    Ok((kind, body_json))
}

async fn cmd_interchange_store_from_envelope(envelope_json: &str, strict: bool) -> i32 {
    let url = match std::env::var("DATABASE_URL") {
        Ok(u) => u,
        Err(_) => {
            eprintln!("DATABASE_URL is not set");
            return 1;
        }
    };
    let store = match kotonoha_core::store::postgres::PgStore::connect(&url).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("database connection failed: {e}");
            return 3;
        }
    };
    use kotonoha_core::store::postgres::StoreError;
    match store
        .insert_interchange_document_json(envelope_json, strict)
        .await
    {
        Ok(id) => {
            println!("{}", id);
            0
        }
        Err(e) => {
            let code = match &e {
                StoreError::InterchangeValidation(_)
                | StoreError::RdeValidation(_)
                | StoreError::Lineage(_)
                | StoreError::SemanticLineage(_)
                | StoreError::MissingField(_)
                | StoreError::Json(_) => 2,
                StoreError::Sql(_) | StoreError::Migrate(_) => 3,
            };
            eprintln!("{}", e);
            code
        }
    }
}

/// Returns `(exit_code, Option<error_message>)` on I/O or UTF-8 failure.
fn read_input_text(path: Option<&Path>) -> Result<String, (i32, Option<String>)> {
    let mut buf = Vec::new();
    match load_input(path, &mut buf) {
        Ok(()) => {}
        Err(e) => return Err((1, Some(e))),
    }
    String::from_utf8(buf).map_err(|_| (1, Some("input is not valid UTF-8".to_string())))
}

fn load_input(path: Option<&Path>, buf: &mut Vec<u8>) -> Result<(), String> {
    match path {
        None => io::stdin()
            .read_to_end(buf)
            .map(|_| ())
            .map_err(|e| format!("read stdin: {e}")),
        Some(p) if p.as_os_str() == "-" => io::stdin()
            .read_to_end(buf)
            .map(|_| ())
            .map_err(|e| format!("read stdin: {e}")),
        Some(p) => std::fs::read(p)
            .map(|b| buf.extend(b))
            .map_err(|e| format!("{}: {e}", p.display())),
    }
}
