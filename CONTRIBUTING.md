# Contributing to kotonoha-cli

Contributions should preserve alignment with [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec). Update [docs/cli-definition.md](docs/cli-definition.md) when you change stable command names, exit codes, or traceability.

## Phase 2 acceptance-style demo

For a reproducible command sequence (version, RDE / interchange validation, optional Postgres path), see the public tutorial **[Phase 2 CLI walkthrough](https://github.com/zyx-corporation/kotonoha-docs/blob/main/docs/tutorials/phase2_cli_walkthrough.md)** in [`kotonoha-docs`](https://github.com/zyx-corporation/kotonoha-docs). Exact contracts remain defined in `docs/cli-definition.md`.

**Automated script (same checks as the tutorial, plus invalid-JSON exit‑2 check):** [`scripts/phase2_acceptance_demo.sh`](scripts/phase2_acceptance_demo.sh) — run after `cargo build --release`; set `DATABASE_URL` to exercise `db migrate` and `interchange store`. CI runs this script on `main` / pull requests with PostgreSQL.

## Workflow

Organization **Git/Issue/branch/PR** rules (**no direct edits to `main`**): **[`docs/git_operation_rules.md`](docs/git_operation_rules.md)** ([canonical in **`kotonoha-management`**](https://github.com/zyx-corporation/kotonoha-management/blob/main/docs/04_git_operation_rules.md); update canon first).

1. **Issue** first for command-shape or breaking CLI changes.
2. **Pull request** with tests where applicable (`cargo test`).
3. Link specification sections affected (see traceability matrix in `docs/cli-definition.md`).
4. Run `cargo fmt --all` before submitting; CI runs `cargo fmt --check`, **`cargo test`**, **`cargo build --release`**, and **`scripts/phase2_acceptance_demo.sh`** on the release binary **with PostgreSQL `DATABASE_URL`** (see workflow file).

CI uses `DATABASE_URL` pointing at a test database. Locally, start PostgreSQL (for example match credentials in [`kotonoha-core` docker-compose](https://github.com/zyx-corporation/kotonoha-core/blob/main/docker-compose.yml)), export `DATABASE_URL`, then run `kotonoha db migrate` before exercising store commands.

## Language

English-first in documentation; Japanese may be added as `*_ja.md`.

## License

[Apache License 2.0](LICENSE).
