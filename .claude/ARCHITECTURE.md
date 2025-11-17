# Makebeliv アーキテクチャドキュメント

## 🏗️ システム全体像

```
┌─────────────────────────────────────────────────────────────────┐
│                         User Interface                          │
│                    (CLI / 将来: WebUI)                          │
└────────────────────────┬────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Rust Layer (制御層)                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │   CLI        │  │  Audio I/O   │  │ HTTP Client  │         │
│  │  (clap)      │  │   (cpal)     │  │ (reqwest)    │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
└────────────────────────┬───────────────────┬────────────────────┘
                         │                   │ HTTP POST
                         │ Audio Stream      │ (localhost)
                         ▼                   ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Python Layer (処理層)                          │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              FastAPI Server (非同期)                      │  │
│  │  - /convert: ファイル変換                                 │  │
│  │  - /convert-chunk: チャンク変換                           │  │
│  │  - /status: サーバーステータス                            │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              ▼                                   │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │ RVC Engine   │  │ Fluctuation  │  │ Noise Mixer  │         │
│  │ (PyTorch)    │  │   Engine     │  │              │         │
│  └──────────────┘  └──────────────┘  └──────────────┘         │
│         │                  │                  │                  │
│         └──────────────────┴──────────────────┘                 │
│                            ▼                                     │
│                   Audio Processing                               │
│                   Pipeline (NumPy)                               │
└─────────────────────────────────────────────────────────────────┘
                            ▼
                   ┌─────────────────┐
                   │  Virtual Mic    │
                   │  / Speaker      │
                   └─────────────────┘
```

---

## 📦 レイヤー構成

### Layer 1: User Interface

#### CLI (Rust)
- **責務**: ユーザーコマンドの受付と実行
- **実装**: `src/main.rs`
- **コマンド**:
  - `setup` - 環境構築
  - `server` - APIサーバー起動
  - `process` - ファイル処理
  - `monitor` - リアルタイム変換
  - `list-devices` - デバイス一覧

### Layer 2: Control Layer (Rust)

#### 2.1 Audio I/O (`src/audio.rs`)
- **責務**: 低レイテンシ音声入出力
- **技術**: cpal (クロスプラットフォーム音声ライブラリ)
- **機能**:
  - マイク入力ストリーム
  - スピーカー出力ストリーム
  - リングバッファ管理
  - デバイス列挙

**なぜRustか？**
- GIL (Global Interpreter Lock) の影響を受けない
- GC (Garbage Collection) による遅延がない
- ノンブロッキング処理が得意

#### 2.2 HTTP Client (`src/client.rs`)
- **責務**: Python APIサーバーとの通信
- **技術**: reqwest + tokio (非同期)
- **機能**:
  - ファイル変換リクエスト
  - チャンク変換リクエスト
  - ステータス確認
  - セッション管理

**通信フォーマット**:
```
Request:
  - Method: POST
  - Content-Type: multipart/form-data
  - Body: audio (binary), model, pitch_shift, etc.

Response:
  - Content-Type: audio/wav
  - Headers: X-Processing-Time-Ms, X-Audio-Length-Seconds
```

### Layer 3: Processing Layer (Python)

#### 3.1 FastAPI Server (`python/api_server.py`)
- **責務**: HTTPエンドポイント提供、リクエスト処理
- **技術**: FastAPI + Uvicorn (ASGI)
- **機能**:
  - 非同期音声変換
  - セッション管理（揺らぎの連続性のため）
  - エラーハンドリング

**エンドポイント設計**:

| エンドポイント | メソッド | 用途 | レスポンス時間目標 |
|---------------|---------|------|-------------------|
| `/status` | GET | ヘルスチェック | < 10ms |
| `/convert` | POST | ファイル変換 | 音声長に依存 |
| `/convert-chunk` | POST | チャンク変換 | < 200ms |
| `/reset-session` | POST | セッションリセット | < 10ms |

#### 3.2 RVC Engine (`python/rvc_engine.py`)
- **責務**: 音声変換（声質変換）
- **技術**: PyTorch + CUDA
- **機能**:
  - モデルロード・管理
  - F0抽出（ピッチ）
  - 声質変換推論
  - デモモード（モデルなしでピッチシフト）

**処理フロー**:
```
入力音声
  ↓
F0抽出 (librosa.pyin / harvest)
  ↓
ピッチシフト適用
  ↓
RVCモデル推論 (PyTorch)
  ↓
変換後音声
```

#### 3.3 Fluctuation Engine (`python/fluctuation.py`)
- **責務**: 人間らしい揺らぎの生成
- **技術**: NumPy (純粋Python)
- **機能**:
  - ピッチ揺らぎ生成
  - 音量揺らぎ生成
  - EQ揺らぎ生成
  - 時間的連続性の保持

**アルゴリズム**:
```python
# 前回の値から滑らかに遷移
def _smooth_transition(current, target, smoothness):
    return current * smoothness + target * (1 - smoothness)

# 現在の値 = 前回の値 * 0.8 + 新しい値 * 0.2
```

#### 3.4 Noise Mixer (`python/fluctuation.py`)
- **責務**: 背景ノイズの合成
- **機能**:
  - 環境音の生成（カフェ、街中、部屋など）
  - ノイズレベル調整
  - 音声とのミキシング

---

## 🔄 データフロー

### ファイル処理モード

```
[ユーザー] makebeliv process -i input.wav --use-api
    ↓
[Rust CLI] ファイル読み込み
    ↓
[HTTP] POST /convert (multipart/form-data)
    ↓
[FastAPI] リクエスト受信
    ↓
[RVC Engine] 音声変換
    ↓
[Fluctuation] 揺らぎ適用
    ↓
[Noise Mixer] ノイズ合成
    ↓
[FastAPI] レスポンス返却 (audio/wav)
    ↓
[Rust CLI] ファイル保存
    ↓
[ユーザー] 完了
```

### リアルタイムモード（計画）

```
[マイク] 音声入力 (cpal)
    ↓
[Rust] 100ms チャンクをバッファリング
    ↓
[HTTP] POST /convert-chunk
    ↓
[FastAPI] チャンク受信
    ↓
[RVC Engine] 高速変換
    ↓
[Fluctuation] 揺らぎ適用（軽量版）
    ↓
[FastAPI] 変換済みチャンク返却
    ↓
[Rust] チャンク受信
    ↓
[スピーカー/仮想マイク] 音声出力 (cpal)
```

**並列化**:
```
Time: 0ms    100ms   200ms   300ms   400ms
       ┌─────┬─────┬─────┬─────┬─────┐
録音:   │  A  │  B  │  C  │  D  │  E  │
       └─────┴─────┴─────┴─────┴─────┘
変換:         │  A  │  B  │  C  │  D  │
             └─────┴─────┴─────┴─────┘
再生:               │  A  │  B  │  C  │
                   └─────┴─────┴─────┘

総遅延: 200ms（2チャンク分）
```

---

## 🗂️ ディレクトリ構造

```
makebeliv/
├── src/                      # Rust層
│   ├── main.rs              # CLIエントリーポイント
│   ├── audio.rs             # 音声I/O (cpal)
│   ├── client.rs            # HTTPクライアント
│   └── lib.rs               # ライブラリルート
│
├── python/                   # Python層
│   ├── api_server.py        # FastAPIサーバー
│   ├── rvc_engine.py        # RVC変換エンジン
│   ├── fluctuation.py       # 揺らぎエンジン
│   ├── file_processor.py    # ファイル処理
│   └── __init__.py
│
├── scripts/                  # ユーティリティ
│   ├── setup.sh            # 環境構築
│   ├── test.sh             # テスト
│   └── docker-run.sh       # Docker管理
│
├── audio/                    # 音声ファイル
│   ├── input/              # 入力音声
│   └── output/             # 処理済み音声
│
├── models/                   # RVCモデル
│   ├── default/
│   ├── vtuber1/
│   └── ...
│
├── .claude/                  # プロジェクト管理
│   ├── CLAUDE.md           # 概要・構想
│   ├── ARCHITECTURE.md     # 本ファイル
│   ├── DESIGN.md           # 設計詳細
│   ├── DECISIONS.md        # 設計決定記録
│   └── TODO.md             # タスク管理
│
├── Cargo.toml                # Rust依存関係
├── pyproject.toml            # Python環境設定
├── requirements.txt          # Python依存関係
├── Dockerfile                # GPU版イメージ
├── Dockerfile.cpu            # CPU版イメージ
└── docker-compose.yml        # Docker構成
```

---

## ⚡ パフォーマンス設計

### 遅延の内訳（目標値）

| 処理 | 遅延 | 最適化手法 |
|------|------|-----------|
| マイク入力バッファリング | 50-100ms | チャンクサイズ調整 |
| HTTP通信 (Rust→Python) | 5-10ms | localhost、非同期 |
| RVC推論 (GPU) | 100-150ms | モデル軽量化、fp16 |
| 揺らぎ適用 | < 5ms | NumPy最適化 |
| HTTP通信 (Python→Rust) | 5-10ms | ストリーミング |
| スピーカー出力バッファリング | 50-100ms | バッファサイズ調整 |
| **合計** | **215-375ms** | **目標: 300ms以内** |

### ボトルネック対策

1. **RVC推論が最大のボトルネック**
   - 対策: モデルの軽量化、fp16化、ONNX化検討

2. **HTTP通信のオーバーヘッド**
   - 対策: localhost使用、キープアライブ、並列処理

3. **チャンク境界の不自然さ**
   - 対策: クロスフェード、オーバーラップ処理

---

## 🔐 セキュリティ設計

### 原音漏洩防止

1. **物理マイクの無効化**
   - Discordなどで物理マイクを選択不可にする
   - 仮想マイクのみを有効化

2. **強制フィルター**
   - 入力音声に必ず変換処理を通す
   - 変換失敗時は無音またはノイズで代替

3. **テスト機能**
   - モニタリング機能で変換後の音声を確認
   - 原音が混ざっていないかチェック

### API セキュリティ

- **ローカルホストのみ**: デフォルトで127.0.0.1にバインド
- **認証なし**: ローカル専用のため不要
- **HTTPS不要**: ローカル通信のため平文OK

---

## 🐳 デプロイメント設計

### ローカル実行

```bash
# 開発モード
uv run uvicorn python.api_server:app --reload

# 本番モード
makebeliv server
```

### Docker実行

**GPU版**:
- ベースイメージ: `nvidia/cuda:11.8.0-cudnn8-runtime-ubuntu22.04`
- PyTorch: CUDA 11.8対応
- 必要: NVIDIA Docker Runtime

**CPU版**:
- ベースイメージ: `ubuntu:22.04`
- PyTorch: CPU版
- GPU不要

---

## 📊 スケーラビリティ

### 現状（シングルユーザー想定）

- 1ユーザー、1セッション
- ローカルマシンで完結
- スケールアウト不要

### 将来的な拡張（オプション）

- マルチユーザー対応（WebUIの場合）
- 負荷分散（複数GPUの活用）
- キュー管理（リクエスト多数時）

---

## 🧪 テスト戦略

### ユニットテスト

- Python: pytest
- Rust: cargo test

### 統合テスト

- ファイル処理のエンドツーエンドテスト
- API呼び出しのテスト

### パフォーマンステスト

- 遅延測定スクリプト
- プロファイリング（cProfile, flamegraph）

---

## 🔄 将来の拡張性

### 計画中

- リアルタイムモード完全実装
- 仮想マイク統合
- WebUI（オプション）

### 検討中

- モデルのホットスワップ
- プリセット管理
- 音声録音機能
- エフェクトチェーン（リバーブ、コンプレッサーなど）

---

**最終更新**: 2025-11-17
**バージョン**: 1.0
**ステータス**: Phase 3 進行中
