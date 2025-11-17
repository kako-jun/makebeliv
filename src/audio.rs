use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Host, Stream, StreamConfig};
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

/// 音声入力マネージャー
pub struct AudioInput {
    host: Host,
    device: Device,
    config: StreamConfig,
}

impl AudioInput {
    /// デフォルトの入力デバイスで初期化
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .context("入力デバイスが見つかりません")?;

        let config = device
            .default_input_config()
            .context("入力デバイスの設定取得エラー")?
            .into();

        info!("入力デバイス: {}", device.name()?);

        Ok(Self {
            host,
            device,
            config,
        })
    }

    /// 音声ストリームを開始
    pub fn start_stream<F>(&self, mut callback: F) -> Result<Stream>
    where
        F: FnMut(&[f32]) + Send + 'static,
    {
        let stream = self.device.build_input_stream(
            &self.config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                callback(data);
            },
            |err| {
                warn!("音声入力エラー: {}", err);
            },
            None,
        )?;

        stream.play()?;
        info!("音声入力ストリーム開始");

        Ok(stream)
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate.0
    }

    pub fn channels(&self) -> u16 {
        self.config.channels
    }
}

/// 音声出力マネージャー
pub struct AudioOutput {
    host: Host,
    device: Device,
    config: StreamConfig,
}

impl AudioOutput {
    /// デフォルトの出力デバイスで初期化
    pub fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .context("出力デバイスが見つかりません")?;

        let config = device
            .default_output_config()
            .context("出力デバイスの設定取得エラー")?
            .into();

        info!("出力デバイス: {}", device.name()?);

        Ok(Self {
            host,
            device,
            config,
        })
    }

    /// 音声ストリームを開始
    pub fn start_stream<F>(&self, mut callback: F) -> Result<Stream>
    where
        F: FnMut(&mut [f32]) + Send + 'static,
    {
        let stream = self.device.build_output_stream(
            &self.config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                callback(data);
            },
            |err| {
                warn!("音声出力エラー: {}", err);
            },
            None,
        )?;

        stream.play()?;
        info!("音声出力ストリーム開始");

        Ok(stream)
    }

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate.0
    }
}

/// 音声バッファ（リングバッファ）
pub struct AudioBuffer {
    buffer: Arc<Mutex<Vec<f32>>>,
    capacity: usize,
}

impl AudioBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::with_capacity(capacity))),
            capacity,
        }
    }

    /// データを追加
    pub fn push(&self, data: &[f32]) {
        let mut buffer = self.buffer.lock().unwrap();

        // 容量チェック
        if buffer.len() + data.len() > self.capacity {
            // 古いデータを削除
            let overflow = buffer.len() + data.len() - self.capacity;
            buffer.drain(0..overflow);
        }

        buffer.extend_from_slice(data);
    }

    /// データを取得してクリア
    pub fn take(&self, len: usize) -> Vec<f32> {
        let mut buffer = self.buffer.lock().unwrap();

        if buffer.len() >= len {
            let data = buffer.drain(0..len).collect();
            data
        } else {
            // データ不足の場合は空
            Vec::new()
        }
    }

    /// バッファ内のデータ量
    pub fn len(&self) -> usize {
        self.buffer.lock().unwrap().len()
    }

    /// バッファをクリア
    pub fn clear(&self) {
        self.buffer.lock().unwrap().clear();
    }
}

/// 利用可能なデバイス一覧を表示
pub fn list_devices() -> Result<()> {
    let host = cpal::default_host();

    println!("入力デバイス:");
    for device in host.input_devices()? {
        println!("  - {}", device.name()?);
    }

    println!("\n出力デバイス:");
    for device in host.output_devices()? {
        println!("  - {}", device.name()?);
    }

    Ok(())
}
