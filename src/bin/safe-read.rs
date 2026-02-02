use context_tlp::redact;
use context_tlp::tlp;
use std::env;
use std::fs;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: safe-read <file-path>");
        return ExitCode::from(1);
    }

    let file_path = &args[1];

    // Check TLP classification — refuse RED files
    if let Some(c) = tlp::classify_file(file_path) {
        if c.level == tlp::Tlp::Red {
            eprintln!("TLP:RED — this file is blocked. safe-read only handles AMBER files.");
            return ExitCode::from(1);
        }
    }

    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cannot read {}: {}", file_path, e);
            return ExitCode::from(1);
        }
    };

    let tlp_redacted = redact::redact_tlp_sections(&content);
    let (output, secrets_found) = redact::redact_secrets(&tlp_redacted);

    if secrets_found {
        eprintln!("⚠ WARNING: secret(s) detected and redacted in {}. Consider rotating the exposed key(s).", file_path);
    }

    print!("{}", output);
    ExitCode::SUCCESS
}
