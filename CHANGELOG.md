# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

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
