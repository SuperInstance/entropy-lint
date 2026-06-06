//! Token entropy: Shannon entropy of source code token distribution per file.

use std::collections::HashMap;

use crate::shannon_entropy;

/// Compute token-level Shannon entropy for a source file.
pub struct TokenEntropy;

impl TokenEntropy {
    /// Tokenize source code into broad token categories.
    pub fn tokenize(source: &str) -> Vec<String> {
        let pattern = r#"(?:[a-zA-Z_][a-zA-Z0-9_]*|0x[0-9a-fA-F_]+|[0-9][0-9_]*(?:\.[0-9_]+)?|"[^"]*"|'[^']*'|//[^\n]*|/\*[\s\S]*?\*/|->|=>|::|\.\.\.|\.\.|[+\-*/%=<>!&|^~?@#]{1,3}|[{}()\[\];:.,])"#;
        let re = regex::Regex::new(pattern).unwrap();

        re.find_iter(source)
            .map(|m| m.as_str().to_string())
            .collect()
    }

    /// Compute token frequency map.
    pub fn token_frequencies(source: &str) -> HashMap<String, usize> {
        let tokens = Self::tokenize(source);
        let mut freq: HashMap<String, usize> = HashMap::new();
        for token in tokens {
            *freq.entry(token).or_insert(0) += 1;
        }
        freq
    }

    /// Compute Shannon entropy of token distribution for a file.
    pub fn compute(source: &str) -> f64 {
        let freq = Self::token_frequencies(source);
        shannon_entropy(&freq)
    }
}
