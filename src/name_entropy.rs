//! Name entropy: entropy of identifier names.
//!
//! Random, inconsistent naming = high entropy (potential confusion, bugs).
//! Repetitive, consistent naming = low entropy (easier to understand).

use std::collections::HashMap;

pub struct NameEntropy;

impl NameEntropy {
    /// Extract all identifiers from source code.
    pub fn extract_identifiers(source: &str) -> Vec<String> {
        let re = regex::Regex::new(r"\b([a-z_][a-z0-9_]*)\b").unwrap();
        let keywords = [
            "fn", "let", "mut", "if", "else", "match", "loop", "while", "for", "in",
            "return", "break", "continue", "struct", "enum", "impl", "trait", "type",
            "where", "use", "mod", "pub", "self", "super", "crate", "as", "async",
            "await", "move", "ref", "static", "const", "unsafe", "extern", "true",
            "false", "dyn", "from", "into", "new", "default", "some", "none", "ok",
            "err", "and", "or", "not", "the", "to", "i", "j", "k", "a", "b", "n",
            "x", "y", "z", "s", "e", "v", "p", "m", "c", "d", "r", "t",
        ];

        re.find_iter(source)
            .map(|m| m.as_str().to_string())
            .filter(|id| !keywords.contains(&id.as_str()) && id.len() > 1)
            .collect()
    }

    /// Compute per-character entropy of identifier names.
    /// This measures the "randomness" of character choices in names.
    pub fn compute(source: &str) -> f64 {
        let identifiers = Self::extract_identifiers(source);
        if identifiers.is_empty() {
            return 0.0;
        }

        // Collect all characters across all identifiers
        let mut char_freq: HashMap<char, usize> = HashMap::new();
        let mut total_chars = 0;
        for id in &identifiers {
            for ch in id.chars() {
                *char_freq.entry(ch).or_insert(0) += 1;
                total_chars += 1;
            }
        }

        if total_chars == 0 {
            return 0.0;
        }

        let total_f = total_chars as f64;
        let mut entropy = 0.0;
        for &count in char_freq.values() {
            let p = count as f64 / total_f;
            entropy -= p * p.log2();
        }

        // Normalize by log2 of unique character count for comparability
        let max_entropy = (char_freq.len() as f64).log2();
        if max_entropy == 0.0 {
            return 0.0;
        }
        entropy / max_entropy
    }
}
