# `kotonoha` CLI — public definition

This document is the **authoritative definition** of the command-line interface shipped from this repository. Implementations **MUST** conform to it unless an explicit exception is documented in [CHANGELOG.md](../CHANGELOG.md).

**Related:** backlog and Phase 3 intent are captured in **`[cli-requirements.md](cli-requirements.md)`**; substantive behaviour changes MUST keep both documents consistent.

Behaviour that implements SLS semantics **MUST** remain traceable to [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec). If this document conflicts with the specification, **`kotonoha-spec` wins**.

## 1. Executable identity

- **Name:** the installed executable **MUST** be invocable as **`kotonoha`** on supported platforms (platform packaging MAY add a prefix or suffix only when convention requires it; such exceptions **MUST** be documented in this repository).
- **Purpose:** provide a **developer-facing** interface for validating and manipulating **interchange** artifacts defined by `kotonoha-spec`, and for future lineage operations, without replacing normative specifications.

## 2. Scope (Phase 2 minimum)

For organizational **Phase 2** (see internal phase plan in project governance), the CLI definition **MUST** cover at least:

| Area | Requirement |
| --- | --- |
| **Identity** | `kotonoha --version` (or equivalent) reports the CLI build identity and **SHOULD** report the [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) bundle version the build targets (when known). |
| **RDE interchange** | Commands **MUST** exist to **validate** and/or **emit** JSON aligned with `docs/rde-review-output.md` in `kotonoha-spec` (`spec_version` **0.1** minimum). Exact subcommand names **MAY** evolve; see §4. |
| **Traceability** | Documentation in this repository **MUST** map commands and flags to specification sections (see §6). |

From **release 0.2.0**, the CLI **MAY** additionally expose **Phase 3** ingest paths documented in §4.1 (console-equivalent JSON **without** changing `kotonoha-spec` normative prose).

Implementations **MAY** ship additional commands; they **MUST** be listed in this document when stable.

## 3. Non-goals (Phase 2)

- Replacing Git, issue trackers, or project boards.
- Defining new normative interchange fields beyond what `kotonoha-spec` requires.
- Performing automated **approval** or **authorization** of human decisions.

## 4. Command surface (initial)

Subcommand groups:

| Group | Intent |
| --- | --- |
| `kotonoha version` | Report CLI and targeted specification compatibility. |
| `kotonoha db` | Apply **PostgreSQL** migrations shipped with `kotonoha-core` (requires `DATABASE_URL`). |
| `kotonoha rde` | Operate on **RDE review output** interchange (validate JSON, emit skeleton). |
| `kotonoha interchange` | Validate / emit / **store** / **ingest** **core interchange envelope** JSON (`kotonoha.interchange.v1`) — bundles optional lineage +/or RDE for pipelines (**not** normative in `kotonoha-spec`). **`ingest`** (≥ **0.2.0**) accepts a **Phase 3** `console_event` wrapper (§4.1). |
| `kotonoha init` | Create **`.kotonoha/config.toml`** in a Git repo (M1 workspace bootstrap). |
| `kotonoha status` | Print Git context, project config, and optional DB summary (`DATABASE_URL`). |
| `kotonoha diff` | Print **unstaged** `git diff` (optional file scope). |
| `kotonoha delta` | Create **MeaningDelta** rows anchored to Git (`DATABASE_URL` + M1 schema). |
| `kotonoha rde attach` | Attach validated RDE JSON to an existing MeaningDelta (`rde_assessments`). |
| `kotonoha review` | Record human **approve** / **hold** / **reject** (`review_decisions`). |
| `kotonoha export` | JSON audit report (MeaningDelta + RDE + decisions) by `--delta-id` or `--git-commit`. |

### Concrete signatures (Phase 2 baseline — **0.1.x**)

| Invocation | Behaviour |
| --- | --- |
| `kotonoha` | Prints help (same as `--help`). Global `--version` prints short version via Clap. |
| `kotonoha version` | Prints **two lines** to stdout: (1) `kotonoha <crate semver>` (2) `kotonoha-spec (target bundle): 0.1`. Exit **0**. |
| `kotonoha db migrate` | Reads **`DATABASE_URL`**, connects with SQLx, applies migrations from `kotonoha-core/migrations`. Missing env → exit **1**. Connection / migration failure → exit **3**. Success prints one confirmation line to stdout → exit **0**. |
| `kotonoha rde validate [--strict] [PATH]` | Reads JSON from **PATH**, or from **stdin** when PATH is omitted or `-`. Validates Phase 1 interchange (`spec_version` **MUST** be `0.1`). Items missing `summary` emit **warnings** on stderr unless `--strict`, then exit **2**. Malformed args / unreadable file / invalid UTF-8 → exit **1**. Validation failure → exit **2**. Success → exit **0**. |
| `kotonoha rde emit` | Writes a **minimal compliant** JSON skeleton (pretty-printed) to stdout. Exit **0**. |
| `kotonoha interchange validate [--strict] [PATH]` | Validates **`kotonoha.interchange.v1`** envelope (`format`, `spec_bundle`, optional `lineage_unit`, optional `rde_document`). Nested RDE uses the same `--strict` semantics as `rde validate`. **Unknown JSON keys:** rejected when validation uses **`kotonoha_core`** **≥ 0.1.6** — only the four top‑level envelope properties are accepted; **`lineage_unit`** objects accept **`id` / `prior_unit_id` only** (serde `deny_unknown_fields`; exit **`2`** on failure). Exit codes identical pattern to `rde validate`. |
| `kotonoha interchange store [--strict] [PATH]` | Reads envelope JSON (same IO rules as `validate`). Requires **`DATABASE_URL`**. Validates via `kotonoha_core::interchange`, then persists in **one transaction**: **`interchange_documents`** and (when present) derived **`lineage_units`** / **`rde_documents`** rows (`kotonoha_core::store::postgres::PgStore::insert_interchange_document_json`). Primary key UUID of the **`interchange_documents`** row is printed to **stdout**. Missing **`DATABASE_URL`** → exit **1**. Validation failure → exit **2**. Connection / persistence failure → exit **3**. Success → exit **0**. Run **`kotonoha db migrate`** first so tables exist. |
| `kotonoha interchange emit` | Writes a **minimal lineage-only** envelope skeleton (pretty-printed) to stdout. Exit **0**. |

### M1 — workspace, Git, and database context (≥ **0.2.1**, `kotonoha_core` ≥ **0.1.7**)

| Invocation | Behaviour |
| --- | --- |
| `kotonoha init [--project-id ID] [--path DIR]` | Requires **DIR** (default `.`) to be inside a **Git** repository. Writes **`.kotonoha/config.toml`** with `project_id` (default: directory basename). Not a Git repo → exit **1**. I/O failure → exit **3**. Success → exit **0**. |
| `kotonoha status [--path DIR]` | Prints repository root, branch or detached HEAD, `commit`, working tree clean/dirty counts, whether `.kotonoha/config.toml` exists, and if **`DATABASE_URL`** is set whether M1 tables exist and `meaning_deltas` row count. Not a Git repo → exit **1**. Success → exit **0**. |
| `kotonoha diff [--path DIR] [--file PATH]` | Prints unified **unstaged** diff via `kotonoha_core::git` (optional single-file scope). Empty diff prints `(no unstaged diff)`. Exit codes same family as §4 (Git/IO → **1** / **3**). |

### M1 — MeaningDelta and RDE attach (≥ **0.2.2**, `kotonoha_core` ≥ **0.1.7**)

| Invocation | Behaviour |
| --- | --- |
| `kotonoha delta create FILE [--path DIR] [--line-start N] [--line-end N] [--diff-ref REF] [--observation PATH]` | Requires **Git** repo and **`DATABASE_URL`**. Builds [`GitAnchor`](https://github.com/zyx-corporation/kotonoha-core/blob/main/src/semantic_lineage.rs) from current `HEAD` (or `"(no commits yet)"` on empty repo) and **FILE** (repo-relative). When neither line range nor `--diff-ref` is given, defaults `diff_ref` to `unstaged:<rel_path>`. **Observation** JSON: omit for `{}`, or pass a file path (same IO rules as `rde validate`). Persists via `PgStore::create_meaning_delta`; prints new **`meaning_deltas.id`** UUID to stdout. Missing **`DATABASE_URL`** → exit **1**. Anchor / validation failure → exit **2**. DB failure → exit **3**. Success → exit **0**. |
| `kotonoha rde attach --delta-id UUID [--strict] [--materialize-document] [--audit-correlation-id ID] [PATH]` | Reads RDE JSON from **PATH** or stdin. Validates (same rules as `rde validate` when `--strict`). Persists `rde_assessments` row linked to **UUID** via `PgStore::attach_rde_assessment`. Optional **`--materialize-document`** also inserts spec-shaped `rde_documents` and FK. Prints new assessment UUID to stdout. Missing **`DATABASE_URL`** → exit **1**. Validation / lineage errors → exit **2**. DB failure → exit **3**. Success → exit **0**. |

### M1 — review and export (≥ **0.2.3**, `kotonoha_core` ≥ **0.1.8**)

| Invocation | Behaviour |
| --- | --- |
| `kotonoha review approve\|hold\|reject --delta-id UUID [--assessment-id UUID] [--decided-by ID] [--rationale PATH]` | Records [`ReviewDecision`](https://github.com/zyx-corporation/kotonoha-core/blob/main/src/semantic_lineage.rs) via `PgStore::record_review_decision`. **`decided_by`** defaults: `--decided-by` → `KOTONOHA_DECIDED_BY` → `git config user.email` → `$USER`. Help text states RDE does **not** substitute for human judgment. Prints decision UUID. Missing **`DATABASE_URL`** → exit **1**. Validation → exit **2**. DB → exit **3**. |
| `kotonoha export (--delta-id UUID \| --git-commit SHA) [--out FILE]` | Emits JSON **`kotonoha.m1_export.v0.1`** (single delta) or **`kotonoha.m1_export_bundle.v0.1`** (all deltas for commit). Includes `meaning_delta`, `rde_assessments`, `review_decisions`, and `summary_paragraph` (paste-friendly one paragraph). Writes to **FILE** or stdout. Unknown delta → exit **2**. |

### 4.1 Phase 3 — `kotonoha.console_event.v0` (ingest wrapper)

This object is **not** normative in `kotonoha-spec`. It exists so channels (future console, automation) can submit payloads that **merge into the same validation paths** as `interchange validate` / `rde validate` (see internal outline [`20_phase3_core_console_contract_outline_draft.md`](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/20_phase3_core_console_contract_outline_draft.md) in `kotonoha-management`).

**Root JSON** (single top-level key):

| Property | Type | Requirement |
| --- | --- | --- |
| `console_event` | object | **MUST** be present. |

**`console_event` object:**

| Property | Type | Requirement |
| --- | --- | --- |
| `version` | string | **MUST** be exactly **`kotonoha.console_event.v0`**. |
| `kind` | string | **MUST** be one of: **`interchange.ingest.submitted`**, **`rde.review.requested`**. |
| `body` | JSON value | **MUST** be present. Interpretation depends on **`kind`** (below). |

**`body` by `kind`:**

| `kind` | `body` MUST be… | Delegates to |
| --- | --- | --- |
| `interchange.ingest.submitted` | A **`kotonoha.interchange.v1`** envelope object (same constraints as `interchange validate`). | `kotonoha_core::interchange::validate_interchange_json` |
| `rde.review.requested` | RDE interchange root JSON (same shape as `rde validate`: top-level **`rde_review_output`** object). | `kotonoha_core::rde::validate_json` |

Malformed wrapper (missing keys, wrong `version`, unsupported `kind`) → exit **`1`**. Validation failure on `body` → exit **`2`**. Same stderr conventions as `validate` (warnings for non-strict `summary` gaps).

### Concrete signatures — Phase 3 additive (**≥ 0.2.0**)

| Invocation | Behaviour |
| --- | --- |
| `kotonoha interchange ingest [--strict] [--persist] [PATH]` | Reads **`kotonoha.console_event.v0`** JSON from **PATH** or **stdin** (same IO rules as `validate`). Parses **`console_event`**, dispatches **`body`** per §4.1. **`--persist`** (only with **`kind`** **`interchange.ingest.submitted`**): after successful validation, persists the **interchange `body`** exactly like **`interchange store`** (requires **`DATABASE_URL`**; UUID to **stdout** on success). Missing **`DATABASE_URL`** when **`--persist`** → exit **`1`**. DB errors → exit **`3`**. RDE kind ignores **`--persist`**. Exit **`0`** on success without **`--persist`**. |

Implementation notes: **`kotonoha` ≥ 0.1.7** links against **`kotonoha_core`** from [`kotonoha-core`](https://github.com/zyx-corporation/kotonoha-core) (`Cargo.toml` Git dependency on tag **`v0.1.6`**, feature **`postgres`**, for `db` / `interchange store`). **`kotonoha` ≥ 0.2.0** adds **`interchange ingest`**. Local development MAY override via Cargo **`[patch]`** to a path checkout. RDE validation lives in `kotonoha_core::rde`; interchange envelopes in `kotonoha_core::interchange`; migrations and pool helpers in `kotonoha_core::store::postgres`. See [`docs/spec-traceability.md`](https://github.com/zyx-corporation/kotonoha-core/blob/main/docs/spec-traceability.md).

### Exit codes (minimum contract)

| Code | Meaning |
| --- | --- |
| `0` | Success. |
| `1` | User error (invalid arguments or invalid input file). |
| `2` | Validation failure against `kotonoha-spec` interchange rules. |
| `3` | Internal / unexpected error in the CLI implementation. |

Implementations **MAY** extend codes with documentation.

## 5. Relationship to `kotonoha-core`

The CLI **delegates** RDE validation and **interchange envelope** validation to the **`kotonoha_core`** crate ([`kotonoha-core`](https://github.com/zyx-corporation/kotonoha-core)), keeping this repository focused on **argument parsing, IO, and UX**. Future lineage commands **SHOULD** follow the same pattern.

## 6. Traceability matrix (normative intent)

| CLI concern | `kotonoha-spec` reference |
| --- | --- |
| PostgreSQL migrations (`db migrate`) | *(not normative)* — DDL sketch in [`kotonoha-core` migrations](https://github.com/zyx-corporation/kotonoha-core/tree/main/migrations); correlates with [`audit-trail-relationship.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/audit-trail-relationship.md) only at deployment level |
| Interchange persistence (`interchange store`) | *(not normative)* — table **`interchange_documents`** in [`kotonoha-core` migrations](https://github.com/zyx-corporation/kotonoha-core/tree/main/migrations); stores core envelope JSON only |
| RDE review output shape | [`docs/rde-review-output.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/rde-review-output.md) |
| Loss representation obligations | [`docs/representation-of-loss.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/representation-of-loss.md) |
| Semantic lineage unit | [`docs/semantic-lineage-model.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/semantic-lineage-model.md) |
| Conformance keywords | [`docs/introduction.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/introduction.md) |
| Interchange envelope (`interchange` subcommands) | *(not normative in spec)* — implementation in [`kotonoha-core` `interchange`](https://github.com/zyx-corporation/kotonoha-core/blob/main/src/interchange.rs); aligns `spec_bundle` / lineage / nested RDE with spec sections above |
| **`interchange ingest`** (`kotonoha.console_event.v0`) | *(not normative in spec)* — transport wrapper only; **`body`** validation/traceability identical to rows above for RDE / interchange. Internal event names align with [`20` §2](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/20_phase3_core_console_contract_outline_draft.md) working list. |
| MeaningDelta (`delta create`) | [`docs/semantic-lineage-model.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/semantic-lineage-model.md); M1 DDL in [`kotonoha-core` `postgresql-schema-m1.md`](https://github.com/zyx-corporation/kotonoha-core/blob/main/docs/postgresql-schema-m1.md) |
| RDE attach (`rde attach`) | [`docs/rde-review-output.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/rde-review-output.md); table **`rde_assessments`** in M1 schema |
| Review (`review approve\|hold\|reject`) | [`docs/semantic-lineage-model.md`](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/semantic-lineage-model.md); table **`review_decisions`** |
| Export (`export`) | M1 audit bundle (informative JSON); aggregates rows above |

This table **MUST** be updated when new commands tie to additional specification sections.

## 7. Versioning

CLI releases **SHOULD** follow [Semantic Versioning](https://semver.org/) for **behaviour visible to scripts** (flags, exit codes, default paths). Documentation-only clarifications may be patch releases.

## 8. Human accountability

The CLI **MUST NOT** be documented as replacing human judgment for publication, compliance, or design approval. Commands assist validation and automation; they do not **authorize** outcomes.

---

## Changelog (document level)

| Date | Change |
| --- | --- |
| 2026-05-10 | Initial public definition for Phase 2 gate. |
| 2026-05-10 | Concrete signatures for Rust CLI **0.1.0** (`version`, `rde validate`, `rde emit`). |
| 2026-05-10 | **`interchange validate` Unknown-key contract** (**`kotonoha_core`** **≥ 0.1.6**); CLI **≥ 0.1.7** depends on **`kotonoha_core` `v0.1.6`**. |
| 2026-05-10 | **`interchange`** subcommands (**0.1.2**); core tag **`v0.1.1`**. |
| 2026-05-10 | Cross-link **`cli-requirements.md`** (requirements backlog vs this contract document). |
| 2026-05-12 | Bare **`kotonoha`** invocation: full help on stdout, exit **0** (**0.1.8**; aligns §4 table with Clap optional subcommand behaviour). |
| 2026-05-12 | **§4.1** `kotonoha.console_event.v0` + **`interchange ingest`** (**≥ 0.2.0**); Phase 3 ingest path; §6 matrix row. |
| 2026-05-20 | **M1** `delta create`, `rde attach` (**≥ 0.2.2**); §6 matrix rows for MeaningDelta / RDE assessment. |
| 2026-05-20 | **M1** `review`, `export` (**≥ 0.2.3**); closes CLI track for management#97 M1-f. |
