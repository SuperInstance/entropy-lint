//! # entropy-lint
//!
//! Information entropy analysis for code quality.
//! Hypothesis: high-entropy codebases have more bugs.
//!
//! Measures Shannon entropy across multiple dimensions:
//! - Token distribution per file
//! - Function length distribution
//! - Identifier name randomness
//! - Test name diversity
//! - File size distribution across a crate

use std::collections::HashMap;
use std::path::{Path, PathBuf};

mod token_entropy;
mod function_entropy;
mod name_entropy;
mod test_entropy;
mod file_entropy;
mod report;
#[cfg(test)]
mod tests;

pub use token_entropy::TokenEntropy;
pub use function_entropy::FunctionEntropy;
pub use name_entropy::NameEntropy;
pub use test_entropy::TestEntropy;
pub use file_entropy::FileEntropy;
pub use report::{EntropyReport, FileEntropyScore, Severity};

/// Compute Shannon entropy from a frequency map.
/// H = -Σ p_i * log2(p_i)
pub fn shannon_entropy(freq: &HashMap<String, usize>) -> f64 {
    let total: usize = freq.values().sum();
    if total == 0 {
        return 0.0;
    }
    let total_f = total as f64;
    let mut entropy = 0.0;
    for &count in freq.values() {
        if count == 0 {
            continue;
        }
        let p = count as f64 / total_f;
        entropy -= p * p.log2();
    }
    entropy
}

/// Compute Shannon entropy from a slice of values (using generic string representation).
pub fn shannon_entropy_from_slice(items: &[impl AsRef<str>]) -> f64 {
    let mut freq: HashMap<String, usize> = HashMap::new();
    for item in items {
        *freq.entry(item.as_ref().to_string()).or_insert(0) += 1;
    }
    shannon_entropy(&freq)
}

/// A single source file ready for entropy analysis.
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
}

impl SourceFile {
    pub fn from_path(path: &Path) -> std::io::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(Self {
            path: path.to_path_buf(),
            content,
        })
    }
}

/// Collect all Rust source files from a directory.
pub fn collect_rust_files(dir: &Path) -> std::io::Result<Vec<SourceFile>> {
    let mut files = Vec::new();
    if !dir.is_dir() {
        return Ok(files);
    }
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "rs") {
            // Skip target directories
            if path.components().any(|c| c.as_os_str() == "target") {
                continue;
            }
            if let Ok(sf) = SourceFile::from_path(path) {
                files.push(sf);
            }
        }
    }
    Ok(files)
}
