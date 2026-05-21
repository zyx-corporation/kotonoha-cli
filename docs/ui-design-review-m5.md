# M5 UI design review record

**Normative checklist:** [kotonoha-management `32` §2.2](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/32_milestone_ui_quality_gates_draft.md)

**Product spec:** [`31_m5_agent_run_integration_spec_draft.md`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/31_m5_agent_run_integration_spec_draft.md) §7.5

**MCP/UX normative:** [`04_mcp_tools_and_ux.md`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/chatgpt-app/04_mcp_tools_and_ux.md)

**Parent:** [management#106](https://github.com/zyx-corporation/kotonoha-management/issues/106)

| Field | Value |
| --- | --- |
| **Date** | 2026-05-21 |
| **Reviewer** | M5 gate verification (local + `tests/m5_integration.rs`) |
| **CLI** | kotonoha 0.2.6 · kotonoha-core 0.1.12 |
| **Judgment** | **Pass with notes** |

M5 UI scope: MCP tool names/descriptions, capability-deny stderr (en/ja), exit-code mapping — **not** VS Code panels (M3) or ChatGPT host UI.

## D1 — Information design

| Result | Notes |
| --- | --- |
| **Pass** | Tool names encode role: `kotonoha_context_export`, `kotonoha_agent_record_*`, `kotonoha_rde_*`. CLI subcommands mirror names (`context export`, `agent record`, `agent delta create`). |

## D2 — Operation flow

| Result | Notes |
| --- | --- |
| **Pass** | Documented sequence in [`04_mcp_tools_and_ux.md` §2](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/chatgpt-app/04_mcp_tools_and_ux.md). Automated: [`scripts/m5_agent_run_demo.sh`](../scripts/m5_agent_run_demo.sh). |

## D3 — Accountability boundary

| Result | Notes |
| --- | --- |
| **Pass** | §4.1 human-responsibility banner (en/ja). Agent path `review approve` denied; human path succeeds. Matches [`19`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/19_rde_review_operating_policy_outline_draft.md). |

## D4 — Error experience

| Result | Notes |
| --- | --- |
| **Pass** | Exit **1** (`DATABASE_URL`), **2** (validation / capability deny), **3** (DB) documented in [`04` §4.5](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/chatgpt-app/04_mcp_tools_and_ux.md) and `docs/cli-definition.md` M5 section. Agent deny prints bilingual template (§4.2). |

## D5 — Wireframe alignment

| Result | Notes |
| --- | --- |
| **Pass with notes** | No ChatGPT widget wireframe in M5 MVP. Intentional gap vs M3 VS Code panels (§1 boundary table in `04`). |

## M5 §8 functional gate (cross-reference)

| Criterion | Result |
| --- | --- |
| AgentRun record | Pass — `agent record start\|complete` |
| MeaningDelta from run | Pass — `agent delta create --agent-run-id` |
| RDE validate + attach (`llm`) | Pass — demo + m2 path |
| Capability deny + `denied_actions` | Pass — `review approve --agent-run-id` exit **2** |
| Context pack export | Pass — `kotonoha.context_pack.v0.1` |
| i18n (§32 §1) | Pass — `04` §4 en/ja templates |
| D1–D5 | Pass with notes (this file) |

## Follow-up (P1/P2)

- ChatGPT Apps SDK MCP server implementation repo (tools delegate to CLI).
- MCP descriptor `description_ja` fields in implementation.
- Widget for validated RDE summary (optional minimal).
