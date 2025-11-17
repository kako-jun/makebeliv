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

### 1. セットアップ

```bash
# Rustプロジェクトをビルド
cargo build --release

# Python環境を自動構築（uvを使用）
./target/release/makebeliv setup
```

### 2. テスト実行（ファイルベース）

```bash
# テスト用音声ファイルを配置
cp your_voice.wav audio/input/test.wav

# 音声変換を実行
./target/release/makebeliv process -i audio/input/test.wav
```

### 3. リアルタイムモード（開発中）

```bash
makebeliv monitor --model vtuber1 --noise cafe --pitch +3
```

## 📦 技術スタック

### Python側
- **RVC (Retrieval-based Voice Conversion)** - 高品質な声変換
- **PyTorch** - GPU推論
- **librosa / scipy** - 音声処理
- **FastAPI** - HTTPサーバー（将来）

### Rust側
- **clap** - CLIインターフェース
- **cpal / rodio** - 低レイテンシ音声I/O（将来）
- **tokio** - 非同期処理
- **uv統合** - Python環境の自動構築

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
├── src/                   # Rustソースコード
│   └── main.rs           # CLIエントリーポイント
├── python/               # Pythonモジュール
│   ├── fluctuation.py    # 揺らぎエンジン
│   ├── file_processor.py # ファイルベース処理
│   └── rvc_engine.py     # RVC変換（TODO）
├── audio/                # 音声ファイル
│   ├── input/           # 入力音声
│   └── output/          # 処理済み音声
├── models/              # RVCモデル
├── Cargo.toml           # Rust依存関係
├── pyproject.toml       # Python環境設定
└── requirements.txt     # Python依存関係
```

## 🧪 開発ステータス

- [x] プロジェクト構造設計
- [x] Python環境自動構築（uv統合）
- [x] 揺らぎエンジン基本実装
- [x] ファイルベース処理フロー
- [ ] RVC変換統合
- [ ] HTTPサーバー実装
- [ ] Rust音声I/O実装
- [ ] 仮想マイク出力対応
- [ ] リアルタイムモード実装

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
# セットアップ
makebeliv setup [--yes]

# ファイル処理
makebeliv process -i <input> [-o <output>] [--model <model>] [--noise <type>] [--pitch <shift>]

# リアルタイム変換（開発中）
makebeliv monitor --model <model> --noise <type> --pitch <shift>
```

## 🤝 コントリビューション

プロジェクトは開発初期段階です。以下の領域で貢献を歓迎します：

- RVCモデルの最適化
- 揺らぎアルゴリズムの改良
- 遅延削減のための技術検証
- クロスプラットフォーム対応の強化

## 📄 ライセンス

MIT License

## 🙏 謝辞

- [RVC (Retrieval-based Voice Conversion)](https://github.com/RVC-Project/Retrieval-based-Voice-Conversion-WebUI)
- Sox コマンドによるボイスチェンジャー参考記事
  - https://qiita.com/Nabeshin/items/5a904fe0baf76a9bf651
  - https://qiita.com/teteyateiya/items/e4dc27e384d947b9946d
