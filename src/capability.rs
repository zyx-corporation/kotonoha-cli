//! M5 capability deny-by-default for agent-channel invocations.
//!
//! Spec: [`31_m5_agent_run_integration_spec_draft.md`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/31_m5_agent_run_integration_spec_draft.md) §6.1

use kotonoha_core::semantic_lineage::ReviewDecisionKind;
use kotonoha_core::store::postgres::PgStore;
use kotonoha_core::store::{AgentRunRow, DeniedActionRecord, StartAgentRunInput};

/// Default profile for `kotonoha agent record start`.
pub const PROFILE_AGENT: &str = "kotonoha-agent";
/// Read-only agent tools (context export, rde validate). Reserved for MCP `kotonoha-readonly`.
#[allow(dead_code)]
pub const PROFILE_READONLY: &str = "kotonoha-readonly";

pub const ACTION_REVIEW_APPROVE: &str = "review.approve";
pub const ACTION_REVIEW_HOLD: &str = "review.hold";
pub const ACTION_REVIEW_REJECT: &str = "review.reject";
pub const ACTION_REVIEW_NEEDS_REVISION: &str = "review.needs_revision";
pub const ACTION_GIT_PUSH: &str = "git.push";
pub const ACTION_GIT_COMMIT: &str = "git.commit";
pub const ACTION_SHELL: &str = "shell";

const DENIED_FOR_AGENT_CHANNEL: &[&str] = &[
    ACTION_REVIEW_APPROVE,
    ACTION_REVIEW_HOLD,
    ACTION_REVIEW_REJECT,
    ACTION_REVIEW_NEEDS_REVISION,
    ACTION_GIT_PUSH,
    ACTION_GIT_COMMIT,
    ACTION_SHELL,
];

/// Whether an action is blocked when invoked with an active AgentRun context.
pub fn is_denied_in_agent_context(action: &str) -> bool {
    DENIED_FOR_AGENT_CHANNEL.contains(&action)
}

pub fn action_for_review(decision: ReviewDecisionKind) -> &'static str {
    match decision {
        ReviewDecisionKind::Approve => ACTION_REVIEW_APPROVE,
        ReviewDecisionKind::Hold => ACTION_REVIEW_HOLD,
        ReviewDecisionKind::Reject => ACTION_REVIEW_REJECT,
        ReviewDecisionKind::NeedsRevision => ACTION_REVIEW_NEEDS_REVISION,
    }
}

fn profile_label(run: &AgentRunRow) -> String {
    run.capability_profile
        .clone()
        .unwrap_or_else(|| PROFILE_AGENT.to_string())
}

/// Records deny in `denied_actions` and returns exit code **2** when blocked.
pub async fn deny_if_agent_context(
    store: &PgStore,
    run_id: uuid::Uuid,
    action: &str,
) -> Result<(), i32> {
    if !is_denied_in_agent_context(action) {
        return Ok(());
    }
    let run = match store.get_agent_run(run_id).await {
        Ok(Some(r)) => r,
        Ok(None) => {
            eprintln!("agent run not found: {run_id}");
            return Err(2);
        }
        Err(e) => {
            eprintln!("{}", e);
            return Err(3);
        }
    };
    let profile = profile_label(&run);
    let reason = format!("deny-by-default for agent channel (profile {profile})");
    let record = DeniedActionRecord {
        action: action.to_string(),
        reason: reason.clone(),
        profile: Some(profile.clone()),
    };
    if let Err(e) = store.append_agent_run_denied_action(run_id, &record).await {
        eprintln!("{}", e);
        return Err(3);
    }
    eprintln!(
        "Action `{action}` is not allowed for agent profile `{profile}`. Recorded in AgentRun denied_actions."
    );
    eprintln!(
        "エージェントプロファイル `{profile}` では `{action}` は許可されていません。AgentRun の denied_actions に記録しました。"
    );
    Err(2)
}

/// Default [`StartAgentRunInput`] for CLI demos and MCP delegation.
pub fn default_start_input(agent_kind: &str) -> StartAgentRunInput {
    StartAgentRunInput {
        agent_kind: agent_kind.to_string(),
        external_ref: None,
        capability_profile: Some(PROFILE_AGENT.to_string()),
        parent_run_id: None,
        payload: serde_json::Value::Object(serde_json::Map::new()),
        principal_id: None,
    }
}
