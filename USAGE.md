# Makebeliv 使用ガイド

## 基本的な使い方

### 1. 環境構築

#### uvを使ったセットアップ（推奨）

```bash
# 自動セットアップスクリプト
./scripts/setup.sh

# または手動で
uv venv .venv
source .venv/bin/activate
uv pip install -r requirements.txt

# CUDA対応PyTorch（GPU環境）
uv pip install torch torchaudio --index-url https://download.pytorch.org/whl/cu118

# CPU版PyTorch
uv pip install torch torchaudio --index-url https://download.pytorch.org/whl/cpu
```

#### Rustバイナリのビルド

```bash
cargo build --release
```

### 2. APIサーバーの起動

```bash
# makebeliv CLIを使用
./target/release/makebeliv server

# または直接uvを使用
uv run uvicorn python.api_server:app --host 0.0.0.0 --port 8000 --reload
```

サーバーが起動したら http://localhost:8000/docs でAPIドキュメントを確認できます。

### 3. 音声処理

#### ファイル処理（API経由）

```bash
# 基本的な使い方
makebeliv process -i audio/input/test.wav --use-api

# ピッチシフト +3半音
makebeliv process -i audio/input/test.wav --use-api --pitch 3

# 出力先を指定
makebeliv process -i audio/input/test.wav -o audio/output/result.wav --use-api

# ノイズタイプを変更
makebeliv process -i audio/input/test.wav --use-api --noise street
```

#### ファイル処理（直接実行）

APIサーバーなしで直接Pythonスクリプトを実行：

```bash
makebeliv process -i audio/input/test.wav

# または
uv run python python/file_processor.py audio/input/test.wav
```

### 4. リアルタイム変換（開発中）

```bash
makebeliv monitor --model default --noise cafe --pitch 3
```

現在は開発中のため、ファイル処理モードを使用してください。

## 高度な使い方

### RVCモデルの配置

1. RVCモデルファイル（.pth）を取得
2. `models/<モデル名>/` ディレクトリに配置

```
models/
├── default/
│   └── model.pth
├── vtuber1/
│   └── model.pth
└── vtuber2/
    └── model.pth
```

3. モデルを指定して実行

```bash
makebeliv process -i input.wav --use-api --model vtuber1
```

### 揺らぎエンジンのカスタマイズ

`python/file_processor.py` の `FluctuationConfig` を編集：

```python
config = ProcessConfig(
    input_path=input_file,
    output_path=output_file,
    fluctuation_config=FluctuationConfig(
        pitch_variation=0.08,  # ピッチ揺らぎを大きく
        volume_variation=0.05,  # 音量揺らぎを大きく
        temporal_smoothness=0.9  # より滑らかに
    )
)
```

### APIを直接使用

#### curlでの例

```bash
# 音声変換
curl -X POST "http://localhost:8000/convert" \
  -F "audio=@audio/input/test.wav" \
  -F "model=default" \
  -F "pitch_shift=3" \
  -F "noise_type=cafe" \
  -F "noise_level=0.02" \
  --output audio/output/converted.wav

# サーバーステータス確認
curl http://localhost:8000/status
```

#### Pythonでの例

```python
import requests

# 音声ファイルを変換
with open("audio/input/test.wav", "rb") as f:
    files = {"audio": f}
    data = {
        "model": "default",
        "pitch_shift": 3,
        "noise_type": "cafe",
        "noise_level": 0.02
    }

    response = requests.post(
        "http://localhost:8000/convert",
        files=files,
        data=data
    )

    with open("audio/output/result.wav", "wb") as out:
        out.write(response.content)
```

## Docker環境での使用

詳細は [DOCKER.md](./DOCKER.md) を参照してください。

### 基本的な使い方

```bash
# GPU版
./scripts/docker-run.sh build
./scripts/docker-run.sh gpu

# CPU版
./scripts/docker-run.sh cpu

# ログ確認
./scripts/docker-run.sh logs

# 停止
./scripts/docker-run.sh down
```

## トラブルシューティング

### uvが見つからない

```bash
# uvをインストール
curl -LsSf https://astral.sh/uv/install.sh | sh

# または
cargo install uv
```

### PyTorchがGPUを認識しない

```bash
# GPUが利用可能か確認
nvidia-smi

# PyTorchでGPU確認
uv run python -c "import torch; print(torch.cuda.is_available())"

# CUDA版を再インストール
uv pip uninstall torch torchaudio
uv pip install torch torchaudio --index-url https://download.pytorch.org/whl/cu118
```

### APIサーバーに接続できない

```bash
# サーバーが起動しているか確認
curl http://localhost:8000/status

# ファイアウォールの確認
sudo ufw status
sudo ufw allow 8000/tcp

# ポート使用状況の確認
lsof -i :8000
```

### 音声ファイルが処理できない

```bash
# サポートされている形式
# - WAV (推奨)
# - MP3
# - FLAC

# ffmpegで変換
ffmpeg -i input.mp3 -ar 16000 -ac 1 output.wav
```

## パフォーマンスチューニング

### GPU使用率の最大化

```python
# python/rvc_engine.py の設定を調整
config = RVCConfig(
    device="cuda",
    # バッチサイズを増やす（メモリに余裕がある場合）
    # チャンクサイズを調整
)
```

### 遅延の測定

APIレスポンスヘッダーに処理時間が含まれています：

```bash
curl -v -X POST "http://localhost:8000/convert" \
  -F "audio=@test.wav" \
  2>&1 | grep "X-Processing-Time-Ms"
```

### メモリ使用量の最適化

```bash
# Docker環境でのメモリ制限
docker-compose.yml に追加:

deploy:
  resources:
    limits:
      memory: 8G
```

## 次のステップ

1. [DOCKER.md](./DOCKER.md) - Docker環境での実行
2. [models/README.md](./models/README.md) - RVCモデルの準備
3. GitHubリポジトリ - 最新情報とIssue報告
