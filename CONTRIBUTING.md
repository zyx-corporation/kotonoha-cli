# Contributing to kotonoha-cli

Contributions should preserve alignment with [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec). Update [docs/cli-definition.md](docs/cli-definition.md) when you change stable command names, exit codes, or traceability.

## Workflow

1. **Issue** first for command-shape or breaking CLI changes.
2. **Pull request** with tests where applicable (`cargo test`).
3. Link specification sections affected (see traceability matrix in `docs/cli-definition.md`).
4. Run `cargo fmt --all` before submitting; CI runs `cargo fmt --check`, `cargo test`, and `cargo build --release`.

## Language

English-first in documentation; Japanese may be added as `*_ja.md`.

## License

[Apache License 2.0](LICENSE).
