//! Test entropy: entropy of test name patterns.
//!
//! Diverse test names = good coverage signal (low entropy = all tests named similarly,
//! which may indicate copy-paste testing or missing edge cases).

use std::collections::HashMap;

use crate::shannon_entropy;

pub struct TestEntropy;

impl TestEntropy {
    /// Extract test function names from source code.
    pub fn extract_test_names(source: &str) -> Vec<String> {
        let re = regex::Regex::new(
            r"#\[test\](?:\s*#\[.*\])*\s*(?:pub\s+)?(?:async\s+)?fn\s+(\w+)"
        ).unwrap();

        re.captures_iter(source)
            .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    /// Extract "words" from test names by splitting on underscores.
    fn test_name_words(test_names: &[String]) -> Vec<String> {
        let mut words = Vec::new();
        for name in test_names {
            for word in name.split('_') {
                if !word.is_empty() {
                    words.push(word.to_lowercase());
                }
            }
        }
        words
    }

    /// Compute entropy of test name word distribution.
    /// Higher = more diverse vocabulary in test names = better coverage signal.
    pub fn compute(source: &str) -> f64 {
        let test_names = Self::extract_test_names(source);
        if test_names.is_empty() {
            return 0.0;
        }
        let words = Self::test_name_words(&test_names);
        if words.is_empty() {
            return 0.0;
        }

        let mut freq: HashMap<String, usize> = HashMap::new();
        for word in &words {
            *freq.entry(word.clone()).or_insert(0) += 1;
        }
        shannon_entropy(&freq)
    }

    /// Return number of tests found.
    pub fn test_count(source: &str) -> usize {
        Self::extract_test_names(source).len()
    }
}
