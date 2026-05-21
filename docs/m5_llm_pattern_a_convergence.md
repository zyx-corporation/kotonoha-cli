# M5 MVP vs [`26`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/26_rde_llm_connection_design_draft.md) pattern A

**Parent:** [management#106](https://github.com/zyx-corporation/kotonoha-management/issues/106) · **M5-e** [#115](https://github.com/zyx-corporation/kotonoha-management/issues/115)

Pattern A (§4.1): LLM produces RDE draft → human reads → `kotonoha rde validate` → persist / attach.

## Convergence table

| `26` pattern A step | M5 MVP implementation | Notes |
| --- | --- | --- |
| LLM generates RDE draft JSON | Channel (future MCP) or manual `rde emit` / file | MVP: **CLI demo** only — **P1-a:** [`33`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/33_m5_channel_operations_followup_plan_draft.md) §4 |
| Human reads meaning / responsibility | **Required** — not automated in M5 | VS Code M3 or operator review |
| `kotonoha rde validate [--strict]` | `kotonoha rde validate --strict` | Unchanged; MCP tool `kotonoha_rde_validate` |
| Persist / attach to lineage | `rde attach --source-kind llm` + `agent delta create` | AgentRun links via `meaning_deltas.agent_run_id` |
| Trust boundary after LLM output | `validate` before attach; invalid → exit **2** | Same as M2 |
| Gateway optional | **`kotonoha` CLI** is Gateway for MVP | Independent `kotonoha-gateway` is P2 per [`31` §7.4](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/31_m5_agent_run_integration_spec_draft.md) |

## M5 additions (not in `26` §4.1 alone)

| Capability | Artifact |
| --- | --- |
| Context for LLM prompt | `kotonoha context export` → `kotonoha.context_pack.v0.1` |
| Agent accountability | `agent_runs` + `denied_actions` |
| Deny human-only actions on agent path | `review *` with `--agent-run-id` / `KOTONOHA_AGENT_RUN_ID` |

## Demo

[`scripts/m5_agent_run_demo.sh`](../scripts/m5_agent_run_demo.sh) — end-to-end Pattern A + AgentRun + capability deny.

## References

- [`04_mcp_tools_and_ux.md`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/chatgpt-app/04_mcp_tools_and_ux.md) §6 E2E
- [`ui-design-review-m5.md`](ui-design-review-m5.md)
