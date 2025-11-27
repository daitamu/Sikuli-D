//! Special REPL commands
//! 特殊REPLコマンド

/// Special commands for REPL control
/// REPL制御用の特殊コマンド
#[derive(Debug, Clone, PartialEq)]
pub enum SpecialCommand {
    /// Show help
    /// ヘルプを表示
    Help,

    /// Exit REPL
    /// REPLを終了
    Exit,

    /// Quit REPL (alias for Exit)
    /// REPLを終了 (Exitのエイリアス)
    Quit,

    /// Clear screen
    /// 画面をクリア
    Clear,

    /// Show command history
    /// コマンド履歴を表示
    History,

    /// Show defined variables
    /// 定義済み変数を表示
    Vars,

    /// Reset Python context
    /// Pythonコンテキストをリセット
    Reset,
}

impl SpecialCommand {
    /// Parse a line to check if it's a special command
    /// 行を解析して特殊コマンドか確認
    pub fn parse(line: &str) -> Option<Self> {
        let trimmed = line.trim();

        match trimmed {
            ":help" | ":h" | ":?" => Some(Self::Help),
            ":exit" => Some(Self::Exit),
            ":quit" | ":q" => Some(Self::Quit),
            ":clear" | ":cls" => Some(Self::Clear),
            ":history" | ":hist" => Some(Self::History),
            ":vars" | ":variables" => Some(Self::Vars),
            ":reset" => Some(Self::Reset),
            _ => None,
        }
    }

    /// Get command description
    /// コマンドの説明を取得
    #[allow(dead_code)]
    pub fn description(&self) -> &str {
        match self {
            Self::Help => "Show help message / ヘルプメッセージを表示",
            Self::Exit | Self::Quit => "Exit REPL / REPLを終了",
            Self::Clear => "Clear screen / 画面をクリア",
            Self::History => "Show command history / コマンド履歴を表示",
            Self::Vars => "Show defined variables / 定義済み変数を表示",
            Self::Reset => "Reset Python context / Pythonコンテキストをリセット",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_help() {
        assert_eq!(SpecialCommand::parse(":help"), Some(SpecialCommand::Help));
        assert_eq!(SpecialCommand::parse(":h"), Some(SpecialCommand::Help));
        assert_eq!(SpecialCommand::parse(":?"), Some(SpecialCommand::Help));
    }

    #[test]
    fn test_parse_exit() {
        assert_eq!(SpecialCommand::parse(":exit"), Some(SpecialCommand::Exit));
        assert_eq!(SpecialCommand::parse(":quit"), Some(SpecialCommand::Quit));
        assert_eq!(SpecialCommand::parse(":q"), Some(SpecialCommand::Quit));
    }

    #[test]
    fn test_parse_clear() {
        assert_eq!(SpecialCommand::parse(":clear"), Some(SpecialCommand::Clear));
        assert_eq!(SpecialCommand::parse(":cls"), Some(SpecialCommand::Clear));
    }

    #[test]
    fn test_parse_invalid() {
        assert_eq!(SpecialCommand::parse(":invalid"), None);
        assert_eq!(SpecialCommand::parse("print('hello')"), None);
    }

    #[test]
    fn test_parse_whitespace() {
        assert_eq!(SpecialCommand::parse("  :help  "), Some(SpecialCommand::Help));
    }
}
