#!/usr/bin/env bash
# Makebeliv テストスクリプト

set -euo pipefail

echo "🧪 Makebeliv テスト実行"
echo ""

# 仮想環境の確認
if [ ! -d ".venv" ]; then
    echo "❌ 仮想環境が見つかりません"
    echo "   ./scripts/setup.sh を実行してください"
    exit 1
fi

# 仮想環境を有効化
source .venv/bin/activate

echo "✅ Python環境: $(python --version)"
echo "✅ uv: $(uv --version)"
echo ""

# Pythonモジュールのテスト
echo "📋 Pythonモジュールテスト"
echo ""

echo "  - fluctuation.py"
uv run python python/fluctuation.py

echo ""
echo "  - rvc_engine.py"
# RVCエンジンは引数が必要なのでスキップ
echo "    (スキップ - ファイル引数が必要)"

echo ""
echo "✅ 全テスト完了"
