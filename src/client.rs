use anyhow::{Context, Result};
use bytes::Bytes;
use reqwest::multipart;
use std::path::Path;
use tracing::{debug, info};

/// 音声変換APIクライアント
pub struct VoiceConversionClient {
    client: reqwest::Client,
    base_url: String,
}

impl VoiceConversionClient {
    /// 新しいクライアントを作成
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
        }
    }

    /// サーバーのステータスを確認
    pub async fn check_status(&self) -> Result<serde_json::Value> {
        let url = format!("{}/status", self.base_url);
        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("ステータス取得エラー")?;

        let status = response.json().await.context("JSON解析エラー")?;
        Ok(status)
    }

    /// 音声ファイルを変換
    pub async fn convert_file(
        &self,
        input_path: &Path,
        output_path: &Path,
        model: &str,
        pitch_shift: i32,
        noise_type: &str,
        noise_level: f32,
    ) -> Result<()> {
        info!("音声変換リクエスト送信...");

        // ファイルを読み込み
        let audio_bytes = std::fs::read(input_path).context("入力ファイル読み込みエラー")?;

        // マルチパートフォームを構築
        let form = multipart::Form::new()
            .part(
                "audio",
                multipart::Part::bytes(audio_bytes)
                    .file_name(input_path.file_name().unwrap().to_string_lossy().to_string())
                    .mime_str("audio/wav")?,
            )
            .text("model", model.to_string())
            .text("pitch_shift", pitch_shift.to_string())
            .text("noise_type", noise_type.to_string())
            .text("noise_level", noise_level.to_string());

        // リクエスト送信
        let url = format!("{}/convert", self.base_url);
        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .context("変換リクエストエラー")?;

        // 処理時間を取得
        if let Some(processing_time) = response.headers().get("X-Processing-Time-Ms") {
            info!(
                "サーバー処理時間: {}ms",
                processing_time.to_str().unwrap_or("?")
            );
        }

        // レスポンスを保存
        let audio_data = response.bytes().await.context("レスポンス読み込みエラー")?;

        std::fs::write(output_path, audio_data).context("出力ファイル書き込みエラー")?;

        info!("✓ 変換完了: {}", output_path.display());

        Ok(())
    }

    /// 音声チャンクを変換（リアルタイム用）
    pub async fn convert_chunk(
        &self,
        audio_data: &[u8],
        model: &str,
        pitch_shift: i32,
        session_id: &str,
    ) -> Result<Bytes> {
        debug!("チャンク変換リクエスト: {} bytes", audio_data.len());

        let form = multipart::Form::new()
            .part(
                "audio",
                multipart::Part::bytes(audio_data.to_vec())
                    .file_name("chunk.wav")
                    .mime_str("audio/wav")?,
            )
            .text("model", model.to_string())
            .text("pitch_shift", pitch_shift.to_string())
            .text("session_id", session_id.to_string());

        let url = format!("{}/convert-chunk", self.base_url);
        let response = self
            .client
            .post(&url)
            .multipart(form)
            .send()
            .await
            .context("チャンク変換リクエストエラー")?;

        let converted_data = response.bytes().await.context("チャンク読み込みエラー")?;

        Ok(converted_data)
    }

    /// セッションをリセット
    pub async fn reset_session(&self, session_id: &str) -> Result<()> {
        let url = format!("{}/reset-session?session_id={}", self.base_url, session_id);
        self.client
            .post(&url)
            .send()
            .await
            .context("セッションリセットエラー")?;

        info!("セッションリセット完了: {}", session_id);
        Ok(())
    }
}
