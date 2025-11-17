# Makebeliv Dockerfile
# uvを使用したPython環境 + Rust CLI

FROM nvidia/cuda:11.8.0-cudnn8-runtime-ubuntu22.04 AS base

# 環境変数
ENV DEBIAN_FRONTEND=noninteractive \
    PYTHONUNBUFFERED=1 \
    LANG=C.UTF-8 \
    LC_ALL=C.UTF-8

# 基本パッケージのインストール
RUN apt-get update && apt-get install -y \
    curl \
    git \
    build-essential \
    libsndfile1 \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# uvのインストール
RUN curl -LsSf https://astral.sh/uv/install.sh | sh
ENV PATH="/root/.cargo/bin:${PATH}"

# 作業ディレクトリ
WORKDIR /app

# Pythonプロジェクトファイルをコピー
COPY pyproject.toml requirements.txt ./
COPY python ./python

# Python依存関係のインストール（uvを使用）
RUN uv venv .venv && \
    . .venv/bin/activate && \
    uv pip install -r requirements.txt && \
    uv pip install torch torchaudio --index-url https://download.pytorch.org/whl/cu118

# モデルディレクトリ作成
RUN mkdir -p models audio/input audio/output

# Rustステージ（マルチステージビルド）
FROM base AS rust-builder

# Rustのインストール
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Rustプロジェクトファイルをコピー
COPY Cargo.toml ./
COPY src ./src

# Rustビルド
RUN cargo build --release

# 最終イメージ
FROM base

# Rustバイナリをコピー
COPY --from=rust-builder /app/target/release/makebeliv /usr/local/bin/makebeliv

# ポート公開
EXPOSE 8000

# デフォルトコマンド（APIサーバー起動）
CMD ["makebeliv", "server", "--host", "0.0.0.0", "--port", "8000"]
