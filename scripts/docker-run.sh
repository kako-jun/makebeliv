#!/usr/bin/env bash
# Docker環境でMakebelivを実行

set -euo pipefail

MODE=${1:-gpu}

echo "🐳 Makebeliv Docker起動"
echo "   モード: $MODE"
echo ""

case $MODE in
    gpu)
        echo "🎮 GPU版を起動します"
        docker-compose up -d api-server
        ;;
    cpu)
        echo "💻 CPU版を起動します"
        docker-compose --profile cpu up -d api-server-cpu
        ;;
    build)
        echo "🔨 イメージをビルドします"
        docker-compose build
        ;;
    down)
        echo "🛑 コンテナを停止します"
        docker-compose down
        ;;
    logs)
        echo "📄 ログを表示します"
        docker-compose logs -f
        ;;
    *)
        echo "使用法: $0 {gpu|cpu|build|down|logs}"
        exit 1
        ;;
esac

echo ""
echo "✅ 完了"
echo ""
echo "APIサーバー: http://localhost:8000"
echo "ドキュメント: http://localhost:8000/docs"
