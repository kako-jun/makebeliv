# RVCモデル格納ディレクトリ

このディレクトリには、RVC（Retrieval-based Voice Conversion）の学習済みモデルを配置します。

## モデルの取得方法

### 1. 既存モデルのダウンロード

RVC公式やコミュニティから公開されているモデルを使用できます：

- [RVC-Models Collection](https://huggingface.co/models?search=rvc)
- [RVC WebUI](https://github.com/RVC-Project/Retrieval-based-Voice-Conversion-WebUI)

### 2. 自作モデルのトレーニング

独自の声を学習させることも可能です：

1. RVC WebUIをセットアップ
2. 音声データを収集（10分以上推奨）
3. トレーニング実行
4. 生成された.pthファイルをこのディレクトリに配置

## ファイル構成例

```
models/
├── vtuber1/
│   ├── model.pth          # モデルファイル
│   ├── config.json        # 設定ファイル
│   └── index.faiss        # インデックスファイル（オプション）
├── vtuber2/
│   └── ...
└── README.md
```

## 使用方法

```bash
# モデルを指定して変換
makebeliv process -i input.wav --model vtuber1
```

## 注意事項

- モデルファイルは通常数百MB以上になるため、Gitにはコミットしません
- 商用利用の際は、モデルのライセンスを確認してください
- 個人の声をトレーニングする際は、本人の同意を得てください
