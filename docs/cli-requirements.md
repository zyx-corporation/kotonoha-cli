# `kotonoha` CLI — requirements specification (Phase 3)

This document collects **requirements and acceptance cues** for the command-line interface. It is **not** the runtime contract surface: authoritative command behaviour, signatures, and exit semantics for releases are defined in **`[cli-definition.md](cli-definition.md)`**.  

When implementation meets a requirement here that is not yet spelled out in `cli-definition.md`, **update both documents in the same change set** (PR + CI).

Tracking: **[Issue #4](https://github.com/zyx-corporation/kotonoha-cli/issues/4)**. Broader Phase 3 execution / integration detail is tracked in organization **internal issues and drafts** (not hyperlinked from this public repository).

---

## 1. Document roles

| Document | Role |
| --- | --- |
| **`cli-definition.md`** | Stable **public definition** — what scripts and users MAY rely on (flags, exit codes, stdout shape). |
| **`cli-requirements.md` (this file)** | **Intent and backlog** — what we still need Phase 3 to satisfy, projections to semantic error classes, and traceability/evidence pointers. |

---

## 2. Normative layering

Requirements **MUST NOT** redefine `kotonoha-spec` prose. Semantic truth remains in **`kotonoha-spec`**. Implementation gaps escalate per repository and **public** governance/escalation paths described in **`kotonoha-spec`** and contributor docs (private playbooks stay off this file).

---

## 3. Functional requirements — baseline (maintained behaviour)

Identifiers are stable for Issue/PR linkage. “Met” implies covered in **`cli-definition.md`** for the cited release band.

| ID | Requirement | Evidence |
| --- | --- | --- |
| **REQ-CLI-001** | Expose **`kotonoha`** entrypoint identity and **`kotonoha version`** with build + targeted spec bundle line. | [`cli-definition.md`](cli-definition.md) §4 |
| **REQ-CLI-010** | Provide **`rde validate` / `rde emit`** for Phase 2 RDE interchange JSON (`spec_version` **0.1** semantics). stdin / path parity. | §4 |
| **REQ-CLI-011** | Provide **`interchange validate` / `emit` / `store`** for **`kotonoha.interchange.v1`** envelopes; **`store`** persists via **`kotonoha_core`** when **`DATABASE_URL`** is set. | §4–§5 |
| **REQ-CLI-012** | Apply Postgres migrations via **`kotonoha db migrate`** using core migrations when **`DATABASE_URL`** available. | §4 |
| **REQ-CLI-020** | Keep **minimum exit codes `0–3`** contract documented and stable unless a semver-major change is intentional (see REQ-CLI-021). | Exit-code table |
| **REQ-CLI-021** | Any script-visible exit-code change requires **semver + CHANGELOG + matrix update** together. | §7 |

---

## 4. Phase 3 — ingest path and symmetry (Milestone M3.3)

Path **A/B** (ingest strategy) is decided under organization maintainer tracking **outside this public repo**. If **path A** (CLI-first) proceeds:

| ID | Requirement | Notes |
| --- | --- | --- |
| **REQ-CLI-050** | Support **deterministic ingestion** of console-equivalent payloads with **no less validation** than `interchange validate` for the envelope shape (JSON file or stdin). | **Met (≥ 0.2.0):** **`kotonoha interchange ingest`** + **`kotonoha.console_event.v0`** — [`cli-definition.md`](cli-definition.md) §4.1. |
| **REQ-CLI-051** | Emit machine-usable differentiation between **validation shape vs semantic rejection vs persistence/environment failure** (aligned with the informative [**`kotonoha-core` gap memo**](https://github.com/zyx-corporation/kotonoha-core/blob/main/docs/core-console-contract-gap-phase3-draft.md) and future **`cli-definition.md`** updates). **Table TBD** until maintainers publish a stable v0 mapping. | Allowed interim: documented exit **`2`** split via stderr prefixes **only if** scripted in `CHANGELOG` + `cli-definition`. Prefer stable integer map after review. |
| **REQ-CLI-052** | Preserve **`phase2_acceptance_demo`** and CI behaviours; extend demo or CI when Phase 3 adds user-visible paths. | See [`scripts/phase2_acceptance_demo.sh`](../scripts/phase2_acceptance_demo.sh). |

---

## 5. Non-functional requirements

| ID | Requirement |
| --- | --- |
| **REQ-CLI-N01** | **Stderr vs stdout**: diagnostic detail on stderr; machine-oriented single-line payloads on stdout only where `cli-definition` promises it (e.g. UUID from `store`). |
| **REQ-CLI-N02** | **Traceability**: **`cli-definition.md`** §6 matrix MUST include any new externally visible command tied to a spec anchor or an explicit _(not normative)_ core pointer. [`kotonoha-core` `spec-traceability.md`](https://github.com/zyx-corporation/kotonoha-core/blob/main/docs/spec-traceability.md). |
| **REQ-CLI-N03** | **Security posture (Phase 3 scope)**: no interactive secret prompts in baseline; **`DATABASE_URL`** is environment supplied. Fine-grained IAM is out of CLI scope for the Phase 3 baseline. |

---

## 6. Acceptance (internal cues)

Minimum evidence for Phase 3 CLI slice:

1. PR references the **Phase 3 execution tracker** used by maintainers (internal issue list) stating which REQ IDs moved—do not embed private URLs in public threads.
2. Updated **`cli-definition.md`** signatures for anything script-visible.
3. Green CI on **`kotonoha-cli`** `main` for the merge commit.

---

## Changelog (document level)

| Date | Change |
| --- | --- |
| 2026-05-10 | Initial requirements draft (**Issue #4**); Phase 3 pointers avoid private-repository hyperlinks in this OSS copy (**#37**). |
| 2026-05-12 | **REQ-CLI-050** marked met for **`kotonoha` ≥ 0.2.0** (`interchange ingest`, `cli-definition` §4.1). |
