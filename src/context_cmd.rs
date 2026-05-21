//! `kotonoha context export` — M5 context pack (no database required).

use std::path::{Path, PathBuf};

use kotonoha_core::context_pack::{
    build_context_pack, validate_context_pack, BuildContextPackInput, MeaningDeltaDraft,
    CONTEXT_PACK_FORMAT,
};
use kotonoha_core::semantic_lineage::GitAnchor;
use serde_json::Value;

pub const M5_POLICY_REF: &str =
    "https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/31_m5_agent_run_integration_spec_draft.md";

#[derive(clap::Args)]
pub struct ContextExportArgs {
    #[arg(long, default_value = ".")]
    pub path: PathBuf,
    /// Anchor file (repo-relative or absolute).
    pub file: PathBuf,
    #[arg(long)]
    pub line_start: Option<i32>,
    #[arg(long)]
    pub line_end: Option<i32>,
    #[arg(long)]
    pub diff_ref: Option<String>,
    /// Observation JSON for `meaning_delta_draft` (omit for empty draft).
    #[arg(long)]
    pub observation: Option<PathBuf>,
    #[arg(long)]
    pub policy_ref: Option<String>,
}

pub fn run(args: &ContextExportArgs) -> i32 {
    let git = match kotonoha_core::git::discover_repo(Some(&args.path)) {
        Ok(c) => c,
        Err(kotonoha_core::git::GitError::NotARepository) => {
            eprintln!("context export requires a Git repository");
            return 1;
        }
        Err(e) => {
            eprintln!("{}", e);
            return 3;
        }
    };
    let rel = match kotonoha_core::git::path_relative_to_root(&git, &args.file) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e);
            return 1;
        }
    };
    let observation = match read_observation_json(args.observation.as_deref()) {
        Ok(v) => v,
        Err(code) => return code,
    };
    let diff_ref = args.diff_ref.clone().or_else(|| {
        if args.line_start.is_none() && args.line_end.is_none() {
            Some(format!("unstaged:{rel}"))
        } else {
            None
        }
    });
    let anchor = GitAnchor {
        git_commit: git.commit,
        file_path: rel,
        line_range_start: args.line_start,
        line_range_end: args.line_end,
        diff_ref,
    };
    if let Err(e) = anchor.validate() {
        eprintln!("git_anchor: {e}");
        return 2;
    }
    let has_draft =
        args.observation.is_some() || observation.as_object().is_none_or(|m| !m.is_empty());
    let meaning_delta_draft = has_draft.then(|| MeaningDeltaDraft {
        observation,
        source_context: Value::Object(serde_json::Map::new()),
    });
    let pack = build_context_pack(BuildContextPackInput {
        git_anchor: anchor,
        meaning_delta_draft,
        policy_ref: Some(
            args.policy_ref
                .clone()
                .unwrap_or_else(|| M5_POLICY_REF.to_string()),
        ),
    });
    if let Err(e) = validate_context_pack(&pack) {
        eprintln!("{}", e);
        return 2;
    }
    let pretty = match serde_json::to_string_pretty(&pack) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("context pack JSON: {e}");
            return 3;
        }
    };
    println!("{pretty}");
    if pack.format != CONTEXT_PACK_FORMAT {
        return 2;
    }
    0
}

fn read_observation_json(path: Option<&Path>) -> Result<Value, i32> {
    match path {
        None => Ok(Value::Object(serde_json::Map::new())),
        Some(p) => {
            let text = match crate::read_input_text(Some(p)) {
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
