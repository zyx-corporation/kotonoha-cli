# `kotonoha` CLI — public definition

This document is the **authoritative definition** of the command-line interface shipped from this repository. Implementations **MUST** conform to it unless an explicit exception is documented in [CHANGELOG.md](../CHANGELOG.md).

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
| `kotonoha interchange` | Validate / emit / **store** **core interchange envelope** JSON (`kotonoha.interchange.v1`) — bundles optional lineage +/or RDE for pipelines (**not** normative in `kotonoha-spec`). |

### Concrete signatures (release **0.1.x**)

| Invocation | Behaviour |
| --- | --- |
| `kotonoha` | Prints help (same as `--help`). Global `--version` prints short version via Clap. |
| `kotonoha version` | Prints **two lines** to stdout: (1) `kotonoha <crate semver>` (2) `kotonoha-spec (target bundle): 0.1`. Exit **0**. |
| `kotonoha db migrate` | Reads **`DATABASE_URL`**, connects with SQLx, applies migrations from `kotonoha-core/migrations`. Missing env → exit **1**. Connection / migration failure → exit **3**. Success prints one confirmation line to stdout → exit **0**. |
| `kotonoha rde validate [--strict] [PATH]` | Reads JSON from **PATH**, or from **stdin** when PATH is omitted or `-`. Validates Phase 1 interchange (`spec_version` **MUST** be `0.1`). Items missing `summary` emit **warnings** on stderr unless `--strict`, then exit **2**. Malformed args / unreadable file / invalid UTF-8 → exit **1**. Validation failure → exit **2**. Success → exit **0**. |
| `kotonoha rde emit` | Writes a **minimal compliant** JSON skeleton (pretty-printed) to stdout. Exit **0**. |
| `kotonoha interchange validate [--strict] [PATH]` | Validates **`kotonoha.interchange.v1`** envelope (`format`, `spec_bundle`, optional `lineage_unit`, optional `rde_document`). Nested RDE uses the same `--strict` semantics as `rde validate`. Exit codes identical pattern to `rde validate`. |
| `kotonoha interchange store [--strict] [PATH]` | Reads envelope JSON (same IO rules as `validate`). Requires **`DATABASE_URL`**. Validates via `kotonoha_core::interchange`, inserts into PostgreSQL table **`interchange_documents`** (`INSERT` UUID primary key returned on stdout). Missing **`DATABASE_URL`** → exit **1**. Validation failure → exit **2**. Connection / persistence failure → exit **3**. Success → exit **0**. Run **`kotonoha db migrate`** first so `interchange_documents` exists. |
| `kotonoha interchange emit` | Writes a **minimal lineage-only** envelope skeleton (pretty-printed) to stdout. Exit **0**. |

Implementation notes: **`kotonoha` ≥ 0.1.4** links against **`kotonoha_core`** from [`kotonoha-core`](https://github.com/zyx-corporation/kotonoha-core) (`Cargo.toml` Git dependency on tag **`v0.1.3`**, feature **`postgres`**, for `db` / `interchange store`). Local development MAY override via Cargo **`[patch]`** to a path checkout. RDE validation lives in `kotonoha_core::rde`; interchange envelopes in `kotonoha_core::interchange`; migrations and pool helpers in `kotonoha_core::store::postgres`. See [`docs/spec-traceability.md`](https://github.com/zyx-corporation/kotonoha-core/blob/main/docs/spec-traceability.md).

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
| 2026-05-10 | **`interchange`** subcommands (**0.1.2**); core tag **`v0.1.1`**. |
