# Contributing to kotonoha-cli

Contributions should preserve alignment with [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec). Update [docs/cli-definition.md](docs/cli-definition.md) when you change stable command names, exit codes, or traceability.

## Phase 2 acceptance-style demo

For a reproducible command sequence (version, RDE / interchange validation, optional Postgres path), see the public tutorial **[Phase 2 CLI walkthrough](https://github.com/zyx-corporation/kotonoha-docs/blob/main/docs/tutorials/phase2_cli_walkthrough.md)** in [`kotonoha-docs`](https://github.com/zyx-corporation/kotonoha-docs). Exact contracts remain defined in `docs/cli-definition.md`.

## Workflow

1. **Issue** first for command-shape or breaking CLI changes.
2. **Pull request** with tests where applicable (`cargo test`).
3. Link specification sections affected (see traceability matrix in `docs/cli-definition.md`).
4. Run `cargo fmt --all` before submitting; CI runs `cargo fmt --check`, `cargo test`, `cargo build --release`, and (with PostgreSQL) `kotonoha db migrate` plus piping `interchange emit` into `interchange store`.

CI uses `DATABASE_URL` pointing at a test database. Locally, start PostgreSQL (for example match credentials in [`kotonoha-core` docker-compose](https://github.com/zyx-corporation/kotonoha-core/blob/main/docker-compose.yml)), export `DATABASE_URL`, then run `kotonoha db migrate` before exercising store commands.

## Language

English-first in documentation; Japanese may be added as `*_ja.md`.

## License

[Apache License 2.0](LICENSE).
