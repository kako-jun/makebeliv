"""
FastAPI音声変換サーバー

Rustクライアントから音声データを受け取り、
RVC変換 + 揺らぎエンジンを適用して返すHTTPサーバー。

遅延を最小化するため：
- 非同期処理
- バイナリストリーミング
- GPU推論の並列化
"""

import io
import numpy as np
import soundfile as sf
from fastapi import FastAPI, UploadFile, File, HTTPException
from fastapi.responses import StreamingResponse
from pydantic import BaseModel
from typing import Optional
import logging
import time

from rvc_engine import RVCEngine, RVCConfig, RVCRealtimeEngine
from fluctuation import FluctuationEngine, FluctuationConfig, add_background_noise

# ロギング設定
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# FastAPIアプリケーション
app = FastAPI(
    title="Makebeliv Voice Conversion API",
    description="リアルタイムボイスチェンジャー API",
    version="0.1.0"
)


# リクエスト/レスポンスモデル
class ConvertRequest(BaseModel):
    """音声変換リクエスト"""
    model: str = "default"
    pitch_shift: int = 0
    noise_type: str = "cafe"
    noise_level: float = 0.02
    enable_fluctuation: bool = True


class ServerStatus(BaseModel):
    """サーバーステータス"""
    status: str
    device: str
    models_loaded: int
    uptime_seconds: float


# グローバル状態
class ServerState:
    """サーバーの状態管理"""

    def __init__(self):
        self.start_time = time.time()
        self.rvc_engines = {}  # モデル名 -> RVCEngine
        self.fluctuation_engines = {}  # セッションID -> FluctuationEngine
        self.device = "cuda" if __import__("torch").cuda.is_available() else "cpu"

        logger.info(f"サーバー初期化: device={self.device}")

    def get_or_create_rvc_engine(self, model_name: str, pitch_shift: int = 0) -> RVCEngine:
        """RVCエンジンを取得または作成"""
        key = f"{model_name}_{pitch_shift}"

        if key not in self.rvc_engines:
            config = RVCConfig(
                model_path=f"models/{model_name}/model.pth",
                f0_up_key=pitch_shift,
                device=self.device
            )
            self.rvc_engines[key] = RVCEngine(config)
            logger.info(f"RVCエンジン作成: {key}")

        return self.rvc_engines[key]

    def get_or_create_fluctuation_engine(self, session_id: str) -> FluctuationEngine:
        """揺らぎエンジンを取得または作成"""
        if session_id not in self.fluctuation_engines:
            config = FluctuationConfig()
            self.fluctuation_engines[session_id] = FluctuationEngine(config)
            logger.info(f"揺らぎエンジン作成: {session_id}")

        return self.fluctuation_engines[session_id]


# グローバルインスタンス
state = ServerState()


# エンドポイント
@app.get("/")
async def root():
    """ルート - サーバー情報"""
    return {
        "name": "Makebeliv Voice Conversion API",
        "version": "0.1.0",
        "status": "running"
    }


@app.get("/status", response_model=ServerStatus)
async def get_status():
    """サーバーステータスを取得"""
    return ServerStatus(
        status="running",
        device=state.device,
        models_loaded=len(state.rvc_engines),
        uptime_seconds=time.time() - state.start_time
    )


@app.post("/convert")
async def convert_audio(
    audio: UploadFile = File(...),
    model: str = "default",
    pitch_shift: int = 0,
    noise_type: str = "cafe",
    noise_level: float = 0.02,
    enable_fluctuation: bool = True,
    session_id: str = "default"
):
    """音声変換API

    Args:
        audio: 音声ファイル（WAV, MP3など）
        model: 使用するモデル名
        pitch_shift: ピッチシフト（半音単位）
        noise_type: ノイズの種類
        noise_level: ノイズレベル（0-1）
        enable_fluctuation: 揺らぎエンジンを有効化
        session_id: セッションID（揺らぎの連続性のため）

    Returns:
        変換後の音声（WAV形式）
    """
    start_time = time.time()

    try:
        # 音声データを読み込み
        audio_bytes = await audio.read()
        audio_data, sr = sf.read(io.BytesIO(audio_bytes))

        # モノラル化
        if len(audio_data.shape) > 1:
            audio_data = np.mean(audio_data, axis=1)

        logger.info(f"入力音声: sr={sr}Hz, len={len(audio_data)/sr:.2f}秒")

        # 1. RVC変換
        rvc_engine = state.get_or_create_rvc_engine(model, pitch_shift)
        converted = rvc_engine.convert(audio_data, sr)

        # 2. 揺らぎエンジン適用
        if enable_fluctuation:
            fluct_engine = state.get_or_create_fluctuation_engine(session_id)
            converted = fluct_engine.apply_volume_fluctuation(converted)

        # 3. ノイズ追加
        if noise_level > 0:
            converted = add_background_noise(
                converted,
                noise_type=noise_type,
                noise_level=noise_level,
                sr=sr
            )

        # 4. 出力
        output_buffer = io.BytesIO()
        sf.write(output_buffer, converted, sr, format='WAV')
        output_buffer.seek(0)

        elapsed = time.time() - start_time
        logger.info(f"変換完了: {elapsed*1000:.1f}ms")

        return StreamingResponse(
            output_buffer,
            media_type="audio/wav",
            headers={
                "X-Processing-Time-Ms": str(int(elapsed * 1000)),
                "X-Audio-Length-Seconds": str(len(converted) / sr)
            }
        )

    except Exception as e:
        logger.error(f"変換エラー: {e}", exc_info=True)
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/convert-chunk")
async def convert_audio_chunk(
    audio: UploadFile = File(...),
    model: str = "default",
    pitch_shift: int = 0,
    enable_fluctuation: bool = True,
    session_id: str = "default"
):
    """音声チャンク変換API（リアルタイム用）

    小さな音声チャンクを高速に変換します。
    遅延を最小化するため、ノイズ追加などは省略されています。

    Args:
        audio: 音声チャンク（WAV形式、100-200ms推奨）
        model: 使用するモデル名
        pitch_shift: ピッチシフト（半音単位）
        enable_fluctuation: 揺らぎエンジンを有効化
        session_id: セッションID

    Returns:
        変換後の音声チャンク
    """
    start_time = time.time()

    try:
        # 音声データを読み込み
        audio_bytes = await audio.read()
        audio_data, sr = sf.read(io.BytesIO(audio_bytes))

        # モノラル化
        if len(audio_data.shape) > 1:
            audio_data = np.mean(audio_data, axis=1)

        # RVC変換（チャンクモード）
        # TODO: RVCRealtimeEngineを使用
        rvc_engine = state.get_or_create_rvc_engine(model, pitch_shift)
        converted = rvc_engine.convert(audio_data, sr)

        # 揺らぎ（軽量版）
        if enable_fluctuation:
            fluct_engine = state.get_or_create_fluctuation_engine(session_id)
            converted = fluct_engine.apply_volume_fluctuation(converted)

        # 出力
        output_buffer = io.BytesIO()
        sf.write(output_buffer, converted, sr, format='WAV')
        output_buffer.seek(0)

        elapsed = time.time() - start_time
        logger.debug(f"チャンク変換: {elapsed*1000:.1f}ms")

        return StreamingResponse(
            output_buffer,
            media_type="audio/wav",
            headers={
                "X-Processing-Time-Ms": str(int(elapsed * 1000))
            }
        )

    except Exception as e:
        logger.error(f"チャンク変換エラー: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/reset-session")
async def reset_session(session_id: str):
    """セッションをリセット

    揺らぎエンジンの状態をクリアします。
    """
    if session_id in state.fluctuation_engines:
        state.fluctuation_engines[session_id].reset()
        logger.info(f"セッションリセット: {session_id}")
        return {"status": "reset", "session_id": session_id}
    else:
        return {"status": "not_found", "session_id": session_id}


@app.on_event("startup")
async def startup_event():
    """サーバー起動時の処理"""
    logger.info("🚀 Makebeliv API サーバー起動")
    logger.info(f"   Device: {state.device}")


@app.on_event("shutdown")
async def shutdown_event():
    """サーバーシャットダウン時の処理"""
    logger.info("🛑 Makebeliv API サーバー停止")


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(
        "api_server:app",
        host="0.0.0.0",
        port=8000,
        reload=True,
        log_level="info"
    )
