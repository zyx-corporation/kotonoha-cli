# kotonoha-cli（日本語概要）

**Kotonoha エコシステム向けの公式 CLI** を置くリポジトリです。実行ファイル名は **`kotonoha`** とします。

規範的な契約（RDE 出力・lineage 等）は [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) が正本です。本リポジトリは **CLI の公開定義**・実装・開発者向けメモを扱います。

**English:** [README.md](README.md)

## CLI の公開定義

| 文書 | 内容 |
| --- | --- |
| [docs/cli-definition.md](docs/cli-definition.md) | `kotonoha` のコマンド境界・[`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) へのトレース |

**Phase 2 MVP（本リポジトリの射程）:** `cli-definition.md` §2〜§4 に書かれた挙動（RDE・`kotonoha.interchange.v1`・任意の Postgres）。Phase 3 の追加要件は [docs/cli-requirements.md](docs/cli-requirements.md) にあり、ベースライン契約とは別です。

## インストール（推奨）

ターミナルで次を実行します（詳細は [kotonoha-docs: CLI のインストール](https://github.com/zyx-corporation/kotonoha-docs/blob/main/ja/tutorials/install_kotonoha_cli.md)）。

```bash
curl -fsSL https://raw.githubusercontent.com/zyx-corporation/kotonoha-cli/main/scripts/install.sh | bash
export PATH="$HOME/.local/bin:$PATH"
kotonoha version
```

バージョン指定・インストール先の変更・メンテナ向け手順は [`scripts/install.sh`](scripts/install.sh)、[RELEASING.md](RELEASING.md)、[実装手順書](https://github.com/zyx-corporation/kotonoha-docs/blob/main/ja/manual/cli_installer_implementation.md) を参照してください。バイナリは Git に含めず、タグ push 時に CI が GitHub Releases へ公開します。

## ビルド（ソースから）

[Rust](https://www.rust-lang.org/tools/install) が必要です。

```bash
cargo build --release
./target/release/kotonoha version
./target/release/kotonoha rde emit | ./target/release/kotonoha rde validate
./target/release/kotonoha interchange emit | ./target/release/kotonoha interchange validate
./target/release/kotonoha interchange emit | ./target/release/kotonoha interchange store
```

Phase 3 の **console event** 取り込み（ラッパー仕様は **`docs/cli-definition.md`** §4.1。例では **Python 3** を使用）:

```bash
./target/release/kotonoha interchange emit | python3 -c 'import json,sys; b=json.load(sys.stdin); print(json.dumps({"console_event":{"version":"kotonoha.console_event.v0","kind":"interchange.ingest.submitted","body":b}}))' | ./target/release/kotonoha interchange ingest --strict
```

PostgreSQL のマイグレーション（`DATABASE_URL` が必要。[`kotonoha-core` の `docker-compose.yml`](https://github.com/zyx-corporation/kotonoha-core/blob/main/docker-compose.yml) と同じ接続形の例）:

```bash
export DATABASE_URL="postgres://kotonoha:kotonoha@localhost:5432/kotonoha_dev"
./target/release/kotonoha db migrate
```

エンベロープを PostgreSQL に保存する例（先に **`kotonoha db migrate`** で **`interchange_documents`** を作成）:

```bash
export DATABASE_URL="postgres://kotonoha:kotonoha@localhost:5432/kotonoha_dev"
./target/release/kotonoha db migrate
./target/release/kotonoha interchange emit | ./target/release/kotonoha interchange store
```

エンベロープに **`lineage_unit`** や **`rde_document`** が含まれる場合、同一トランザクションで **`lineage_units`** / **`rde_documents`** にも反映されます（`kotonoha_core` **0.1.6** 以降）。

## `kotonoha-core` との依存関係

CLI は [`kotonoha_core`](https://github.com/zyx-corporation/kotonoha-core) を **`Cargo.toml` の Git 依存**で、タグ **`v0.1.6`**・機能 **`postgres`** として取り込みます。GitHub 上に **`Cargo.toml` が参照するタグが無いと** `cargo build` は依存取得で失敗します。運用では `kotonoha-core` をマージしたうえで、`Cargo.toml` が参照するタグ（例: **`v0.1.6`**）をプッシュしてください。

ローカルの `kotonoha-core` を参照してビルドする場合は、Cargo の **[patch]**（例: `~/.cargo/config.toml`、パスは環境に合わせる）で上書きできます。

```toml
[patch."https://github.com/zyx-corporation/kotonoha-core.git"]
kotonoha_core = { path = "/path/to/kotonoha-core" }
```

（**`postgres`** などの機能指定は、この CLI の `Cargo.toml` 側の依存定義がそのまま効きます。）

## 関連リポジトリ

| リポジトリ | 役割 |
| --- | --- |
| [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) | 公開仕様の正本 |
| [`kotonoha-core`](https://github.com/zyx-corporation/kotonoha-core) | OSS コア実装（CLI は実装時に依存を想定） |
| [`kotonoha-docs`](https://github.com/zyx-corporation/kotonoha-docs) | 仕様外のマニュアル・チュートリアル |
| **kotonoha-cli（本リポジトリ）** | CLI 定義・実装 |

## 言語方針

文書は原則 **英語**。日本語は `*_ja.md` で併置します。

## ライセンス

特記なき限り [Apache License 2.0](LICENSE) とします。
