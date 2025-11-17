# Makebeliv 設計ドキュメント

## 🎨 設計原則

### 1. 自然さ優先 (Naturalness First)

品質よりも自然さを優先する。電話レベルの音質でも、人間らしい揺らぎがあれば自然に聞こえる。

### 2. 遅延最小化 (Low Latency)

リアルタイム性を重視。200-300msの遅延を目標とする。

### 3. シンプルな構成 (Simplicity)

過度な複雑化を避ける。必要最小限の技術スタックで実現する。

### 4. クロスプラットフォーム (Cross-Platform)

Windows/macOS/Linuxで動作すること。

---

## 🧩 主要コンポーネント設計

### 1. 揺らぎエンジン (Fluctuation Engine)

#### 目的
人間の声の自然な変化を再現し、機械的な印象を減らす。

#### 設計

```python
@dataclass
class FluctuationConfig:
    pitch_variation: float = 0.05      # ±5%
    volume_variation: float = 0.03     # ±3%
    eq_variation: float = 0.02         # ±2%
    temporal_smoothness: float = 0.8   # 0-1
```

**時間的連続性の実装**:
```python
def _smooth_transition(current, target, smoothness):
    """前回の値から滑らかに遷移"""
    return current * smoothness + target * (1 - smoothness)
```

- `smoothness = 0.8` の場合:
  - 前回の値を80%維持
  - 新しい値を20%混ぜる
  - 急激な変化を防ぐ

**ランダム性**:
```python
target = 1.0 + rng.normal(0, pitch_variation)
```

- 正規分布に従うランダム値
- 平均=1.0、標準偏差=0.05
- 約68%が0.95-1.05の範囲に収まる

#### 適用タイミング

1. **RVC変換後に適用** - 変換済みの音声に揺らぎを加える
2. **チャンクごとに適用** - 連続性を保つため、前回の値を記憶

#### 状態管理

```python
class FluctuationEngine:
    def __init__(self):
        self.prev_pitch_factor = 1.0
        self.prev_volume_factor = 1.0
        # ...
```

セッションIDごとにエンジンを保持することで、長い発話でも連続性を保つ。

---

### 2. RVC変換エンジン (RVC Engine)

#### 処理フロー

```
入力音声 (16kHz, モノラル)
    ↓
[1] F0抽出 (ピッチ情報)
    ↓
[2] ピッチシフト適用
    ↓
[3] RVCモデル推論
    ↓
出力音声
```

#### F0抽出方法

| 方法 | 特徴 | 速度 | 精度 |
|------|------|------|------|
| **harvest** | ロバスト、ノイズに強い | 中 | 高 |
| **crepe** | 深層学習ベース | 遅 | 最高 |
| **pm** | 高速 | 速 | 中 |

現在の実装: **librosa.pyin** (harvest相当)

#### デモモード

RVCモデルがない場合の代替処理：

```python
def _demo_conversion(audio, sr):
    """ピッチシフトのみ適用"""
    return librosa.effects.pitch_shift(
        audio, sr=sr, n_steps=self.config.f0_up_key
    )
```

開発・テスト時に便利。

---

### 3. FastAPIサーバー (API Server)

#### エンドポイント設計

**GET /status**
```json
{
  "status": "running",
  "device": "cuda",
  "models_loaded": 2,
  "uptime_seconds": 1234.56
}
```

**POST /convert**
- Input: multipart/form-data (audio, model, pitch_shift, ...)
- Output: audio/wav
- Headers: `X-Processing-Time-Ms`, `X-Audio-Length-Seconds`

**POST /convert-chunk**
- 軽量版（ノイズなし、セッション管理のみ）
- リアルタイム用

**POST /reset-session**
- 揺らぎエンジンの状態をリセット
- 新しい発話の開始時に使用

#### 状態管理

```python
class ServerState:
    def __init__(self):
        self.rvc_engines = {}          # モデル名 -> RVCEngine
        self.fluctuation_engines = {}   # セッションID -> FluctuationEngine
```

- RVCエンジンはモデルごとにキャッシュ
- 揺らぎエンジンはセッションごとに保持

---

### 4. 音声I/O (Audio I/O)

#### リングバッファ設計

```rust
pub struct AudioBuffer {
    buffer: Arc<Mutex<Vec<f32>>>,
    capacity: usize,
}

impl AudioBuffer {
    pub fn push(&self, data: &[f32]) {
        // 容量超過時は古いデータを削除
        if buffer.len() + data.len() > capacity {
            let overflow = buffer.len() + data.len() - capacity;
            buffer.drain(0..overflow);
        }
        buffer.extend_from_slice(data);
    }
}
```

**特徴**:
- スレッドセーフ (`Arc<Mutex>`)
- 固定容量（メモリリーク防止）
- FIFO (First In, First Out)

#### ストリーム処理

```rust
let stream = device.build_input_stream(
    &config,
    move |data: &[f32], _| {
        // コールバック内で音声データを処理
        callback(data);
    },
    |err| { warn!("Error: {}", err); },
    None,
)?;
```

**ノンブロッキング**:
- コールバックは別スレッドで実行
- メインスレッドをブロックしない

---

## 🔧 技術的な設計決定

### チャンクサイズの選定

**検討した選択肢**:
- 50ms: 低遅延だが処理回数が多い、チャンク境界が目立つ
- 100ms: **採用** - バランスが良い
- 200ms: 処理回数は少ないが遅延が大きい

**決定**:
- デフォルト: **100ms**
- 設定可能にする（将来）

### サンプリングレート

- **16kHz**: RVCの推奨、計算量が少ない
- 44.1kHz/48kHz: 高音質だが計算量が多い

**決定**: **16kHz** (RVCの推奨に従う)

### 通信プロトコル

**検討した選択肢**:
1. HTTP POST (multipart) - **採用**
2. gRPC - 高速だが複雑
3. WebSocket - リアルタイムだが実装が複雑
4. IPC (Unix Domain Socket) - 高速だがクロスプラットフォームに難

**決定理由**:
- クロスプラットフォーム対応が容易
- 実装がシンプル
- 遅延も許容範囲（5-10ms程度）

---

## 📐 データ構造設計

### 音声データ表現

```python
# NumPy配列（1次元）
audio: np.ndarray  # shape: (samples,), dtype: float32

# サンプリングレート
sr: int = 16000

# 長さ（秒）
duration = len(audio) / sr
```

### 設定データ

```python
# ProcessConfig（ファイル処理）
@dataclass
class ProcessConfig:
    input_path: str
    output_path: str
    target_sr: int = 16000
    enable_fluctuation: bool = True
    enable_noise: bool = True
    pitch_shift: int = 0
    rvc_model_path: str = "models/default/model.pth"

# RVCConfig（RVC変換）
@dataclass
class RVCConfig:
    model_path: str
    device: str = "cuda" if torch.cuda.is_available() else "cpu"
    f0_up_key: int = 0
    # ...
```

---

## 🎯 エラーハンドリング設計

### Python側

```python
try:
    # 処理
except Exception as e:
    logger.error(f"エラー: {e}", exc_info=True)
    raise HTTPException(status_code=500, detail=str(e))
```

- FastAPIが自動的にJSON形式でエラーを返す
- ログに詳細を記録

### Rust側

```rust
fn process_audio(...) -> Result<()> {
    let audio_bytes = std::fs::read(input)
        .context("入力ファイル読み込みエラー")?;

    // ...

    Ok(())
}
```

- `anyhow::Result`でエラーチェーン
- `.context()`で文脈情報を追加

---

## 🧪 テスタビリティ

### モジュール分離

```
python/
├── api_server.py       # エンドポイント定義のみ
├── rvc_engine.py       # RVC処理（独立）
├── fluctuation.py      # 揺らぎ処理（独立）
└── file_processor.py   # ファイル処理（統合）
```

各モジュールは単独でテスト可能。

### テストデータ

```
tests/
├── fixtures/
│   ├── test_audio.wav  # テスト用音声
│   └── expected.wav    # 期待される出力
└── test_rvc_engine.py
```

---

## 🚀 最適化戦略

### 1. PyTorch最適化

```python
# fp16化（半精度浮動小数点）
model = model.half()

# JITコンパイル
model = torch.jit.script(model)

# ONNX変換（将来）
torch.onnx.export(model, ...)
```

### 2. NumPy最適化

```python
# ベクトル化演算
audio = audio * volume_factor  # 全要素に一度に適用

# インプレース演算
audio *= volume_factor  # メモリコピー不要
```

### 3. Rust最適化

```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
```

---

## 🔮 将来の拡張ポイント

### プラグインシステム（検討中）

```python
class EffectPlugin(Protocol):
    def apply(self, audio: np.ndarray, sr: int) -> np.ndarray:
        ...

# プラグインチェーン
effects = [ReverbPlugin(), CompressorPlugin(), ...]
for effect in effects:
    audio = effect.apply(audio, sr)
```

### モデル管理（検討中）

```python
class ModelManager:
    def list_models(self) -> List[str]:
        ...

    def load_model(self, name: str) -> RVCEngine:
        ...

    def hot_swap(self, old_name: str, new_name: str):
        ...
```

---

**最終更新**: 2025-11-17
**バージョン**: 1.0
**ステータス**: Phase 3 進行中
