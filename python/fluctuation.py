"""
揺らぎエンジン - 人間らしい自然な変化を音声に加える

このモジュールは音声に以下の揺らぎを加えます：
- ピッチの微細な変動
- 音量のランダムな変化
- EQ（周波数特性）の揺らぎ
- 時間的な連続性を保った変化
"""

import numpy as np
from typing import Optional
from dataclasses import dataclass


@dataclass
class FluctuationConfig:
    """揺らぎエンジンの設定"""

    pitch_variation: float = 0.05  # ±5%のピッチ変動
    volume_variation: float = 0.03  # ±3%の音量変動
    eq_variation: float = 0.02  # ±2%のEQ変動
    temporal_smoothness: float = 0.8  # 時間的な滑らかさ（0-1、高いほど滑らか）
    seed: Optional[int] = None


class FluctuationEngine:
    """揺らぎエンジンの実装"""

    def __init__(self, config: Optional[FluctuationConfig] = None):
        self.config = config or FluctuationConfig()
        self.rng = np.random.default_rng(self.config.seed)

        # 前回の揺らぎパラメータ（連続性のため）
        self.prev_pitch_factor = 1.0
        self.prev_volume_factor = 1.0
        self.prev_eq_factors = np.ones(5)  # 5バンドEQ

    def _smooth_transition(self, current: float, target: float, smoothness: float) -> float:
        """前回の値から滑らかに遷移する"""
        return current * smoothness + target * (1 - smoothness)

    def generate_pitch_factor(self) -> float:
        """ピッチの揺らぎ係数を生成（時間的に連続）"""
        target = 1.0 + self.rng.normal(0, self.config.pitch_variation)
        self.prev_pitch_factor = self._smooth_transition(
            self.prev_pitch_factor,
            target,
            self.config.temporal_smoothness
        )
        return self.prev_pitch_factor

    def generate_volume_factor(self) -> float:
        """音量の揺らぎ係数を生成（時間的に連続）"""
        target = 1.0 + self.rng.normal(0, self.config.volume_variation)
        self.prev_volume_factor = self._smooth_transition(
            self.prev_volume_factor,
            target,
            self.config.temporal_smoothness
        )
        return self.prev_volume_factor

    def apply_volume_fluctuation(self, audio: np.ndarray) -> np.ndarray:
        """音声に音量の揺らぎを適用"""
        factor = self.generate_volume_factor()
        return audio * factor

    def apply_pitch_fluctuation(self, audio: np.ndarray, sr: int) -> np.ndarray:
        """音声にピッチの揺らぎを適用

        注：実際のピッチシフトはlibrosaなどを使用
        ここでは簡易的な実装のプレースホルダー
        """
        # TODO: librosa.effects.pitch_shift を使った実装
        # 現時点ではパススルー
        return audio

    def reset(self):
        """揺らぎの状態をリセット"""
        self.prev_pitch_factor = 1.0
        self.prev_volume_factor = 1.0
        self.prev_eq_factors = np.ones(5)


def add_background_noise(
    audio: np.ndarray,
    noise_type: str = "cafe",
    noise_level: float = 0.02,
    sr: int = 16000
) -> np.ndarray:
    """背景ノイズを追加して不自然さを隠す

    Args:
        audio: 入力音声
        noise_type: ノイズの種類（"cafe", "street", "room"など）
        noise_level: ノイズレベル（0-1）
        sr: サンプリングレート

    Returns:
        ノイズが加わった音声
    """
    # TODO: 実際のノイズファイルを読み込む実装
    # 現時点では白色ノイズを生成
    rng = np.random.default_rng()
    noise = rng.normal(0, noise_level, audio.shape)
    return audio + noise


if __name__ == "__main__":
    # テストコード
    config = FluctuationConfig(pitch_variation=0.05, volume_variation=0.03)
    engine = FluctuationEngine(config)

    # 10回の連続した揺らぎを生成
    print("連続したピッチ揺らぎ係数:")
    for i in range(10):
        pitch = engine.generate_pitch_factor()
        volume = engine.generate_volume_factor()
        print(f"  {i}: pitch={pitch:.4f}, volume={volume:.4f}")
