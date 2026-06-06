//! Entropy report: aggregates all entropy scores per file and flags high-entropy files.

use std::path::{Path, PathBuf};

use crate::{
    collect_rust_files, FileEntropy, FunctionEntropy, NameEntropy, SourceFile, TestEntropy,
    TokenEntropy,
};

/// Severity level for high-entropy files.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Below 90th percentile — no issue.
    Ok,
    /// 90th–95th percentile — warning.
    Warning,
    /// Above 95th percentile — error.
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Ok => write!(f, "OK"),
            Severity::Warning => write!(f, "WARNING"),
            Severity::Error => write!(f, "ERROR"),
        }
    }
}

/// Per-file entropy scores.
#[derive(Debug, Clone)]
pub struct FileEntropyScore {
    pub path: PathBuf,
    pub token_entropy: f64,
    pub function_entropy: f64,
    pub name_entropy: f64,
    pub test_entropy: f64,
    /// Composite score (weighted sum).
    pub composite_score: f64,
    pub severity: Severity,
}

/// Full entropy report for a crate.
#[derive(Debug, Clone)]
pub struct EntropyReport {
    pub files: Vec<FileEntropyScore>,
    pub file_size_entropy: f64,
    pub total_files: usize,
}

impl EntropyReport {
    /// Analyze a directory and produce an entropy report.
    pub fn analyze(dir: &Path) -> std::io::Result<Self> {
        let files = collect_rust_files(dir)?;
        Self::from_files(&files)
    }

    /// Produce a report from a list of source files.
    pub fn from_files(files: &[SourceFile]) -> std::io::Result<Self> {
        let file_size_entropy = FileEntropy::compute(files);

        let mut scores: Vec<FileEntropyScore> = files
            .iter()
            .map(|f| {
                let token_e = TokenEntropy::compute(&f.content);
                let func_e = FunctionEntropy::compute(&f.content);
                let name_e = NameEntropy::compute(&f.content);
                let test_e = TestEntropy::compute(&f.content);

                // Composite: weighted sum
                // Token entropy is the primary signal, function/name/test are secondary
                let composite = token_e * 0.40 + func_e * 0.25 + name_e * 0.25 + test_e * 0.10;

                FileEntropyScore {
                    path: f.path.clone(),
                    token_entropy: token_e,
                    function_entropy: func_e,
                    name_entropy: name_e,
                    test_entropy: test_e,
                    composite_score: composite,
                    severity: Severity::Ok,
                }
            })
            .collect();

        // Assign severity based on percentiles of composite score
        assign_severity(&mut scores);

        Ok(EntropyReport {
            files: scores,
            file_size_entropy,
            total_files: files.len(),
        })
    }

    /// Get files flagged as needing review (Warning or Error).
    pub fn flagged_files(&self) -> Vec<&FileEntropyScore> {
        self.files
            .iter()
            .filter(|f| f.severity != Severity::Ok)
            .collect()
    }

    /// Get top N files by composite entropy score.
    pub fn top_n(&self, n: usize) -> Vec<&FileEntropyScore> {
        let mut sorted: Vec<&FileEntropyScore> = self.files.iter().collect();
        sorted.sort_by(|a, b| b.composite_score.partial_cmp(&a.composite_score).unwrap());
        sorted.into_iter().take(n).collect()
    }

    /// Format as a human-readable report.
    pub fn format_report(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "=== Entropy Report ===\nFiles analyzed: {}\nFile size entropy: {:.4}\n\n",
            self.total_files, self.file_size_entropy
        ));

        let flagged = self.flagged_files();
        if flagged.is_empty() {
            out.push_str("No high-entropy files detected.\n");
        } else {
            out.push_str(&format!("{} files flagged for review:\n\n", flagged.len()));
            for f in &flagged {
                out.push_str(&format!(
                    "[{}] {}\n  Token: {:.4}  Function: {:.4}  Name: {:.4}  Test: {:.4}\n  Composite: {:.4}\n\n",
                    f.severity,
                    f.path.display(),
                    f.token_entropy,
                    f.function_entropy,
                    f.name_entropy,
                    f.test_entropy,
                    f.composite_score,
                ));
            }
        }

        out.push_str("\n--- Top 10 files by composite entropy ---\n");
        for (i, f) in self.top_n(10).iter().enumerate() {
            out.push_str(&format!(
                "{:2}. [{:.4}] {} (token={:.4} func={:.4} name={:.4})\n",
                i + 1,
                f.composite_score,
                f.path.display(),
                f.token_entropy,
                f.function_entropy,
                f.name_entropy,
            ));
        }

        out
    }
}

/// Assign severity based on composite score percentiles.
pub(crate) fn assign_severity(scores: &mut [FileEntropyScore]) {
    if scores.len() < 2 {
        return;
    }

    let mut composite_scores: Vec<f64> = scores.iter().map(|s| s.composite_score).collect();
    composite_scores.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let p90_idx = ((composite_scores.len() as f64) * 0.90).ceil() as usize;
    let p95_idx = ((composite_scores.len() as f64) * 0.95).ceil() as usize;

    let p90 = composite_scores[p90_idx.min(composite_scores.len() - 1)];
    let p95 = composite_scores[p95_idx.min(composite_scores.len() - 1)];

    for score in scores.iter_mut() {
        score.severity = if score.composite_score >= p95 {
            Severity::Error
        } else if score.composite_score >= p90 {
            Severity::Warning
        } else {
            Severity::Ok
        };
    }
}
