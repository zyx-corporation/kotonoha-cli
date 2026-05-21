//! `kotonoha agent` — AgentRun record and agent-scoped MeaningDelta (M5-c).

use std::path::{Path, PathBuf};

use clap::Subcommand;
use kotonoha_core::semantic_lineage::{GitAnchor, MeaningDeltaInput};
use kotonoha_core::store::AgentRunStatus;
use serde_json::Value;

use crate::capability::{default_start_input, deny_if_agent_context, is_denied_in_agent_context, PROFILE_AGENT};

#[derive(Subcommand)]
pub enum AgentAction {
    /// AgentRun lifecycle (start / complete).
    Record {
        #[command(subcommand)]
        action: AgentRecordAction,
    },
    /// Create MeaningDelta linked to an AgentRun (`meaning_deltas.agent_run_id`).
    Delta {
        #[command(subcommand)]
        action: AgentDeltaAction,
    },
    /// Explicit capability probe for agent channel (M5-P1b-1).
    Capability {
        #[command(subcommand)]
        action: AgentCapabilityAction,
    },
}

#[derive(Subcommand)]
pub enum AgentCapabilityAction {
    /// Check whether `ACTION` is allowed for `--agent-run-id` (deny → exit **2** + `denied_actions`).
    Check {
        #[arg(long)]
        action: String,
        #[arg(long)]
        agent_run_id: uuid::Uuid,
    },
}

#[derive(Subcommand)]
pub enum AgentRecordAction {
    /// Insert `agent_runs` with `status = started`.
    Start {
        #[arg(long, default_value = "cli")]
        agent_kind: String,
        #[arg(long)]
        external_ref: Option<String>,
        #[arg(long, default_value = PROFILE_AGENT)]
        capability_profile: String,
        #[arg(long)]
        parent_run_id: Option<uuid::Uuid>,
        /// Optional JSON payload file (default `{}`).
        #[arg(long)]
        payload: Option<PathBuf>,
    },
    /// Set AgentRun `status = completed`.
    Complete {
        #[arg(long)]
        run_id: uuid::Uuid,
        /// JSON array of artifact refs (default `[]`).
        #[arg(long)]
        output_artifacts: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum AgentDeltaAction {
    /// Same as `delta create` but requires `--agent-run-id`.
    Create {
        file: PathBuf,
        #[arg(long, default_value = ".")]
        path: PathBuf,
        #[arg(long)]
        agent_run_id: uuid::Uuid,
        #[arg(long)]
        line_start: Option<i32>,
        #[arg(long)]
        line_end: Option<i32>,
        #[arg(long)]
        diff_ref: Option<String>,
        #[arg(long)]
        observation: Option<PathBuf>,
    },
}

pub async fn run(action: AgentAction) -> i32 {
    match action {
        AgentAction::Record { action } => match action {
            AgentRecordAction::Start {
                agent_kind,
                external_ref,
                capability_profile,
                parent_run_id,
                payload,
            } => {
                cmd_agent_record_start(
                    &agent_kind,
                    external_ref.as_deref(),
                    &capability_profile,
                    parent_run_id,
                    payload.as_deref(),
                )
                .await
            }
            AgentRecordAction::Complete {
                run_id,
                output_artifacts,
            } => cmd_agent_record_complete(run_id, output_artifacts.as_deref()).await,
        },
        AgentAction::Capability { action } => match action {
            AgentCapabilityAction::Check {
                action,
                agent_run_id,
            } => cmd_agent_capability_check(&action, agent_run_id).await,
        },
        AgentAction::Delta { action } => match action {
            AgentDeltaAction::Create {
                file,
                path,
                agent_run_id,
                line_start,
                line_end,
                diff_ref,
                observation,
            } => {
                cmd_agent_delta_create(
                    &path,
                    &file,
                    agent_run_id,
                    line_start,
                    line_end,
                    diff_ref,
                    observation.as_deref(),
                )
                .await
            }
        },
    }
}

async fn cmd_agent_capability_check(action: &str, agent_run_id: uuid::Uuid) -> i32 {
    let store = match crate::pg_store().await {
        Ok(s) => s,
        Err(c) => return c,
    };
    if !store.m5_schema_present().await.unwrap_or(false) {
        eprintln!("M5 agent_runs schema missing — run `kotonoha db migrate`");
        return 1;
    }
    let run = match store.get_agent_run(agent_run_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            eprintln!("agent run not found: {agent_run_id}");
            return 2;
        }
        Err(e) => {
            eprintln!("{}", e);
            return crate::store_error_code(&e);
        }
    };
    if is_denied_in_agent_context(action) {
        return match deny_if_agent_context(&store, agent_run_id, action).await {
            Ok(()) => 0,
            Err(code) => code,
        };
    }
    let profile = run
        .capability_profile
        .clone()
        .unwrap_or_else(|| PROFILE_AGENT.to_string());
    let payload = serde_json::json!({
        "format": "kotonoha.agent_capability_check.v0.1",
        "allowed": true,
        "action": action,
        "agent_run_id": agent_run_id,
        "capability_profile": profile,
    });
    println!("{}", serde_json::to_string_pretty(&payload).unwrap_or_default());
    0
}

async fn cmd_agent_record_start(
    agent_kind: &str,
    external_ref: Option<&str>,
    capability_profile: &str,
    parent_run_id: Option<uuid::Uuid>,
    payload_path: Option<&Path>,
) -> i32 {
    let payload = match read_json_value(payload_path) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let store = match crate::pg_store().await {
        Ok(s) => s,
        Err(c) => return c,
    };
    if !store.m5_schema_present().await.unwrap_or(false) {
        eprintln!("M5 agent_runs schema missing — run `kotonoha db migrate`");
        return 1;
    }
    let mut input = default_start_input(agent_kind);
    input.external_ref = external_ref.map(str::to_string);
    input.capability_profile = Some(capability_profile.to_string());
    input.parent_run_id = parent_run_id;
    input.payload = payload;
    match store.start_agent_run(&input).await {
        Ok(run) => {
            println!("{}", run.id);
            0
        }
        Err(e) => {
            eprintln!("{}", e);
            crate::store_error_code(&e)
        }
    }
}

async fn cmd_agent_record_complete(run_id: uuid::Uuid, artifacts_path: Option<&Path>) -> i32 {
    let artifacts = match read_json_value(artifacts_path) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let store = match crate::pg_store().await {
        Ok(s) => s,
        Err(c) => return c,
    };
    if !artifacts.is_null() {
        if let Err(e) = store
            .set_agent_run_output_artifacts(run_id, &artifacts)
            .await
        {
            eprintln!("{}", e);
            return crate::store_error_code(&e);
        }
    }
    match store
        .update_agent_run_status(run_id, AgentRunStatus::Completed)
        .await
    {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("{}", e);
            crate::store_error_code(&e)
        }
    }
}

async fn cmd_agent_delta_create(
    repo_path: &Path,
    file: &Path,
    agent_run_id: uuid::Uuid,
    line_start: Option<i32>,
    line_end: Option<i32>,
    diff_ref: Option<String>,
    observation_path: Option<&Path>,
) -> i32 {
    let store = match crate::pg_store().await {
        Ok(s) => s,
        Err(c) => return c,
    };
    if store
        .get_agent_run(agent_run_id)
        .await
        .ok()
        .flatten()
        .is_none()
    {
        eprintln!("agent run not found: {agent_run_id}");
        return 2;
    }
    let git = match kotonoha_core::git::discover_repo(Some(repo_path)) {
        Ok(c) => c,
        Err(kotonoha_core::git::GitError::NotARepository) => {
            eprintln!("agent delta create requires a Git repository");
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
    let input = MeaningDeltaInput {
        document_object_id: None,
        prior_meaning_state_id: None,
        new_meaning_state_id: None,
        agent_run_id: Some(agent_run_id),
        git_anchor: GitAnchor {
            git_commit: git.commit,
            file_path: rel,
            line_range_start: line_start,
            line_range_end: line_end,
            diff_ref,
        },
        observation,
        source_context: Value::Object(Default::default()),
    };
    match store.create_meaning_delta(&input).await {
        Ok(id) => {
            println!("{}", id);
            0
        }
        Err(e) => {
            eprintln!("{}", e);
            crate::store_error_code(&e)
        }
    }
}

fn read_json_value(path: Option<&Path>) -> Result<Value, i32> {
    match path {
        None => Ok(Value::Object(serde_json::Map::new())),
        Some(p) => {
            let text = crate::read_input_text(Some(p)).map_err(|(c, msg)| {
                if let Some(m) = msg {
                    eprintln!("{}", m);
                }
                c
            })?;
            serde_json::from_str(&text).map_err(|e| {
                eprintln!("JSON: {e}");
                2
            })
        }
    }
}

fn read_observation_json(path: Option<&Path>) -> Result<Value, i32> {
    read_json_value(path)
}

/// Resolves AgentRun context from flag or `KOTONOHA_AGENT_RUN_ID`.
pub fn resolve_agent_run_id(flag: Option<uuid::Uuid>) -> Option<uuid::Uuid> {
    if flag.is_some() {
        return flag;
    }
    std::env::var("KOTONOHA_AGENT_RUN_ID")
        .ok()
        .and_then(|s| uuid::Uuid::parse_str(s.trim()).ok())
}
