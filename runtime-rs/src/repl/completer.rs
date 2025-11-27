//! Tab completion for SikuliX REPL
//! SikuliX REPL用のタブ補完

use rustyline::completion::{Completer, Pair};
use rustyline::Context;

/// SikuliX API completer
/// SikuliX API補完器
pub struct SikulixCompleter {
    /// List of SikuliX API functions and classes
    api_items: Vec<String>,
}

impl SikulixCompleter {
    /// Create a new completer
    /// 新しい補完器を作成
    pub fn new() -> Self {
        let api_items = vec![
            // Image finding
            "find",
            "findAll",
            "wait",
            "waitVanish",
            "exists",

            // Mouse actions
            "click",
            "doubleClick",
            "rightClick",
            "hover",
            "drag",
            "dragDrop",
            "wheel",
            "mouseMove",
            "mouseDown",
            "mouseUp",

            // Keyboard actions
            "type",
            "paste",
            "hotkey",
            "keyDown",
            "keyUp",

            // Classes
            "Screen",
            "Region",
            "Match",
            "Pattern",
            "Location",
            "Key",

            // Screen operations
            "capture",
            "selectRegion",

            // Settings
            "Settings",

            // Special functions
            "sleep",
            "popup",
            "input",
            "popAsk",
            "popError",

            // Observation
            "observe",
            "onAppear",
            "onVanish",
            "onChange",

            // OCR
            "text",
            "textRead",

            // Utilities
            "getImagePath",
            "setImagePath",
            "addImagePath",
            "removeImagePath",
            "getBundlePath",
            "setBundlePath",

            // App control
            "openApp",
            "closeApp",
            "switchApp",
            "App",

            // Special REPL commands
            ":help",
            ":exit",
            ":quit",
            ":clear",
            ":history",
            ":vars",
            ":reset",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        Self { api_items }
    }

    /// Add custom completion items
    /// カスタム補完項目を追加
    pub fn add_item(&mut self, item: String) {
        if !self.api_items.contains(&item) {
            self.api_items.push(item);
        }
    }

    /// Get completions for a partial word
    /// 部分的な単語の補完を取得
    fn get_completions(&self, word: &str) -> Vec<String> {
        let word_lower = word.to_lowercase();

        self.api_items
            .iter()
            .filter(|item| item.to_lowercase().starts_with(&word_lower))
            .cloned()
            .collect()
    }
}

impl Default for SikulixCompleter {
    fn default() -> Self {
        Self::new()
    }
}

impl Completer for SikulixCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        // Extract the word to complete
        let start = line[..pos]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_' && c != ':')
            .map(|i| i + 1)
            .unwrap_or(0);

        let word = &line[start..pos];

        if word.is_empty() {
            return Ok((pos, vec![]));
        }

        // Get completions
        let completions = self.get_completions(word);

        // Convert to Pairs
        let pairs: Vec<Pair> = completions
            .into_iter()
            .map(|c| Pair {
                display: c.clone(),
                replacement: c,
            })
            .collect();

        Ok((start, pairs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_completion() {
        let completer = SikulixCompleter::new();

        let completions = completer.get_completions("fin");
        assert!(completions.contains(&"find".to_string()));
        assert!(completions.contains(&"findAll".to_string()));

        let completions = completer.get_completions("cli");
        assert!(completions.contains(&"click".to_string()));

        let completions = completer.get_completions(":he");
        assert!(completions.contains(&":help".to_string()));
    }

    #[test]
    fn test_case_insensitive() {
        let completer = SikulixCompleter::new();

        let completions = completer.get_completions("FIN");
        assert!(completions.contains(&"find".to_string()));
    }

    #[test]
    fn test_no_match() {
        let completer = SikulixCompleter::new();

        let completions = completer.get_completions("xyz");
        assert!(completions.is_empty());
    }
}
