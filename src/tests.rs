#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::*;
    use crate::{TokenEntropy, FunctionEntropy, NameEntropy, TestEntropy, FileEntropy};
    use crate::report::{EntropyReport, FileEntropyScore, Severity, assign_severity};

    // === Shannon entropy core ===

    #[test]
    fn test_shannon_entropy_uniform() {
        let mut freq = HashMap::new();
        freq.insert("a".into(), 1);
        freq.insert("b".into(), 1);
        freq.insert("c".into(), 1);
        freq.insert("d".into(), 1);
        // 4 equally likely items → entropy = log2(4) = 2.0
        let e = shannon_entropy(&freq);
        assert!((e - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_shannon_entropy_single() {
        let mut freq = HashMap::new();
        freq.insert("x".into(), 100);
        // All same → entropy = 0
        let e = shannon_entropy(&freq);
        assert!(e.abs() < 0.001);
    }

    #[test]
    fn test_shannon_entropy_empty() {
        let freq = HashMap::new();
        let e = shannon_entropy(&freq);
        assert!(e.abs() < 0.001);
    }

    #[test]
    fn test_shannon_entropy_from_slice() {
        let items = vec!["a", "b", "c", "d"];
        let e = shannon_entropy_from_slice(&items);
        assert!((e - 2.0).abs() < 0.001);
    }

    // === Token entropy ===

    #[test]
    fn test_token_entropy_simple() {
        let source = "fn main() { let x = 1; }";
        let e = TokenEntropy::compute(source);
        assert!(e > 0.0, "Should have positive entropy for non-trivial code");
    }

    #[test]
    fn test_token_entropy_repetitive_low() {
        // Very repetitive code should have lower entropy
        let repetitive = "x = 1; x = 1; x = 1; x = 1; x = 1;";
        let diverse = "fn foo() -> i32 { let alpha = 1; let beta = 2; let gamma = 3; let delta = 4; let epsilon = 5; }";
        let e_rep = TokenEntropy::compute(repetitive);
        let e_div = TokenEntropy::compute(diverse);
        assert!(e_div > e_rep, "Diverse code should have higher token entropy");
    }

    #[test]
    fn test_token_frequencies_count() {
        let source = "let x = 1; let y = 2;";
        let freq = TokenEntropy::token_frequencies(source);
        assert!(freq.get("let").copied().unwrap_or(0) >= 2);
        assert!(freq.get("=").copied().unwrap_or(0) >= 2);
    }

    #[test]
    fn test_tokenize_basic() {
        let source = "fn main() {}";
        let tokens = TokenEntropy::tokenize(source);
        assert!(tokens.contains(&"fn".to_string()));
        assert!(tokens.contains(&"main".to_string()));
    }

    // === Function entropy ===

    #[test]
    fn test_function_entropy_uniform_lengths() {
        // All functions same length → low entropy
        let uniform = "
fn foo() { let x = 1; }
fn bar() { let y = 2; }
fn baz() { let z = 3; }
";
        let diverse = "
fn tiny() {}
fn huge() { let x = 1; let y = 2; let z = 3; let w = 4; let v = 5;
    let a = 6; let b = 7; let c = 8; let d = 9; let e = 10;
    let f = 11; let g = 12; let h = 13; let i = 14; let j = 15;
    let k = 16; let l = 17; let m = 18; let n = 19; let o = 20;
    let p = 21; let q = 22; let r = 23; let s = 24; let t = 25;
    let u = 26; let v2 = 27; let w2 = 28; let x2 = 29; let y2 = 30;
    let z2 = 31; let a2 = 32; let b2 = 33; let c2 = 34; let d2 = 35;
    let e2 = 36; let f2 = 37; let g2 = 38; let h2 = 39; let i2 = 40;
    let j2 = 41; let k2 = 42; let l2 = 43; let m2 = 44; let n2 = 45;
    let o2 = 46; let p2 = 47; let q2 = 48; let r2 = 49; let s2 = 50;
    let t2 = 51; let u2 = 52; let v3 = 53; let w3 = 54; let x3 = 55;
    let y3 = 56; let z3 = 57; let a3 = 58; let b3 = 59; let c3 = 60; }
";
        let e_uniform = FunctionEntropy::compute(uniform);
        let e_diverse = FunctionEntropy::compute(diverse);
        assert!(e_diverse > e_uniform, "Mixed function lengths should have higher entropy");
    }

    #[test]
    fn test_function_lengths_extraction() {
        let source = "fn foo() {\n    let x = 1;\n}\nfn bar() {\n}\n";
        let lengths = FunctionEntropy::function_lengths(source);
        assert_eq!(lengths.len(), 2, "Should find 2 functions");
    }

    #[test]
    fn test_function_entropy_empty() {
        let e = FunctionEntropy::compute("// just a comment");
        assert!(e.abs() < 0.001, "No functions = zero entropy");
    }

    // === Name entropy ===

    #[test]
    fn test_name_entropy_consistent_low() {
        let consistent = "
fn get_user() {}
fn get_item() {}
fn get_data() {}
fn get_info() {}
";
        let random = "
fn calculate_quantum_flux() {}
fn zigzag_meridian_parse() {}
fn xyzzy_plugh_waldo() {}
fn crunchy_banana_split() {}
";
        let e_consistent = NameEntropy::compute(consistent);
        let e_random = NameEntropy::compute(random);
        // Consistent naming has lower entropy (fewer unique chars relative to alphabet)
        assert!(e_random > e_consistent, "Random names should have higher name entropy");
    }

    #[test]
    fn test_extract_identifiers_filters_keywords() {
        let source = "let mut x = 0; fn foo() {}";
        let ids = NameEntropy::extract_identifiers(source);
        assert!(!ids.contains(&"let".to_string()));
        assert!(!ids.contains(&"mut".to_string()));
        assert!(!ids.contains(&"fn".to_string()));
    }

    // === Test entropy ===

    #[test]
    fn test_test_entropy_diverse_names() {
        let diverse = r#"
#[test]
fn test_parse_returns_ok() {}
#[test]
fn test_serialize_handles_none() {}
#[test]
fn test_validate_rejects_empty() {}
"#;
        let uniform = r#"
#[test]
fn test_thing_1() {}
#[test]
fn test_thing_2() {}
#[test]
fn test_thing_3() {}
"#;
        let e_diverse = TestEntropy::compute(diverse);
        let e_uniform = TestEntropy::compute(uniform);
        assert!(e_diverse > e_uniform, "Diverse test names should have higher entropy");
    }

    #[test]
    fn test_extract_test_names() {
        let source = "#[test]\nfn test_foo() {} #[test]\nfn test_bar() {}";
        let names = TestEntropy::extract_test_names(source);
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"test_foo".to_string()));
    }

    #[test]
    fn test_entropy_no_tests() {
        let e = TestEntropy::compute("fn main() {}");
        assert!(e.abs() < 0.001, "No tests = zero entropy");
    }

    // === File entropy ===

    #[test]
    fn test_file_entropy_uniform_sizes() {
        let files: Vec<SourceFile> = (0..5)
            .map(|i| SourceFile {
                path: format!("/test/file{i}.rs").into(),
                content: "line\n".repeat(50),
            })
            .collect();
        let e = FileEntropy::compute(&files);
        assert!(e.abs() < 0.001, "Uniform file sizes = zero entropy");
    }

    #[test]
    fn test_file_entropy_mixed_sizes() {
        let files = vec![
            SourceFile { path: "/test/a.rs".into(), content: "x\n".repeat(5) },
            SourceFile { path: "/test/b.rs".into(), content: "x\n".repeat(500) },
            SourceFile { path: "/test/c.rs".into(), content: "x\n".repeat(50) },
        ];
        let e = FileEntropy::compute(&files);
        assert!(e > 0.0, "Mixed file sizes should have positive entropy");
    }

    // === Report and severity ===

    #[test]
    fn test_assign_severity_flags_top_10pct() {
        // Create 20 scores with clear distribution
        let mut scores: Vec<FileEntropyScore> = (0..20)
            .map(|i| FileEntropyScore {
                path: format!("/test/f{i}.rs").into(),
                token_entropy: i as f64 * 0.1,
                function_entropy: 0.0,
                name_entropy: 0.0,
                test_entropy: 0.0,
                composite_score: i as f64 * 0.1,
                severity: Severity::Ok,
            })
            .collect();

        assign_severity(&mut scores);

        let warnings = scores.iter().filter(|s| s.severity == Severity::Warning).count();
        let errors = scores.iter().filter(|s| s.severity == Severity::Error).count();
        // Should have some warnings and at least 1 error
        assert!(warnings > 0, "Should flag some warnings at 90th percentile");
        assert!(errors > 0, "Should flag errors at 95th percentile");
    }

    #[test]
    fn test_report_format() {
        let files = vec![
            SourceFile { path: "/test/simple.rs".into(), content: "fn main() {}".to_string() },
        ];
        let report = EntropyReport::from_files(&files).unwrap();
        let text = report.format_report();
        assert!(text.contains("Entropy Report"));
        assert!(text.contains("Files analyzed: 1"));
    }

    #[test]
    fn test_report_top_n() {
        let files: Vec<SourceFile> = (0..10)
            .map(|i| SourceFile {
                path: format!("/test/f{i}.rs").into(),
                content: format!("fn main() {{ let x{} = {} + {} + {} + {}; }}", i, i, i+1, i+2, i+3),
            })
            .collect();
        let report = EntropyReport::from_files(&files).unwrap();
        let top5 = report.top_n(5);
        assert_eq!(top5.len(), 5);
        // Should be sorted descending
        for i in 0..top5.len()-1 {
            assert!(top5[i].composite_score >= top5[i+1].composite_score);
        }
    }

    #[test]
    fn test_collect_rust_files() {
        let dir = std::env::current_dir().unwrap();
        let files = collect_rust_files(&dir).unwrap();
        // At minimum, our own source files should be found
        assert!(!files.is_empty(), "Should find .rs files in the project");
        assert!(files.iter().all(|f| f.path.extension().map_or(false, |e| e == "rs")));
    }

    #[test]
    fn test_source_file_from_path() {
        let path = std::path::Path::new(file!());
        let sf = SourceFile::from_path(path).unwrap();
        assert!(sf.content.contains("test_source_file_from_path"));
    }
}
