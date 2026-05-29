# M7 Web Console — `kotonoha export --format m6` integration profile

**Status:** informative (non-normative implementation profile)  
**Audience:** `kotonoha-web-console` server authors, automation wrapping the CLI  
**Normative CLI contract:** [`cli-definition.md`](cli-definition.md)  
**Product context:** [management `37_m7_team_mode_ui_spec_draft.md`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/37_m7_team_mode_ui_spec_draft.md) · M7-b [#143](https://github.com/zyx-corporation/kotonoha-management/issues/143)

## Purpose

Web Console (and similar servers) obtain a **project-scoped audit JSON bundle** by invoking the same path as:

```bash
kotonoha export --format m6 --project-id <UUID> [--git-commit <SHA>] [--delta-id <UUID>] [--out FILE]
```

This document fixes **env, exit codes, stdout shape, and RBAC** so integrators do not rely on undocumented behaviour.

## Prerequisites

| Requirement | Notes |
| --- | --- |
| `DATABASE_URL` | PostgreSQL with M1+M6 migrations (`kotonoha db migrate`) |
| Git repository | Cwd **SHOULD** be the team repo root when using `--git-commit` |
| M6 schema | When present, RBAC is enforced (see below) |

## Environment variables

| Variable | Required | Effect |
| --- | --- | --- |
| `DATABASE_URL` | **Yes** | Connects to `PgStore` |
| `KOTONOHA_PRINCIPAL_ID` | **Yes** when M6 schema present | Acting principal UUID for RBAC |
| `KOTONOHA_PROJECT_ID` | Optional | Default for `--project-id` when flag omitted |

Invalid UUID strings in env vars are **ignored** (same as CLI implementation).

**Console recommendation:** set both `KOTONOHA_PRINCIPAL_ID` and `KOTONOHA_PROJECT_ID` on the child process to match the signed-in user and selected project.

## Invocation matrix

| Goal | Command |
| --- | --- |
| Full project audit (all deltas in project) | `kotonoha export --format m6 --project-id <PID>` |
| Audit for one commit (project-filtered) | `kotonoha export --format m6 --project-id <PID> --git-commit <SHA>` |
| Audit for one delta | `kotonoha export --format m6 --project-id <PID> --delta-id <UUID>` |

`--project-id` and `KOTONOHA_PROJECT_ID` are interchangeable; at least one **MUST** be set for `--format m6`.

## stdout JSON

Root object format id: **`kotonoha.m6_project_audit_export.v0.1`**

| Field | Type | Description |
| --- | --- | --- |
| `format` | string | Always `kotonoha.m6_project_audit_export.v0.1` |
| `generated_at_unix` | number | Unix seconds at export time |
| `project_id` | UUID string | Scoped project |
| `acting_principal_id` | UUID string or null | From `KOTONOHA_PRINCIPAL_ID` / store default |
| `git_commit` | string or null | Filter when `--git-commit` used |
| `export_count` | number | Length of `exports` |
| `exports` | array | Each element is a **`kotonoha.m2_export.v0.1`**-shaped object (meaning delta + RDE assessments + review decisions) |

Console **MAY** parse `export_count` and `exports[].meaning_delta` for read-only delta lists; **MUST NOT** treat this bundle as normative interchange in `kotonoha-spec`.

## Exit codes

| Code | Meaning | Examples |
| --- | --- | --- |
| **0** | Success; JSON on stdout (or written to `--out`) | |
| **1** | User / configuration error | Missing `DATABASE_URL`; missing `--project-id`; unknown `--format`; invalid flag combination |
| **2** | Validation or **access denied** | RBAC: `principal … lacks role 'viewer' on project …`; unknown delta; project mismatch |
| **3** | Database / internal error | Connection failure, SQL errors |

RBAC for M6 export requires role **`viewer`** or higher on the target project ([`kotonoha-core` `agent-schema-m6`](https://github.com/zyx-corporation/kotonoha-core/blob/main/docs/agent-schema-m6.md)).

## Example (server-side)

```bash
export DATABASE_URL='postgres://…'
export KOTONOHA_PRINCIPAL_ID='00000000-0000-4000-8000-000000000001'
export KOTONOHA_PROJECT_ID='00000000-0000-4000-8000-000000000002'

cd /path/to/team-repo
kotonoha export --format m6 --project-id "$KOTONOHA_PROJECT_ID" --git-commit "$(git rev-parse HEAD)"
```

## Smoke / regression

| Asset | Role |
| --- | --- |
| [`tests/m6_integration.rs`](../tests/m6_integration.rs) | Two projects on same commit → isolated `export_count` / observation |
| [`scripts/m7_export_smoke.sh`](../scripts/m7_export_smoke.sh) | Maintainer gate when `DATABASE_URL` is set |

## Related issues

- kotonoha-cli [#37](https://github.com/zyx-corporation/kotonoha-cli/issues/37) (M7 CLI epic)
- kotonoha-management [#139](https://github.com/zyx-corporation/kotonoha-management/issues/139) (M7 parent)
