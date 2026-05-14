# kotonoha-cli

**Official command-line interface for the Kotonoha ecosystem** — the `kotonoha` executable for working with Semantic Lineage System (SLS) interchange, validation, and related developer workflows.

Normative technical contracts for interchange and lineage remain in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec). This repository hosts the **CLI definition**, implementation, and user-facing developer notes for the binary interface.

**Japanese:** [README_ja.md](README_ja.md)

## Specification index (CLI)

| Document | Description |
| --- | --- |
| [docs/cli-definition.md](docs/cli-definition.md) | **Public definition** of the `kotonoha` CLI (command surface, boundaries, traceability to `kotonoha-spec`). |

**Phase 2 MVP (this repository):** behaviour matches §2–§4 of `cli-definition.md` (RDE and `kotonoha.interchange.v1`, optional Postgres via `kotonoha_core`). Phase 3 backlog items live in [docs/cli-requirements.md](docs/cli-requirements.md), not in the baseline contract.

## Relationship to other repositories

| Repository | Role |
| --- | --- |
| [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) | Canonical specifications (including RDE interchange). |
| [`kotonoha-core`](https://github.com/zyx-corporation/kotonoha-core) | OSS core libraries the CLI **SHOULD** depend on when implementing behaviour. |
| [`kotonoha-docs`](https://github.com/zyx-corporation/kotonoha-docs) | Non-normative manuals and tutorials. |
| **kotonoha-cli (this repository)** | CLI definition and implementation. |

## Language policy

**English-first** for documentation in this repository. Japanese translations use the `*_ja.md` suffix alongside English sources.

## License

Unless otherwise stated in a specific file, repository content is licensed under the [Apache License 2.0](LICENSE).

## Quickstart (build from source)

Requires [Rust](https://www.rust-lang.org/tools/install) (stable, MSRV in `Cargo.toml`).

```bash
cargo build --release
./target/release/kotonoha version
./target/release/kotonoha rde emit
./target/release/kotonoha rde validate path/to/file.json
./target/release/kotonoha interchange emit
./target/release/kotonoha interchange validate path/to/envelope.json
```

Pipe JSON on stdin (omit path or use `-`):

```bash
./target/release/kotonoha rde emit | ./target/release/kotonoha rde validate
./target/release/kotonoha interchange emit | ./target/release/kotonoha interchange validate
./target/release/kotonoha interchange emit | ./target/release/kotonoha interchange store
```

Phase 3 **console event** ingest (wrapper schema: **`docs/cli-definition.md`** §4.1; requires **Python 3** only for this one-liner example):

```bash
./target/release/kotonoha interchange emit | python3 -c 'import json,sys; b=json.load(sys.stdin); print(json.dumps({"console_event":{"version":"kotonoha.console_event.v0","kind":"interchange.ingest.submitted","body":b}}))' | ./target/release/kotonoha interchange ingest --strict
```

Use `--strict` to treat missing `summary` on category items as errors (see `kotonoha-spec`).

PostgreSQL migrations (requires `DATABASE_URL`, same URL shape as [`kotonoha-core` `docker-compose.yml`](https://github.com/zyx-corporation/kotonoha-core/blob/main/docker-compose.yml)):

```bash
export DATABASE_URL="postgres://kotonoha:kotonoha@localhost:5432/kotonoha_dev"
./target/release/kotonoha db migrate
```

Persisting an interchange envelope requires the migration that creates **`interchange_documents`** (included in **`db migrate`**). Example:

```bash
export DATABASE_URL="postgres://kotonoha:kotonoha@localhost:5432/kotonoha_dev"
./target/release/kotonoha db migrate
./target/release/kotonoha interchange emit | ./target/release/kotonoha interchange store
```

The new interchange row’s UUID is printed to **stdout**. When the envelope includes **`lineage_unit`** and/or **`rde_document`**, matching rows are written to **`lineage_units`** and **`rde_documents`** in the **same database transaction** (`kotonoha_core` **0.1.6+**).

## `kotonoha-core` dependency

The CLI depends on [`kotonoha_core`](https://github.com/zyx-corporation/kotonoha-core) via **`Cargo.toml` Git** at tag **`v0.1.6`** with feature **`postgres`**. That tag **must exist on GitHub** before `cargo build` / `cargo test` can fetch the dependency (`failed to find tag …` otherwise). Publish flow for maintainers: merge `kotonoha-core` changes for the targeted release, then push the semver tag referenced in `Cargo.toml` (for example `git tag v0.1.6 && git push origin v0.1.6`).

To build **`kotonoha-cli`** against a **local** `kotonoha-core` checkout (e.g. tag not pushed yet), add a Cargo **[patch]** (for example in `~/.cargo/config.toml`; adjust the path):

```toml
[patch."https://github.com/zyx-corporation/kotonoha-core.git"]
kotonoha_core = { path = "/path/to/kotonoha-core" }
```

Requested dependency features (for example **`postgres`**) still come from **`Cargo.toml`** on this crate.

## Links

- Repository: https://github.com/zyx-corporation/kotonoha-cli
- CLI definition: [docs/cli-definition.md](docs/cli-definition.md)
- GitHub Projects (organization workflow): [`docs/github_projects_policy.md`](docs/github_projects_policy.md)
