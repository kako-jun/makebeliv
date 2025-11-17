# makebeliv

**リアルタイムボイスチェンジャー with 揺らぎエンジン**

コナンの変声リボンのような、自然で遅延の少ないボイスチェンジャーを目指すプロジェクト。

## 🎯 プロジェクトの目標

- **VTuberレベルの自然な声変換** - チェンジャーとバレない品質
- **超低遅延（200-300ms）** - リアルタイム実況・会話に対応
- **完全ローカル動作** - GPU活用、従量課金なし
- **原音漏洩ゼロ** - 元の声が一切聞こえない設計
- **揺らぎエンジン** - 人間らしい自然な変化を実現（オリジナル技術）

## 🏗️ アーキテクチャ

```
┌─────────────────────────────────────────────────┐
│                  Rust CLI                       │
│  (音声入出力・制御・セットアップ自動化)              │
└──────────────┬──────────────────────────────────┘
               │ HTTP/IPC
┌──────────────▼──────────────────────────────────┐
│              Python Engine                      │
│  - RVC変換 (PyTorch + GPU)                      │
│  - 揺らぎエンジン (ピッチ/音量/EQ)                 │
│  - ノイズ合成                                    │
└─────────────────────────────────────────────────┘
```

## 🚀 クイックスタート

### オプション1: ローカル実行（推奨）

```bash
# 1. セットアップスクリプトを実行（uvを使用）
./scripts/setup.sh

# 2. Rustバイナリをビルド
cargo build --release

# 3. APIサーバーを起動
./target/release/makebeliv server

# 4. 別のターミナルで音声処理を実行
./target/release/makebeliv process -i audio/input/test.wav --use-api
```

### オプション2: Docker実行

```bash
# GPU版（NVIDIA GPU環境）
./scripts/docker-run.sh build
./scripts/docker-run.sh gpu

# CPU版
./scripts/docker-run.sh cpu

# 音声処理（ローカルから）
makebeliv process -i audio/input/test.wav --use-api --api-url http://localhost:8000
```

詳細は [DOCKER.md](./DOCKER.md) を参照してください。

## 📦 技術スタック

### Python側
- **RVC (Retrieval-based Voice Conversion)** - 高品質な声変換
- **PyTorch** - GPU推論（CUDA 11.8対応）
- **librosa / scipy** - 音声処理・ピッチ抽出
- **FastAPI + Uvicorn** - 非同期HTTPサーバー
- **uv** - 高速パッケージマネージャー

### Rust側
- **clap** - CLIインターフェース
- **cpal** - 低レイテンシ音声I/O
- **tokio + reqwest** - 非同期HTTP通信
- **hound** - WAVファイル処理

## 🎨 オリジナリティ

### 揺らぎエンジン
既存のVCツールにはない、人間らしい自然な変化を実現：

- **時間的連続性** - 急激な変化を避け、滑らかに遷移
- **ピッチの微細な揺らぎ** - ±5%の範囲でランダムに変動
- **音量の自然な変化** - 発話ごとに微妙に変化
- **EQの揺らぎ** - 周波数特性に変化を加える

### 遅延最小化
- **非同期パイプライン** - 録音・変換・再生を並列化
- **先読み処理** - 次のチャンクを事前に変換
- **最適なチャンクサイズ** - 100-200msで品質と遅延のバランス

## 📂 プロジェクト構造

```
makebeliv/
├── src/                     # Rustソースコード
│   ├── main.rs             # CLIエントリーポイント
│   ├── audio.rs            # 音声I/O（cpal）
│   ├── client.rs           # HTTPクライアント
│   └── lib.rs              # ライブラリルート
├── python/                 # Pythonモジュール
│   ├── fluctuation.py      # 揺らぎエンジン
│   ├── file_processor.py   # ファイルベース処理
│   ├── rvc_engine.py       # RVC変換エンジン
│   └── api_server.py       # FastAPIサーバー
├── scripts/                # ユーティリティスクリプト
│   ├── setup.sh           # セットアップ（uv使用）
│   ├── test.sh            # テスト実行
│   └── docker-run.sh      # Docker管理
├── audio/                  # 音声ファイル
├── models/                 # RVCモデル
├── Dockerfile              # GPU版イメージ
├── Dockerfile.cpu          # CPU版イメージ
├── docker-compose.yml      # Docker構成
├── Cargo.toml              # Rust依存関係
├── pyproject.toml          # Python環境設定
└── requirements.txt        # Python依存関係
```

## 🧪 開発ステータス

- [x] プロジェクト構造設計
- [x] Python環境自動構築（uv統合）
- [x] 揺らぎエンジン基本実装
- [x] ファイルベース処理フロー
- [x] RVC変換エンジン実装
- [x] FastAPI HTTPサーバー実装
- [x] Rust HTTPクライアント実装
- [x] Rust音声I/O実装（cpal）
- [x] Docker対応（GPU/CPU両対応）
- [ ] リアルタイムモード完全実装
- [ ] 仮想マイク出力統合
- [ ] 遅延測定・最適化
- [ ] 実際のRVCモデル統合

## 🔧 要件

### ハードウェア
- **GPU推奨**: NVIDIA RTX 3050以上（CUDA対応）
- **メモリ**: 8GB以上

### ソフトウェア
- Rust (2021 edition以降)
- Python 3.10以上
- uv (Pythonパッケージマネージャー)

## 📖 コマンドリファレンス

```bash
# セットアップ（uv使用）
makebeliv setup [--yes]

# APIサーバー起動
makebeliv server [--host 0.0.0.0] [--port 8000]

# ファイル処理（直接実行）
makebeliv process -i <input> [-o <output>] [--model <model>] [--noise <type>] [--pitch <shift>]

# ファイル処理（API経由）
makebeliv process -i <input> --use-api [--api-url http://localhost:8000]

# リアルタイム変換
makebeliv monitor --model <model> --noise <type> --pitch <shift> [--api-url http://localhost:8000]

# オーディオデバイス一覧
makebeliv list-devices
```

### uvを直接使用

```bash
# Python環境セットアップ
uv venv .venv
source .venv/bin/activate  # Linux/macOS
.venv\Scripts\activate     # Windows

# 依存関係インストール
uv pip install -r requirements.txt

# APIサーバー起動
uv run uvicorn python.api_server:app --reload

# Pythonスクリプト実行
uv run python python/file_processor.py audio/input/test.wav
```

## 📚 プロジェクトドキュメント

詳細なプロジェクト管理ドキュメントは `.claude/` ディレクトリにあります：

- **[CLAUDE.md](./.claude/CLAUDE.md)** - プロジェクト概要・構想・マイルストーン
- **[ARCHITECTURE.md](./.claude/ARCHITECTURE.md)** - システムアーキテクチャ詳細
- **[DESIGN.md](./.claude/DESIGN.md)** - 設計詳細・実装ガイド
- **[DECISIONS.md](./.claude/DECISIONS.md)** - 設計決定記録 (ADR)
- **[TODO.md](./.claude/TODO.md)** - タスク管理・進捗状況

## 🤝 コントリビューション

プロジェクトは開発初期段階です。以下の領域で貢献を歓迎します：

- RVCモデルの最適化
- 揺らぎアルゴリズムの改良
- 遅延削減のための技術検証
- クロスプラットフォーム対応の強化

**開発に参加する前に**: [.claude/CLAUDE.md](./.claude/CLAUDE.md) でプロジェクトの全体像を把握してください。

## 📄 ライセンス

MIT License

## 🙏 謝辞

- [RVC (Retrieval-based Voice Conversion)](https://github.com/RVC-Project/Retrieval-based-Voice-Conversion-WebUI)
- Sox コマンドによるボイスチェンジャー参考記事
  - https://qiita.com/Nabeshin/items/5a904fe0baf76a9bf651
  - https://qiita.com/teteyateiya/items/e4dc27e384d947b9946d
