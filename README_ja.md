# kotonoha-cli（日本語概要）

**Kotonoha エコシステム向けの公式 CLI** を置くリポジトリです。実行ファイル名は **`kotonoha`** とします。

規範的な契約（RDE 出力・lineage 等）は [`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) が正本です。本リポジトリは **CLI の公開定義**・実装・開発者向けメモを扱います。

**English:** [README.md](README.md)

## CLI の公開定義

| 文書 | 内容 |
| --- | --- |
| [docs/cli-definition.md](docs/cli-definition.md) | `kotonoha` のコマンド境界・[`kotonoha-spec`](https://github.com/zyx-corporation/kotonoha-spec) へのトレース |

## ビルド（ソースから）

[Rust](https://www.rust-lang.org/tools/install) が必要です。

```bash
cargo build --release
./target/release/kotonoha version
./target/release/kotonoha rde emit | ./target/release/kotonoha rde validate
```

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
