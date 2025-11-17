# Docker実行ガイド

MakebelivをDockerで実行する方法を説明します。

## クイックスタート

### GPU版（NVIDIA GPU搭載環境）

```bash
# イメージをビルド
./scripts/docker-run.sh build

# APIサーバーを起動
./scripts/docker-run.sh gpu

# 動作確認
curl http://localhost:8000/status
```

### CPU版（GPU非搭載環境）

```bash
# CPU版イメージをビルド
docker-compose build api-server-cpu

# APIサーバーを起動
./scripts/docker-run.sh cpu

# 動作確認
curl http://localhost:8001/status
```

## 詳細

### 前提条件

#### GPU版
- Docker 20.10以上
- NVIDIA Docker Runtime
- CUDA 11.8対応GPU

#### CPU版
- Docker 20.10以上

### NVIDIA Docker Runtimeのセットアップ

```bash
# Ubuntu/Debian
distribution=$(. /etc/os-release;echo $ID$VERSION_ID)
curl -s -L https://nvidia.github.io/nvidia-docker/gpgkey | sudo apt-key add -
curl -s -L https://nvidia.github.io/nvidia-docker/$distribution/nvidia-docker.list | \
    sudo tee /etc/apt/sources.list.d/nvidia-docker.list

sudo apt-get update
sudo apt-get install -y nvidia-docker2
sudo systemctl restart docker
```

### コンテナ管理

```bash
# ログ確認
./scripts/docker-run.sh logs

# コンテナ停止
./scripts/docker-run.sh down

# 再起動
docker-compose restart
```

### ボリュームマウント

デフォルトで以下のディレクトリがマウントされます：

- `./models` → `/app/models` - RVCモデル
- `./audio` → `/app/audio` - 入出力音声

### 音声処理の実行

コンテナ起動後、ローカルマシンから：

```bash
# makebeliv CLIを使用（API経由）
makebeliv process -i audio/input/test.wav --use-api

# または直接curlで
curl -X POST "http://localhost:8000/convert" \
  -F "audio=@audio/input/test.wav" \
  -F "model=default" \
  -F "pitch_shift=3" \
  --output audio/output/converted.wav
```

### トラブルシューティング

#### GPUが認識されない

```bash
# Docker内でGPU確認
docker exec -it makebeliv-api nvidia-smi

# GPUが表示されない場合
sudo systemctl restart docker
docker-compose down
docker-compose up -d
```

#### ポートが使用中

```bash
# docker-compose.ymlのポート番号を変更
ports:
  - "8080:8000"  # ホスト側ポートを変更
```

## イメージサイズの最適化

現在のイメージは以下の構成です：

- ベースイメージ: nvidia/cuda:11.8.0（~4GB）
- Python依存関係: ~2GB
- Rustバイナリ: ~10MB

合計: 約6-7GB

### 軽量化のヒント

1. **マルチステージビルド** - すでに実装済み
2. **不要なファイルの除外** - `.dockerignore`で設定済み
3. **Alpine Linuxの使用** - CUDAサポートの制約があるため非推奨

## 環境変数

| 変数名 | デフォルト | 説明 |
|--------|-----------|------|
| `CUDA_VISIBLE_DEVICES` | `0` | 使用するGPU番号 |
| `PYTHONUNBUFFERED` | `1` | Python出力のバッファリング無効 |

## docker-compose.ymlカスタマイズ例

### 複数GPUの使用

```yaml
deploy:
  resources:
    reservations:
      devices:
        - driver: nvidia
          count: 2  # GPU 2枚使用
          capabilities: [gpu]
```

### メモリ制限

```yaml
deploy:
  resources:
    limits:
      memory: 8G
    reservations:
      memory: 4G
```

## 本番環境での使用

### セキュリティ

```yaml
# 非rootユーザーで実行
user: "1000:1000"

# 読み取り専用ファイルシステム
read_only: true

# 一時ファイル用の書き込み可能ボリューム
tmpfs:
  - /tmp
```

### ヘルスチェック

```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8000/status"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 40s
```

### ログローテーション

```yaml
logging:
  driver: "json-file"
  options:
    max-size: "10m"
    max-file: "3"
```
