"""
ファイルベース音声処理

開発初期段階として、マイク入力ではなくファイルから音声を読み込んで処理します。
これにより：
- 再現性の高いテスト
- デバッグの容易さ
- 品質確認のしやすさ
が実現できます。
"""

import soundfile as sf
import numpy as np
from pathlib import Path
from typing import Tuple, Optional
from dataclasses import dataclass

from fluctuation import FluctuationEngine, FluctuationConfig, add_background_noise
from rvc_engine import RVCEngine, RVCConfig


@dataclass
class ProcessConfig:
    """音声処理の設定"""

    # 入出力
    input_path: str
    output_path: str

    # 音声パラメータ
    target_sr: int = 16000  # RVCの推奨サンプリングレート

    # 揺らぎ設定
    enable_fluctuation: bool = True
    fluctuation_config: Optional[FluctuationConfig] = None

    # ノイズ設定
    enable_noise: bool = True
    noise_type: str = "cafe"
    noise_level: float = 0.02

    # ピッチシフト（手動調整）
    pitch_shift: int = 0  # 半音単位

    # RVCモデル設定
    rvc_model_path: str = "models/default/model.pth"


class AudioFileProcessor:
    """音声ファイル処理クラス"""

    def __init__(self, config: ProcessConfig):
        self.config = config
        self.fluctuation_engine = None
        self.rvc_engine = None

        # 揺らぎエンジン初期化
        if config.enable_fluctuation:
            fluct_config = config.fluctuation_config or FluctuationConfig()
            self.fluctuation_engine = FluctuationEngine(fluct_config)

        # RVCエンジン初期化
        rvc_config = RVCConfig(
            model_path=config.rvc_model_path,
            f0_up_key=config.pitch_shift
        )
        self.rvc_engine = RVCEngine(rvc_config)

    def load_audio(self) -> Tuple[np.ndarray, int]:
        """音声ファイルを読み込む"""
        audio, sr = sf.read(self.config.input_path)

        # モノラルに変換
        if len(audio.shape) > 1:
            audio = np.mean(audio, axis=1)

        # リサンプリング（必要なら）
        if sr != self.config.target_sr:
            # TODO: librosa.resampleを使った実装
            print(f"警告: サンプリングレート変換が必要です ({sr} -> {self.config.target_sr})")

        return audio, sr

    def apply_rvc_conversion(self, audio: np.ndarray, sr: int) -> np.ndarray:
        """RVC変換を適用"""
        if not self.rvc_engine:
            print("  [RVC] エンジン未初期化")
            return audio

        return self.rvc_engine.convert(audio, sr)

    def apply_fluctuations(self, audio: np.ndarray, sr: int) -> np.ndarray:
        """揺らぎを適用"""
        if not self.fluctuation_engine:
            return audio

        # 音量の揺らぎ
        audio = self.fluctuation_engine.apply_volume_fluctuation(audio)

        # ピッチの揺らぎ（TODO: 実装）
        # audio = self.fluctuation_engine.apply_pitch_fluctuation(audio, sr)

        return audio

    def apply_noise(self, audio: np.ndarray, sr: int) -> np.ndarray:
        """背景ノイズを追加"""
        if not self.config.enable_noise:
            return audio

        return add_background_noise(
            audio,
            noise_type=self.config.noise_type,
            noise_level=self.config.noise_level,
            sr=sr
        )

    def process(self) -> Path:
        """音声処理のメインフロー"""
        print(f"入力: {self.config.input_path}")

        # 1. 読み込み
        print("[1/4] 音声ファイルを読み込み中...")
        audio, sr = self.load_audio()
        print(f"  サンプリングレート: {sr}Hz, 長さ: {len(audio)/sr:.2f}秒")

        # 2. RVC変換
        print("[2/4] RVC変換を適用中...")
        audio = self.apply_rvc_conversion(audio, sr)

        # 3. 揺らぎ適用
        print("[3/4] 揺らぎエンジンを適用中...")
        audio = self.apply_fluctuations(audio, sr)

        # 4. ノイズ追加
        print("[4/4] 背景ノイズを追加中...")
        audio = self.apply_noise(audio, sr)

        # 出力
        output_path = Path(self.config.output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        sf.write(output_path, audio, sr)
        print(f"出力: {output_path}")

        return output_path


if __name__ == "__main__":
    # テスト実行
    import sys

    if len(sys.argv) < 2:
        print("使用法: python file_processor.py <input_audio_file>")
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = "audio/output/processed.wav"

    config = ProcessConfig(
        input_path=input_file,
        output_path=output_file,
        enable_fluctuation=True,
        enable_noise=True,
        noise_level=0.01
    )

    processor = AudioFileProcessor(config)
    result = processor.process()
    print(f"\n✅ 処理完了: {result}")
