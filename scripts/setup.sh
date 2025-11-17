#!/usr/bin/env bash
# Makebeliv セットアップスクリプト（uv使用）

set -euo pipefail

echo "🔧 Makebeliv セットアップ開始"
echo ""

# uvのインストール確認
if ! command -v uv &> /dev/null; then
    echo "❌ uv が見つかりません"
    echo ""
    echo "以下のコマンドでインストールしてください:"
    echo "  curl -LsSf https://astral.sh/uv/install.sh | sh"
    echo ""
    echo "または:"
    echo "  cargo install uv"
    exit 1
fi

echo "✅ uv: $(uv --version)"
echo ""

# 仮想環境の作成
echo "📦 仮想環境を作成中..."
uv venv .venv
echo "✅ 仮想環境作成完了: .venv"
echo ""

# 依存関係のインストール
echo "📥 依存関係をインストール中..."

# GPU確認
if command -v nvidia-smi &> /dev/null; then
    echo "🎮 NVIDIA GPU検出"
    echo "CUDA対応PyTorchをインストールします..."
    uv pip install torch torchaudio --index-url https://download.pytorch.org/whl/cu118
else
    echo "💻 CPU版PyTorchをインストールします..."
    uv pip install torch torchaudio --index-url https://download.pytorch.org/whl/cpu
fi

# その他の依存関係
uv pip install -r requirements.txt

echo "✅ 依存関係インストール完了"
echo ""

# Rustビルド
if command -v cargo &> /dev/null; then
    echo "🦀 Rustバイナリをビルド中..."
    cargo build --release
    echo "✅ ビルド完了: target/release/makebeliv"
else
    echo "⚠️  Rustがインストールされていません"
    echo "   makebelivコマンドを使用するにはRustが必要です:"
    echo "   https://rustup.rs/"
fi

echo ""
echo "🎉 セットアップ完了！"
echo ""
echo "次のステップ:"
echo "  1. 仮想環境を有効化:"
echo "     source .venv/bin/activate"
echo ""
echo "  2. APIサーバーを起動:"
echo "     makebeliv server"
echo ""
echo "  3. 音声処理を試す:"
echo "     makebeliv process -i audio/input/test.wav --use-api"
echo ""
