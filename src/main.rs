use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::{Child, Command};
use tracing::{info, warn};

mod audio;
mod client;

use client::VoiceConversionClient;

#[derive(Parser)]
#[command(name = "makebeliv")]
#[command(about = "Real-time voice conversion with natural fluctuation", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Setup Python environment using uv
    Setup {
        /// Skip confirmation prompts
        #[arg(short, long)]
        yes: bool,
    },

    /// Start API server
    Server {
        /// Host address
        #[arg(long, default_value = "0.0.0.0")]
        host: String,

        /// Port number
        #[arg(long, default_value = "8000")]
        port: u16,
    },

    /// Process audio file (development mode)
    Process {
        /// Input audio file
        #[arg(short, long)]
        input: PathBuf,

        /// Output audio file (default: audio/output/processed.wav)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Voice model to use
        #[arg(short, long, default_value = "default")]
        model: String,

        /// Background noise type (cafe, street, room)
        #[arg(short, long, default_value = "cafe")]
        noise: String,

        /// Pitch shift in semitones (e.g., +3)
        #[arg(short, long, default_value = "0")]
        pitch: i32,

        /// Use API server (default: direct Python execution)
        #[arg(long)]
        use_api: bool,

        /// API server URL
        #[arg(long, default_value = "http://localhost:8000")]
        api_url: String,
    },

    /// Real-time voice conversion
    Monitor {
        /// Voice model to use
        #[arg(short, long, default_value = "default")]
        model: String,

        /// Background noise type
        #[arg(short, long, default_value = "cafe")]
        noise: String,

        /// Pitch shift in semitones
        #[arg(short, long, default_value = "0")]
        pitch: i32,

        /// API server URL
        #[arg(long, default_value = "http://localhost:8000")]
        api_url: String,
    },

    /// List audio devices
    ListDevices,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Setup { yes } => setup_environment(yes),
        Commands::Server { host, port } => start_server(host, port),
        Commands::Process {
            input,
            output,
            model,
            noise,
            pitch,
            use_api,
            api_url,
        } => {
            if use_api {
                process_audio_via_api(input, output, model, noise, pitch, api_url).await
            } else {
                process_audio_direct(input, output, model, noise, pitch)
            }
        }
        Commands::Monitor {
            model,
            noise,
            pitch,
            api_url,
        } => monitor_realtime(model, noise, pitch, api_url).await,
        Commands::ListDevices => {
            audio::list_devices()?;
            Ok(())
        }
    }
}

fn setup_environment(skip_confirm: bool) -> Result<()> {
    info!("🔧 Makebeliv環境セットアップ");

    // 1. uvがインストールされているか確認
    info!("uvの確認中...");
    let uv_check = Command::new("uv").arg("--version").output();

    let uv_available = match uv_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            info!("  ✓ uv がインストール済み: {}", version.trim());
            true
        }
        _ => {
            warn!("  ✗ uv が見つかりません");
            false
        }
    };

    if !uv_available {
        println!("\n📦 uvをインストールする必要があります:");
        println!("  curl -LsSf https://astral.sh/uv/install.sh | sh");
        println!("\nまたは:");
        println!("  cargo install uv");
        println!("\nインストール後、再度このコマンドを実行してください。");
        return Ok(());
    }

    // 2. 仮想環境の作成
    if !skip_confirm {
        println!("\n仮想環境を作成しますか？ (y/N)");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("セットアップをキャンセルしました。");
            return Ok(());
        }
    }

    info!("仮想環境を作成中...");
    let venv_status = Command::new("uv")
        .args(["venv", ".venv"])
        .status()
        .context("仮想環境の作成に失敗")?;

    if !venv_status.success() {
        anyhow::bail!("仮想環境の作成に失敗しました");
    }
    info!("  ✓ 仮想環境を作成しました: .venv");

    // 3. 依存関係のインストール
    info!("依存関係をインストール中...");

    // PyTorchはCUDA対応版を明示的にインストール
    println!("\n🔍 GPU (CUDA) を使用しますか？");
    println!("  RTX 3050などのNVIDIA GPUをお持ちの場合は推奨します。");
    println!("  使用する場合: y, CPUのみの場合: N");

    let mut gpu_input = String::new();
    std::io::stdin().read_line(&mut gpu_input)?;
    let use_gpu = gpu_input.trim().eq_ignore_ascii_case("y");

    if use_gpu {
        info!("CUDA対応PyTorchをインストール中...");
        let torch_install = Command::new("uv")
            .args([
                "pip",
                "install",
                "torch",
                "torchaudio",
                "--index-url",
                "https://download.pytorch.org/whl/cu118",
            ])
            .status()
            .context("PyTorchのインストールに失敗")?;

        if !torch_install.success() {
            warn!("CUDA版PyTorchのインストールに失敗。CPU版を試します...");
        } else {
            info!("  ✓ CUDA対応PyTorchをインストールしました");
        }
    }

    // その他の依存関係をインストール
    let deps_status = Command::new("uv")
        .args(["pip", "install", "-r", "requirements.txt"])
        .status()
        .context("依存関係のインストールに失敗")?;

    if !deps_status.success() {
        anyhow::bail!("依存関係のインストールに失敗しました");
    }
    info!("  ✓ 依存関係をインストールしました");

    // 4. セットアップ完了
    println!("\n✅ セットアップが完了しました！");
    println!("\n次のステップ:");
    println!("  1. APIサーバーを起動:");
    println!("     makebeliv server");
    println!("  2. テスト用の音声ファイルを audio/input/ に配置");
    println!("  3. 以下のコマンドでテスト実行:");
    println!("     makebeliv process -i audio/input/test.wav --use-api");
    println!("\n仮想環境の有効化:");
    println!("  source .venv/bin/activate  # Linux/macOS");
    println!("  .venv\\Scripts\\activate     # Windows");

    Ok(())
}

fn start_server(host: String, port: u16) -> Result<()> {
    info!("🚀 APIサーバーを起動中...");
    info!("   アドレス: {}:{}", host, port);

    // uvxを使ってAPIサーバーを起動
    let status = Command::new("uv")
        .args([
            "run",
            "uvicorn",
            "python.api_server:app",
            "--host",
            &host,
            "--port",
            &port.to_string(),
            "--reload",
        ])
        .status()
        .context("APIサーバー起動エラー")?;

    if !status.success() {
        anyhow::bail!("APIサーバーの起動に失敗しました");
    }

    Ok(())
}

fn process_audio_direct(
    input: PathBuf,
    output: Option<PathBuf>,
    model: String,
    noise: String,
    pitch: i32,
) -> Result<()> {
    info!("🎙️ 音声ファイル処理モード（直接実行）");

    if !input.exists() {
        anyhow::bail!("入力ファイルが見つかりません: {}", input.display());
    }

    let output_path = output.unwrap_or_else(|| PathBuf::from("audio/output/processed.wav"));

    info!("設定:");
    info!("  入力: {}", input.display());
    info!("  出力: {}", output_path.display());
    info!("  モデル: {}", model);
    info!("  ノイズ: {}", noise);
    info!("  ピッチ: {:+} semitones", pitch);

    // Pythonスクリプトを実行
    let status = Command::new("uv")
        .args(["run", "python", "python/file_processor.py"])
        .arg(input.to_str().unwrap())
        .status()
        .context("Pythonスクリプトの実行に失敗")?;

    if !status.success() {
        anyhow::bail!("音声処理に失敗しました");
    }

    info!("✅ 処理完了: {}", output_path.display());

    Ok(())
}

async fn process_audio_via_api(
    input: PathBuf,
    output: Option<PathBuf>,
    model: String,
    noise: String,
    pitch: i32,
    api_url: String,
) -> Result<()> {
    info!("🎙️ 音声ファイル処理モード（API経由）");

    if !input.exists() {
        anyhow::bail!("入力ファイルが見つかりません: {}", input.display());
    }

    let output_path = output.unwrap_or_else(|| PathBuf::from("audio/output/processed.wav"));

    info!("設定:");
    info!("  入力: {}", input.display());
    info!("  出力: {}", output_path.display());
    info!("  モデル: {}", model);
    info!("  ノイズ: {}", noise);
    info!("  ピッチ: {:+} semitones", pitch);
    info!("  APIサーバー: {}", api_url);

    // APIクライアント作成
    let client = VoiceConversionClient::new(api_url);

    // サーバー状態確認
    match client.check_status().await {
        Ok(status) => {
            info!("✓ サーバー接続成功: {:?}", status);
        }
        Err(e) => {
            warn!("⚠ サーバー接続エラー: {}", e);
            println!("\nAPIサーバーが起動していない可能性があります。");
            println!("以下のコマンドでサーバーを起動してください:");
            println!("  makebeliv server");
            return Err(e);
        }
    }

    // 音声変換
    client
        .convert_file(&input, &output_path, &model, pitch, &noise, 0.02)
        .await?;

    info!("✅ 処理完了: {}", output_path.display());

    Ok(())
}

async fn monitor_realtime(
    model: String,
    noise: String,
    pitch: i32,
    api_url: String,
) -> Result<()> {
    info!("🎧 リアルタイム音声変換モード");
    info!("設定:");
    info!("  モデル: {}", model);
    info!("  ノイズ: {}", noise);
    info!("  ピッチ: {:+} semitones", pitch);
    info!("  APIサーバー: {}", api_url);

    // APIクライアント作成
    let client = VoiceConversionClient::new(api_url);

    // サーバー状態確認
    match client.check_status().await {
        Ok(status) => {
            info!("✓ サーバー接続成功: {:?}", status);
        }
        Err(e) => {
            warn!("⚠ サーバー接続エラー: {}", e);
            println!("\nAPIサーバーが起動していない可能性があります。");
            println!("以下のコマンドでサーバーを起動してください:");
            println!("  makebeliv server");
            return Err(e);
        }
    }

    println!("\n⚠️  リアルタイムモードは現在開発中です。");
    println!("代わりに以下のコマンドでファイル処理をお試しください:");
    println!("  makebeliv process -i audio/input/test.wav --use-api");

    // TODO: リアルタイム処理実装
    // 1. マイク入力開始
    // 2. チャンク単位で変換
    // 3. スピーカー/仮想マイクに出力

    Ok(())
}
