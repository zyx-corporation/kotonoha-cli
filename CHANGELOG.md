# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.2.4] — 2026-05-22

### Added

- **M2** ([#20](https://github.com/zyx-corporation/kotonoha-cli/issues/20)): `kotonoha rde attach --source-kind` (cli|llm|import|replay) via `validate_and_attach_rde`; `kotonoha export --format m2` (`kotonoha.m2_export.v0.1` with RDE meta + `observation_rde_hints`).
- [`scripts/m2_acceptance_demo.sh`](scripts/m2_acceptance_demo.sh) — M2 end-to-end demo.

### Changed

- `kotonoha rde attach` uses `PgStore::validate_and_attach_rde` (strict warnings reject attach).
- Depends on `kotonoha_core` **≥ 0.1.9**.

## [0.2.3] — 2026-05-20

### Added

- **M1 commands** ([#15](https://github.com/zyx-corporation/kotonoha-cli/issues/15)): `kotonoha review approve|hold|reject`, `kotonoha export` — `record_review_decision` and JSON export (`kotonoha.m1_export.v0.1`); depends on `kotonoha_core` **≥ 0.1.8**.
- [`scripts/m1_acceptance_demo.sh`](scripts/m1_acceptance_demo.sh) — M1 end-to-end demo when `DATABASE_URL` is set.

## [0.2.2] — 2026-05-20

### Added

- **M1 commands** ([#14](https://github.com/zyx-corporation/kotonoha-cli/issues/14)): `kotonoha delta create` — Git-anchored MeaningDelta via `PgStore::create_meaning_delta`; `kotonoha rde attach` — RDE JSON to `rde_assessments` via `attach_rde_assessment` (`--delta-id`, `--strict`, `--materialize-document`).
- Shared `pg_store()` / `store_error_code()` helpers for DB-backed commands.

### Changed

- **[`docs/cli-definition.md`](docs/cli-definition.md)** — M1 § for `delta create` / `rde attach`; §6 traceability rows.

### Added (tests)

- Smoke: `delta create` without `DATABASE_URL` → exit **1** (in Git repo).

## [0.2.1] — 2026-05-20

### Added

- **M1 commands** ([#13](https://github.com/zyx-corporation/kotonoha-cli/issues/13)): `kotonoha init`, `kotonoha status`, `kotonoha diff` — Git context via `kotonoha_core` ≥ **0.1.7**, local `.kotonoha/config.toml`.

## [0.2.0] — 2026-05-12

### Added

- **`kotonoha interchange ingest`** — accepts **`kotonoha.console_event.v0`** JSON (`interchange.ingest.submitted` / `rde.review.requested`) and delegates to the same **`kotonoha_core`** validation as **`interchange validate`** / **`rde validate`**. Optional **`--persist`** persists **`interchange.ingest.submitted`** bodies like **`interchange store`** (Phase 3 / [`cli-definition.md`](docs/cli-definition.md) §4.1).

### Changed

- Semver **minor** for new stable subcommand and public wrapper schema documentation.

### Added (tests)

- Smoke tests for **`interchange ingest`** (round-trip from **`interchange emit`**, unknown **`kind`** → exit **1**).

## [0.1.8] — 2026-05-12

### Fixed

- Invoking **`kotonoha`** with no subcommand prints full help and exits **0**, matching **`docs/cli-definition.md`** §4 (same intent as **`--help`**). Previously Clap treated a missing subcommand as an error (**exit `2`**).

### Changed

- **[README.md](README.md)** / **[README_ja.md](README_ja.md)** — document **`kotonoha_core`** Git tag **`v0.1.6`** (prose had drifted from **`Cargo.toml`**).
- **Phase 2 MVP scope** called out in READMEs (baseline vs **`docs/cli-requirements.md`** backlog).

### Added

- Smoke test: bare **`kotonoha`** invocation succeeds and prints **`Usage:`**.

## [0.1.7] — 2026-05-10

### Added

- Smoke test asserting **`interchange validate --strict` exit `2`** on interchange JSON with unknown **top-level** keys (**`kotonoha_core`** `deny_unknown_fields`).

### Changed

- Depend on **`kotonoha_core`** tag **`v0.1.6`** (strict interchange + `lineage_unit` deserialization; expanded library Negative tests).

## [0.1.6] — 2026-05-10

### Added

- **`tests/cli_smoke.rs`**: `assert_cmd` smoke tests for `version`, `rde emit|validate`, `interchange emit|validate`.
- **CI**: PostgreSQL 16 service, `DATABASE_URL`, and shell smoke (`db migrate`, `interchange emit | interchange store`).

### Changed

- Depend on **`kotonoha_core`** **`v0.1.5`** (migration GIN fix for `interchange_documents`, CI / integration tests on core).

## [0.1.5] — 2026-05-10

### Changed

- Depend on **`kotonoha_core`** **`v0.1.4`** — `interchange store` now persists **`interchange_documents`** plus derived **`lineage_units`** / **`rde_documents`** in **one transaction** when the envelope includes those payloads.

## [0.1.4] — 2026-05-10

### Added

- `kotonoha interchange store [--strict] [PATH]` — validates `kotonoha.interchange.v1` JSON and inserts into PostgreSQL **`interchange_documents`** via [`kotonoha_core::store::postgres::PgStore::insert_interchange_document_json`](https://github.com/zyx-corporation/kotonoha-core) (requires **`DATABASE_URL`**; run **`kotonoha db migrate`** first).

### Changed

- Depend on **`kotonoha_core`** at Git tag **`v0.1.3`** (interchange table migration + store API).

## [0.1.3] — 2026-05-10

### Added

- `kotonoha db migrate` — applies SQL migrations from [`kotonoha-core` `migrations/`](https://github.com/zyx-corporation/kotonoha-core/tree/main/migrations) via [`kotonoha_core::store::postgres::PgStore`](https://github.com/zyx-corporation/kotonoha-core) (requires `DATABASE_URL`).

### Changed

- Depend on **`kotonoha_core`** at Git tag **`v0.1.2`** with feature **`postgres`** (`Cargo.toml` Git dependency on [`kotonoha-core`](https://github.com/zyx-corporation/kotonoha-core)).

### Notes

- **`cargo build` / CI** resolve **`kotonoha_core`** from Git tag **`v0.1.2`** on [`kotonoha-core`](https://github.com/zyx-corporation/kotonoha-core). For an unpublished local core checkout, use a Cargo **`[patch]`** (see [README.md](README.md)).

## [0.1.2] — 2026-05-10

### Added

- `kotonoha interchange validate` / `kotonoha interchange emit` using [`kotonoha_core::interchange`](https://github.com/zyx-corporation/kotonoha-core) (`kotonoha.interchange.v1` envelope).

### Changed

- Depend on **`kotonoha_core` `v0.1.1`** (interchange module).

## [0.1.1] — 2026-05-10

### Changed

- RDE validation now delegates to **`kotonoha_core`** ([`kotonoha-core`](https://github.com/zyx-corporation/kotonoha-core) `v0.1.0`, Git dependency). Removed `src/rde_validate.rs`.

## [0.1.0] — 2026-05-10

### Added

- Repository scaffold and [docs/cli-definition.md](docs/cli-definition.md) (public CLI definition for Phase 2).
- Rust implementation: `kotonoha version`, `kotonoha rde validate [--strict] [PATH]`, `kotonoha rde emit`.
- Transitional in-repo validation (`src/rde_validate.rs`) aligned with [`kotonoha-spec` RDE output](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/rde-review-output.md); replaced by `kotonoha_core` in **0.1.1**.
- CI workflow (fmt, test, release build).
