//! Function entropy: entropy of function length distribution.
//!
//! A file with uniform function lengths has low entropy.
//! A file with some very long and some very short functions has high entropy
//! (mixed abstraction levels → bug-prone).

use std::collections::HashMap;

use crate::shannon_entropy;

pub struct FunctionEntropy;

impl FunctionEntropy {
    /// Extract function bodies and return their line counts.
    /// Uses a simple brace-counting approach.
    pub fn function_lengths(source: &str) -> Vec<usize> {
        let re = regex::Regex::new(
            r"(?m)^\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)"
        ).unwrap();

        let lines: Vec<&str> = source.lines().collect();
        let mut lengths = Vec::new();
        let _line_count = lines.len();

        for mat in re.find_iter(source) {
            let start_byte = mat.start();
            // Find the line number
            let mut byte_pos = 0;
            let mut _start_line = 0;
            for (i, line) in lines.iter().enumerate() {
                byte_pos += line.len() + 1;
                if byte_pos > start_byte {
                    _start_line = i;
                    break;
                }
            }

            // Find opening brace and count brace depth
            let rest = &source[start_byte..];
            let brace_start = match rest.find('{') {
                Some(pos) => start_byte + pos,
                None => continue,
            };

            let mut depth = 0i32;
            let mut in_string = false;
            let mut in_char = false;
            let mut in_line_comment = false;
            let mut in_block_comment = false;
            let mut pos = brace_start;
            let bytes = source.as_bytes();

            while pos < source.len() {
                let ch = bytes[pos];
                let next_ch = if pos + 1 < source.len() { bytes[pos + 1] } else { 0 };

                if in_line_comment {
                    if ch == b'\n' { in_line_comment = false; }
                    pos += 1; continue;
                }
                if in_block_comment {
                    if ch == b'*' && next_ch == b'/' { in_block_comment = false; pos += 2; continue; }
                    pos += 1; continue;
                }
                if in_string {
                    if ch == b'\\' { pos += 2; continue; }
                    if ch == b'"' { in_string = false; }
                    pos += 1; continue;
                }
                if in_char {
                    if ch == b'\\' { pos += 2; continue; }
                    if ch == b'\'' { in_char = false; }
                    pos += 1; continue;
                }

                match ch {
                    b'/' if next_ch == b'/' => { in_line_comment = true; pos += 2; continue; }
                    b'/' if next_ch == b'*' => { in_block_comment = true; pos += 2; continue; }
                    b'"' => { in_string = true; pos += 1; continue; }
                    b'\'' => { in_char = true; pos += 1; continue; }
                    b'{' => { depth += 1; }
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            // End of function body
                            let mut func_line_count = 0;
                            let mut b = brace_start;
                            while b <= pos && b < source.len() {
                                if source.as_bytes()[b] == b'\n' {
                                    func_line_count += 1;
                                }
                                b += 1;
                            }
                            lengths.push(func_line_count.max(1));
                            break;
                        }
                    }
                    _ => {}
                }
                pos += 1;
            }
        }

        lengths
    }

    /// Bucket function lengths into categories for entropy calculation.
    fn bucket_lengths(lengths: &[usize]) -> HashMap<String, usize> {
        let mut freq: HashMap<String, usize> = HashMap::new();
        for &len in lengths {
            let bucket = match len {
                0..=5 => "tiny".to_string(),
                6..=15 => "short".to_string(),
                16..=30 => "medium".to_string(),
                31..=60 => "long".to_string(),
                _ => "huge".to_string(),
            };
            *freq.entry(bucket).or_insert(0) += 1;
        }
        freq
    }

    /// Compute function length entropy for a source file.
    pub fn compute(source: &str) -> f64 {
        let lengths = Self::function_lengths(source);
        if lengths.is_empty() {
            return 0.0;
        }
        let freq = Self::bucket_lengths(&lengths);
        shannon_entropy(&freq)
    }
}
