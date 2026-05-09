# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.0] — 2026-05-10

### Added

- Repository scaffold and [docs/cli-definition.md](docs/cli-definition.md) (public CLI definition for Phase 2).
- Rust implementation: `kotonoha version`, `kotonoha rde validate [--strict] [PATH]`, `kotonoha rde emit`.
- Transitional in-repo validation (`src/rde_validate.rs`) aligned with [`kotonoha-spec` RDE output](https://github.com/zyx-corporation/kotonoha-spec/blob/main/docs/rde-review-output.md); delegation to `kotonoha-core` documented as future work.
- CI workflow (fmt, test, release build).
