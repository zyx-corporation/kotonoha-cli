# M5 MVP vs [`26`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/26_rde_llm_connection_design_draft.md) pattern A

**Parent:** [management#106](https://github.com/zyx-corporation/kotonoha-management/issues/106) · **M5-e** [#115](https://github.com/zyx-corporation/kotonoha-management/issues/115)

Pattern A (§4.1): LLM produces RDE draft → human reads → `kotonoha rde validate` → persist / attach.

## Convergence table

| `26` pattern A step | M5 MVP implementation | Notes |
| --- | --- | --- |
| LLM generates RDE draft JSON | MCP tools or manual `rde emit` / file | [`kotonoha-mcp`](https://github.com/zyx-corporation/kotonoha-mcp) ≥ 0.4.0 |
| Human reads meaning / responsibility | **Required** — Agent Approve UI + M3 + CLI | [`05`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/chatgpt-app/05_agent_approve_ui_draft.md) · #136 |
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
| Human approve in Agent channel | `kotonoha_review_{approve,hold,reject}` MCP | No `--agent-run-id` · #136 |

## Demo

| Script | Channel |
| --- | --- |
| [`m5_agent_run_demo.sh`](../scripts/m5_agent_run_demo.sh) | CLI |
| [`m5_mcp_e2e.ts`](https://github.com/zyx-corporation/kotonoha-mcp/blob/main/scripts/m5_mcp_e2e.ts) | MCP stdio（step 8 = `kotonoha_review_approve`） |

## References

- [`04_mcp_tools_and_ux.md`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/chatgpt-app/04_mcp_tools_and_ux.md) §6 E2E
- [`ui-design-review-m5.md`](ui-design-review-m5.md)
