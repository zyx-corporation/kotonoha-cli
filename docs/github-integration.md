# GitHub integration (M4)

**Normative product spec:** [kotonoha-management `30_m4_github_integration_spec_draft.md`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/30_m4_github_integration_spec_draft.md)

**Core schema:** [kotonoha-core `github-schema-m4.md`](https://github.com/zyx-corporation/kotonoha-core/blob/main/docs/github-schema-m4.md)

## Prerequisites

| Item | Note |
| --- | --- |
| PostgreSQL | `DATABASE_URL` + `kotonoha db migrate` (includes M4 tables) |
| `kotonoha_core` | ≥ 0.1.10 (GitHub link DDL) |
| GitHub CLI | `gh auth login` for `pr view` head SHA and optional issue creation |
| Git remote | `origin` pointing at `github.com/owner/repo` when `--owner` / `--repo` omitted |

## Commands

```bash
# Auth check (exit 1 if gh missing or not logged in)
kotonoha github gh-status

# Register owner/repo in DB
kotonoha github link repo --path .

# Link MeaningDelta to Issue / PR
kotonoha github link issue --delta-id UUID --issue-number 42
kotonoha github link pr --delta-id UUID --pr-number 7

# List deltas for a PR (uses DB links + optional head SHA from gh)
kotonoha github list-pr --pr-number 7 --json

# Markdown for PR body or review comment (en | ja)
kotonoha github pr-summary --pr-number 7 --locale ja
kotonoha github pr-summary --pr-number 7 --delta-id UUID --locale en
```

Exit codes match [`cli-definition.md`](cli-definition.md) §3: **1** environment, **2** validation, **3** database.

## CI — semantic check

Workflow template: [`.github/workflows/semantic-check.yml`](../.github/workflows/semantic-check.yml)

Consumer repos should copy the workflow and add JSON fixtures under `examples/interchange/` or adjust the validate step.

## Issue / PR templates (i18n)

| Locale | Intent-gap issue body |
| --- | --- |
| English | [`docs/templates/github/issue_body_en.md`](templates/github/issue_body_en.md) |
| Japanese | [`docs/templates/github/issue_body_ja.md`](templates/github/issue_body_ja.md) |

Paste `kotonoha github pr-summary` output into PR descriptions; templates include the human-accountability banner required by [`32` UI quality gates](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/32_milestone_ui_quality_gates_draft.md).

## Demo

[`scripts/m4_github_demo.sh`](../scripts/m4_github_demo.sh) — local DB path (skips `gh` when unavailable).

## Issues

- CLI: [kotonoha-cli#21](https://github.com/zyx-corporation/kotonoha-cli/issues/21)
- GHA: [kotonoha-cli#22](https://github.com/zyx-corporation/kotonoha-cli/issues/22)
- Parent: [kotonoha-management#105](https://github.com/zyx-corporation/kotonoha-management/issues/105)
