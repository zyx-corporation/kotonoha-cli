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
    };
    process::exit(code);
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
