//! SikuliX Runtime - Headless script execution
//! SikuliX ランタイム - ヘッドレススクリプト実行

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod bundle;
mod converter;
mod hotkey;
mod python;
mod repl;
mod runner;

/// SikuliX Runtime CLI
/// SikuliX ランタイム コマンドラインインターフェース
#[derive(Parser)]
#[command(name = "sikulix")]
#[command(author = "Sikuli-D Team")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "SikuliX headless runtime for script execution")]
#[command(
    long_about = "SikuliX headless runtime - Run SikuliX Python scripts without GUI.\nSikuliX ヘッドレスランタイム - GUIなしでSikuliX Pythonスクリプトを実行"
)]
struct Cli {
    /// Enable verbose logging / 詳細ログを有効化
    #[arg(short, long)]
    verbose: bool,

    /// Log level (error, warn, info, debug, trace)
    #[arg(long, default_value = "info")]
    log_level: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a SikuliX script / SikuliXスクリプトを実行
    Run {
        /// Path to script file (.py) or bundle (.sikuli)
        /// スクリプトファイル (.py) またはバンドル (.sikuli) のパス
        script: PathBuf,

        /// Arguments to pass to the script / スクリプトに渡す引数
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,

        /// Working directory / 作業ディレクトリ
        #[arg(short = 'd', long)]
        workdir: Option<PathBuf>,

        /// Timeout in seconds (0 = no timeout) / タイムアウト秒数 (0 = 無制限)
        #[arg(short, long, default_value = "0")]
        timeout: u64,
    },

    /// Find an image on screen / 画面上で画像を検索
    Find {
        /// Path to template image / テンプレート画像のパス
        image: PathBuf,

        /// Minimum similarity (0.0-1.0) / 最小類似度
        #[arg(short, long, default_value = "0.7")]
        similarity: f64,

        /// Find all matches / 全てのマッチを検索
        #[arg(short, long)]
        all: bool,
    },

    /// Capture screen / 画面をキャプチャ
    Capture {
        /// Output file path / 出力ファイルパス
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Region to capture (x,y,w,h) / キャプチャ領域
        #[arg(short, long)]
        region: Option<String>,
    },

    /// Start interactive REPL / インタラクティブREPLを開始
    Repl {
        /// Python interpreter path / Pythonインタプリタのパス
        #[arg(long)]
        python: Option<String>,

        /// Disable command history / コマンド履歴を無効化
        #[arg(long)]
        no_history: bool,

        /// Startup script to execute / 実行する起動スクリプト
        #[arg(long)]
        startup: Option<PathBuf>,
    },

    /// Show system information / システム情報を表示
    Info,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { &cli.log_level };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level)).init();

    log::info!("SikuliX Runtime v{}", env!("CARGO_PKG_VERSION"));

    match cli.command {
        Commands::Run {
            script,
            args,
            workdir,
            timeout,
        } => {
            runner::run_script(&script, &args, workdir.as_deref(), timeout)?;
        }
        Commands::Find {
            image,
            similarity,
            all,
        } => {
            runner::find_image(&image, similarity, all)?;
        }
        Commands::Capture { output, region } => {
            runner::capture_screen(output.as_deref(), region.as_deref())?;
        }
        Commands::Repl {
            python,
            no_history,
            startup,
        } => {
            let config = repl::ReplConfig {
                python_path: python,
                enable_history: !no_history,
                startup_script: startup,
            };
            repl::start_repl(config)?;
        }
        Commands::Info => {
            runner::show_info()?;
        }
    }

    Ok(())
}
