//! File entropy: entropy of file size distribution across a crate.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{shannon_entropy, SourceFile};

pub struct FileEntropy;

impl FileEntropy {
    /// Bucket file sizes into categories.
    fn bucket_sizes(sizes: &[(PathBuf, usize)]) -> HashMap<String, usize> {
        let mut freq: HashMap<String, usize> = HashMap::new();
        for (_, size) in sizes {
            let bucket = match *size {
                0..=20 => "tiny".to_string(),
                21..=100 => "small".to_string(),
                101..=300 => "medium".to_string(),
                301..=700 => "large".to_string(),
                _ => "huge".to_string(),
            };
            *freq.entry(bucket).or_insert(0) += 1;
        }
        freq
    }

    /// Compute entropy of file size distribution.
    pub fn compute(files: &[SourceFile]) -> f64 {
        if files.is_empty() {
            return 0.0;
        }
        let sizes: Vec<(PathBuf, usize)> = files
            .iter()
            .map(|f| (f.path.clone(), f.content.lines().count()))
            .collect();
        let freq = Self::bucket_sizes(&sizes);
        shannon_entropy(&freq)
    }

    /// Get file sizes as (path, line_count) pairs.
    pub fn file_sizes(files: &[SourceFile]) -> Vec<(&Path, usize)> {
        files
            .iter()
            .map(|f| (f.path.as_path(), f.content.lines().count()))
            .collect()
    }
}
