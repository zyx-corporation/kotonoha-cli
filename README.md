# kotonoha-cli

**Official command-line interface for the Kotonoha ecosystem** — the `kotonoha` executable for working with Semantic Lineage System (SLS) interchange, validation, and related developer workflows.

Normative technical contracts for interchange and lineage remain in [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec). This repository hosts the **CLI definition**, implementation, and user-facing developer notes for the binary interface.

**Japanese:** [README_ja.md](README_ja.md)

## Specification index (CLI)

| Document | Description |
| --- | --- |
| [docs/cli-definition.md](docs/cli-definition.md) | **Public definition** of the `kotonoha` CLI (command surface, boundaries, traceability to `kotonoha-spec`). |

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
```

Pipe JSON on stdin (omit path or use `-`):

```bash
./target/release/kotonoha rde emit | ./target/release/kotonoha rde validate
```

Use `--strict` to treat missing `summary` on category items as errors (see `kotonoha-spec`).

## Links

- Repository: https://github.com/zyx-corporation/kotonoha-cli
- CLI definition: [docs/cli-definition.md](docs/cli-definition.md)
- GitHub Projects (organization workflow): [`docs/github_projects_policy.md`](docs/github_projects_policy.md)
