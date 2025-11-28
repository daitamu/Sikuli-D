//! REPL module - Interactive Python REPL with SikuliX API
//! REPLモジュール - SikuliX API付きインタラクティブPython REPL

use anyhow::{Context, Result};
use rustyline::error::ReadlineError;
use rustyline::history::DefaultHistory;
use rustyline::Editor;
use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;

mod completer;
mod special_commands;
#[cfg(test)]
mod tests;

pub use completer::SikulixCompleter;
use special_commands::SpecialCommand;

/// REPL configuration
/// REPL設定
pub struct ReplConfig {
    /// Python interpreter path
    /// Pythonインタプリタのパス
    pub python_path: Option<String>,

    /// Enable command history
    /// コマンド履歴を有効化
    pub enable_history: bool,

    /// Startup script to execute
    /// 実行する起動スクリプト
    pub startup_script: Option<PathBuf>,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            python_path: None,
            enable_history: true,
            startup_script: None,
        }
    }
}

/// Interactive REPL session
/// インタラクティブREPLセッション
pub struct Repl {
    /// Readline editor with tab completion
    /// タブ補完付きReadlineエディタ
    editor: Editor<SikulixCompleter, DefaultHistory>,

    /// Command history file path
    history_file: PathBuf,

    /// Python process (for interactive mode)
    python_process: Option<PythonRepl>,

    /// Configuration
    config: ReplConfig,
}

impl Repl {
    /// Create a new REPL instance
    /// 新しいREPLインスタンスを作成
    pub fn new(config: ReplConfig) -> Result<Self> {
        let completer = SikulixCompleter::new();
        let mut editor = Editor::new().context("Failed to create readline editor")?;
        editor.set_helper(Some(completer));

        // Set up history
        let history_file = Self::get_history_file();

        if config.enable_history && history_file.exists() {
            if let Err(e) = editor.load_history(&history_file) {
                log::warn!("Failed to load history: {}", e);
            }
        }

        Ok(Self {
            editor,
            history_file,
            python_process: None,
            config,
        })
    }

    /// Start the REPL session
    /// REPLセッションを開始
    pub fn start(&mut self) -> Result<()> {
        self.print_banner();

        // Find Python interpreter
        let python_cmd = if let Some(ref path) = self.config.python_path {
            crate::python::PythonCommand::new(path)
        } else {
            crate::python::find_python()?
        };

        log::info!(
            "Using Python: {} {:?}",
            python_cmd.program,
            python_cmd.extra_args
        );

        // Start Python REPL process
        self.python_process = Some(PythonRepl::start(&python_cmd)?);

        // Execute startup script if provided
        if let Some(script) = self.config.startup_script.clone() {
            self.execute_file(&script)?;
        }

        // Main REPL loop
        self.run_loop()
    }

    /// Run the main REPL loop
    /// メインREPLループを実行
    fn run_loop(&mut self) -> Result<()> {
        let mut line_buffer = String::new();
        let mut in_multiline = false;

        loop {
            // Determine prompt
            let prompt = if in_multiline { "...    " } else { "sikulix> " };

            // Read line
            match self.editor.readline(prompt) {
                Ok(line) => {
                    // Check for special commands (only if not in multiline)
                    if !in_multiline {
                        if let Some(cmd) = SpecialCommand::parse(&line) {
                            match self.handle_special_command(cmd) {
                                Ok(true) => continue, // Continue REPL
                                Ok(false) => break,   // Exit REPL
                                Err(e) => {
                                    eprintln!("Error: {}", e);
                                    continue;
                                }
                            }
                        }
                    }

                    // Add to history
                    if !line.trim().is_empty() {
                        let _ = self.editor.add_history_entry(&line);
                    }

                    // Handle multiline input
                    if self.is_incomplete(&line) || in_multiline {
                        line_buffer.push_str(&line);
                        line_buffer.push('\n');
                        in_multiline = !self.is_complete(&line_buffer);
                        continue;
                    }

                    // Complete line
                    if !line_buffer.is_empty() {
                        line_buffer.push_str(&line);
                        line_buffer.push('\n');
                    } else {
                        line_buffer = line.clone() + "\n";
                    }

                    // Execute Python code
                    if let Some(ref mut python) = self.python_process {
                        if let Err(e) = python.execute(&line_buffer) {
                            eprintln!("Execution error: {}", e);
                        }
                    }

                    // Clear buffer
                    line_buffer.clear();
                    in_multiline = false;
                }
                Err(ReadlineError::Interrupted) => {
                    // Ctrl+C
                    println!("^C");
                    line_buffer.clear();
                    in_multiline = false;
                    continue;
                }
                Err(ReadlineError::Eof) => {
                    // Ctrl+D
                    println!("exit");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }

        // Save history
        if self.config.enable_history {
            if let Err(e) = self.editor.save_history(&self.history_file) {
                log::warn!("Failed to save history: {}", e);
            }
        }

        Ok(())
    }

    /// Handle special REPL command
    /// 特殊REPLコマンドを処理
    fn handle_special_command(&mut self, cmd: SpecialCommand) -> Result<bool> {
        match cmd {
            SpecialCommand::Help => {
                self.print_help();
                Ok(true)
            }
            SpecialCommand::Exit | SpecialCommand::Quit => Ok(false),
            SpecialCommand::Clear => {
                // Clear screen
                print!("\x1B[2J\x1B[1;1H");
                Ok(true)
            }
            SpecialCommand::History => {
                self.show_history();
                Ok(true)
            }
            SpecialCommand::Vars => {
                if let Some(ref mut python) = self.python_process {
                    python.execute("print(dir())")?;
                }
                Ok(true)
            }
            SpecialCommand::Reset => {
                println!("Resetting Python context...");
                // Restart Python process
                let python_cmd = if let Some(ref path) = self.config.python_path {
                    crate::python::PythonCommand::new(path)
                } else {
                    crate::python::find_python()?
                };
                self.python_process = Some(PythonRepl::start(&python_cmd)?);
                Ok(true)
            }
        }
    }

    /// Execute a Python file
    /// Pythonファイルを実行
    fn execute_file(&mut self, path: &PathBuf) -> Result<()> {
        let content = std::fs::read_to_string(path).context("Failed to read startup script")?;

        if let Some(ref mut python) = self.python_process {
            python.execute(&content)?;
        }

        Ok(())
    }

    /// Check if line is incomplete (needs more input)
    /// 行が不完全（さらに入力が必要）か確認
    fn is_incomplete(&self, line: &str) -> bool {
        let trimmed = line.trim();

        // Check for continuation indicators
        trimmed.ends_with(':') || trimmed.ends_with('\\') || self.has_unclosed_brackets(trimmed)
    }

    /// Check if code block is complete
    /// コードブロックが完全か確認
    fn is_complete(&self, code: &str) -> bool {
        // Check if all brackets are closed
        !self.has_unclosed_brackets(code)
    }

    /// Check for unclosed brackets
    /// 閉じられていない括弧を確認
    fn has_unclosed_brackets(&self, code: &str) -> bool {
        let mut paren = 0;
        let mut bracket = 0;
        let mut brace = 0;
        let mut in_string = false;
        let mut string_char = ' ';
        let mut escaped = false;

        for ch in code.chars() {
            if escaped {
                escaped = false;
                continue;
            }

            if ch == '\\' {
                escaped = true;
                continue;
            }

            if in_string {
                if ch == string_char {
                    in_string = false;
                }
                continue;
            }

            match ch {
                '"' | '\'' => {
                    in_string = true;
                    string_char = ch;
                }
                '(' => paren += 1,
                ')' => paren -= 1,
                '[' => bracket += 1,
                ']' => bracket -= 1,
                '{' => brace += 1,
                '}' => brace -= 1,
                _ => {}
            }
        }

        paren != 0 || bracket != 0 || brace != 0 || in_string
    }

    /// Print welcome banner
    /// ウェルカムバナーを表示
    fn print_banner(&self) {
        println!("SikuliX REPL v{}", env!("CARGO_PKG_VERSION"));
        println!("Interactive Python shell with SikuliX API loaded");
        println!("Type ':help' for help, ':exit' or ':quit' to exit");
        println!();
    }

    /// Print help message
    /// ヘルプメッセージを表示
    fn print_help(&self) {
        println!("Special REPL Commands:");
        println!("  :help      - Show this help message");
        println!("  :exit      - Exit REPL");
        println!("  :quit      - Exit REPL");
        println!("  :clear     - Clear screen");
        println!("  :history   - Show command history");
        println!("  :vars      - Show defined variables");
        println!("  :reset     - Reset Python context");
        println!();
        println!("SikuliX API Examples:");
        println!("  >>> from sikulix import *");
        println!("  >>> m = find('button.png')");
        println!("  >>> click(m)");
        println!("  >>> type('Hello World')");
        println!();
    }

    /// Show command history
    /// コマンド履歴を表示
    fn show_history(&self) {
        println!("Command History:");
        for (i, entry) in self.editor.history().iter().enumerate() {
            println!("  {:4}: {}", i + 1, entry);
        }
    }

    /// Get history file path
    /// 履歴ファイルのパスを取得
    fn get_history_file() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".sikulix_history")
    }
}

/// Python REPL process wrapper
/// Python REPLプロセスラッパー
struct PythonRepl {
    stdin: std::process::ChildStdin,
    _stdout_thread: thread::JoinHandle<()>,
    _stderr_thread: thread::JoinHandle<()>,
}

impl PythonRepl {
    /// Start Python REPL process
    /// Python REPLプロセスを開始
    fn start(python_cmd: &crate::python::PythonCommand) -> Result<Self> {
        // Get SikuliX API path
        let api_path =
            crate::python::get_sikulid_api_path().context("Failed to locate sikulid_api")?;

        // Create startup script
        let startup_script = Self::create_startup_script();

        // Start Python in interactive mode
        let mut cmd = Command::new(&python_cmd.program);
        for arg in &python_cmd.extra_args {
            cmd.arg(arg);
        }
        let mut child = cmd
            .arg("-i")  // Interactive mode
            .arg("-u")  // Unbuffered
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("PYTHONPATH", api_path)
            .spawn()
            .context("Failed to start Python process")?;

        let stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");
        let stderr = child.stderr.take().expect("stderr");

        // Spawn output handler threads
        let stdout_thread = thread::spawn(move || {
            let reader = std::io::BufReader::new(stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    // Filter out Python prompts
                    if !line.starts_with(">>>") && !line.starts_with("...") {
                        println!("{}", line);
                    }
                }
            }
        });

        let stderr_thread = thread::spawn(move || {
            let reader = std::io::BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    eprintln!("{}", line);
                }
            }
        });

        let mut repl = Self {
            stdin,
            _stdout_thread: stdout_thread,
            _stderr_thread: stderr_thread,
        };

        // Execute startup script
        repl.execute(&startup_script)?;

        Ok(repl)
    }

    /// Execute Python code
    /// Pythonコードを実行
    fn execute(&mut self, code: &str) -> Result<()> {
        self.stdin
            .write_all(code.as_bytes())
            .context("Failed to write to Python stdin")?;
        self.stdin.flush().context("Failed to flush stdin")?;

        // Small delay to allow output to appear
        thread::sleep(std::time::Duration::from_millis(50));

        Ok(())
    }

    /// Create startup script to initialize SikuliX API
    /// SikuliX APIを初期化する起動スクリプトを作成
    fn create_startup_script() -> String {
        r#"
import sys
import os

# Import SikuliX API
try:
    from sikulix_api import *
    print("SikuliX API loaded successfully")
except ImportError as e:
    print(f"Warning: Failed to load SikuliX API: {e}")
    print("You can still use standard Python features")

# Set up useful variables
__sikulix_repl__ = True

# Helper function for help
def sikulix_help():
    """Show SikuliX REPL help"""
    help_text = """
SikuliX REPL - Quick Reference
================================

Image Finding:
  find(image)              - Find image on screen
  findAll(image)           - Find all matches
  wait(image, timeout=3)   - Wait for image to appear
  exists(image, timeout=0) - Check if image exists (non-throwing)

Mouse Actions:
  click(x, y)              - Click at coordinates
  doubleClick(x, y)        - Double click
  rightClick(x, y)         - Right click
  hover(x, y)              - Move mouse
  drag(x1, y1, x2, y2)     - Drag

Keyboard Actions:
  type(text)               - Type text
  paste(text)              - Paste via clipboard
  hotkey(*keys)            - Press key combination

Screen:
  Screen()                 - Get screen object
  Region(x, y, w, h)       - Create region

For more information: help(function_name)
"""
    print(help_text)

# Suppress default Python prompt (we handle it in Rust)
sys.ps1 = ""
sys.ps2 = ""

"#
        .to_string()
    }
}

/// Start interactive REPL
/// インタラクティブREPLを開始
pub fn start_repl(config: ReplConfig) -> Result<()> {
    let mut repl = Repl::new(config)?;
    repl.start()
}
