use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: entropy-lint <path-to-crate>");
        eprintln!("Analyzes a Rust crate for code entropy.");
        std::process::exit(1);
    }

    let dir = Path::new(&args[1]);
    if !dir.is_dir() {
        eprintln!("Error: {} is not a directory", args[1]);
        std::process::exit(1);
    }

    match entropy_lint::EntropyReport::analyze(dir) {
        Ok(report) => {
            println!("{}", report.format_report());
            let errors = report.files.iter().filter(|f| f.severity == entropy_lint::Severity::Error).count();
            if errors > 0 {
                std::process::exit(2);
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
