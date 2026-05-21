//! M1/M2 export JSON builders (shared by `export` and `github pr-summary`).

use kotonoha_core::store::postgres::{MeaningDeltaRow, RdeAssessmentRow, ReviewDecisionRow};
use serde_json::Value;

pub const M1_EXPORT_FORMAT: &str = "kotonoha.m1_export.v0.1";
pub const M2_EXPORT_FORMAT: &str = "kotonoha.m2_export.v0.1";

pub fn m1_export_value(
    row: &MeaningDeltaRow,
    assessments: &[RdeAssessmentRow],
    decisions: &[ReviewDecisionRow],
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

pub fn m2_export_value(
    row: &MeaningDeltaRow,
    assessments: &[RdeAssessmentRow],
    decisions: &[ReviewDecisionRow],
) -> Value {
    let mut base = m1_export_value(row, assessments, decisions);
    let obj = base.as_object_mut().expect("export object");
    obj.insert("format".into(), Value::String(M2_EXPORT_FORMAT.into()));
    let hints = kotonoha_core::observation_rde::map_observation_to_rde_hints(&row.observation);
    obj.insert(
        "observation_rde_hints".into(),
        serde_json::to_value(&hints).unwrap_or(Value::Null),
    );
    let rde_assessments = assessments
        .iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "meaning_delta_id": a.meaning_delta_id,
                "payload": a.payload,
                "audit_correlation_id": a.audit_correlation_id,
                "rde_document_id": a.rde_document_id,
                "payload_schema_version": a.payload_schema_version,
                "source_kind": a.source_kind,
                "validation_report": a.validation_report,
            })
        })
        .collect::<Vec<_>>();
    obj.insert(
        "rde_assessments".into(),
        Value::Array(rde_assessments.into_iter().map(|v| v).collect()),
    );
    base
}

pub fn build_summary_paragraph(
    row: &MeaningDeltaRow,
    assessments: &[RdeAssessmentRow],
    decisions: &[ReviewDecisionRow],
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
