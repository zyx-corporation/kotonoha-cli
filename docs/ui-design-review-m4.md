# M4 UI design review record

**Normative checklist:** [kotonoha-management `32` §2.2](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/32_milestone_ui_quality_gates_draft.md)

**Product spec:** [`30_m4_github_integration_spec_draft.md`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/30_m4_github_integration_spec_draft.md) §5.5

**Parent:** [management#105](https://github.com/zyx-corporation/kotonoha-management/issues/105)

| Field | Value |
| --- | --- |
| **Date** | 2026-05-21 |
| **Reviewer** | M4 gate verification (local + CI) |
| **CLI** | kotonoha 0.2.5 · kotonoha-core 0.1.10 |
| **Judgment** | **Pass with notes** |

M4 UI scope: Issue/PR Markdown templates and `kotonoha github pr-summary` sections (not IDE panels).

## D1 — Information design

| Result | Notes |
| --- | --- |
| **Pass** | `pr-summary` sections separate ΔM list, RDE counts, and human-responsibility disclaimer. Issue templates (`issue_body_en.md` / `issue_body_ja.md`) use Preserved / Transformed / Unresolved / Next action structure. |

## D2 — Operation flow

| Result | Notes |
| --- | --- |
| **Pass** | Documented CLI path: `gh-status` → `link repo` → `link issue|pr` → `list-pr` → `pr-summary` → paste into PR. `scripts/m4_github_demo.sh` demonstrates the flow. |

## D3 — Accountability boundary

| Result | Notes |
| --- | --- |
| **Pass** | Banner in en/ja templates and `pr-summary` header (`RDE` / human judgment wording per [`19`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/19_rde_review_operating_policy_outline_draft.md)). |

## D4 — Error experience

| Result | Notes |
| --- | --- |
| **Pass with notes** | CLI exit codes 1/2/3 documented in `docs/github-integration.md`. `gh` missing / DB errors return non-zero with stderr. |

## D5 — Wireframe alignment

| Result | Notes |
| --- | --- |
| **Pass with notes** | No M4-specific wireframe; aligned with spec §5.5 table. Intentional gap: `pr-summary --locale ja` may leave ΔM description lines in DB language (English). |

## M4 §6 functional gate (cross-reference)

| Criterion | Result |
| --- | --- |
| PR ΔM list (`list-pr`) | Pass — local `m4_github_demo.sh` |
| RDE in PR review (`pr-summary`) | Pass — en/ja |
| Issue re-registration (link + templates) | Pass — `link issue`; `open-issue` P1 not implemented |
| CI semantic check | Pass — `semantic-check.yml` on [cli#25](https://github.com/zyx-corporation/kotonoha-cli/pull/25) |
| `github_*_links` + FK docs | Pass — [core `github-schema-m4.md`](https://github.com/zyx-corporation/kotonoha-core/blob/main/docs/github-schema-m4.md) |

## Follow-up issues (if Pass with notes)

- Localize ΔM description lines in `pr-summary --locale ja` (observation text from DB).
- P1: `kotonoha github open-issue` (template-driven issue creation).
- Update [`m4_phase3_convergence_table.md`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/m4_phase3_convergence_table.md) M4-b/c status to **完了** (mgmt PR optional).
