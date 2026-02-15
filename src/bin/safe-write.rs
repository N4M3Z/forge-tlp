use context_tlp::redact;
use context_tlp::tlp;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  safe-write edit <file> --old <old-string> --new <new-string>");
    eprintln!("  safe-write write <file>          (reads new content from stdin)");
    eprintln!();
    eprintln!("Edit: replace exactly one occurrence of old-string with new-string.");
    eprintln!("Write: overwrite entire file, preserving #tlp/red blocks and secrets.");
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        print_usage();
        return ExitCode::from(1);
    }

    let mode = &args[1];
    let file_path = &args[2];

    // TLP gate: refuse RED files
    if let Some(c) = tlp::classify_file(file_path) {
        if c.level == tlp::Tlp::Red {
            eprintln!("TLP:RED — safe-write refuses RED files.");
            return ExitCode::from(1);
        }
    }

    match mode.as_str() {
        "edit" => cmd_edit(file_path, &args[3..]),
        "write" => cmd_write(file_path),
        _ => {
            eprintln!("Unknown mode: {mode}");
            print_usage();
            ExitCode::from(1)
        }
    }
}

// ─── Edit mode ───
//
// Operates on the ORIGINAL file content (not the safe-read view).
// The AI constructs old_string from safe-read output, but non-redacted text
// is identical between the original and safe-read view. So old_string matches
// the original directly — as long as it doesn't span redacted content.

fn cmd_edit(file_path: &str, args: &[String]) -> ExitCode {
    let (mut old_string, mut new_string) = (None, None);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--old" => {
                i += 1;
                old_string = args.get(i).map(String::as_str);
            }
            "--new" => {
                i += 1;
                new_string = args.get(i).map(String::as_str);
            }
            other => {
                eprintln!("Unknown flag: {other}");
                return ExitCode::from(1);
            }
        }
        i += 1;
    }

    let Some(old) = old_string else {
        eprintln!("--old is required for edit mode");
        return ExitCode::from(1);
    };
    let Some(new) = new_string else {
        eprintln!("--new is required for edit mode");
        return ExitCode::from(1);
    };

    // Guard: reject edits targeting redacted placeholders
    if old.contains(redact::REDACTED_MARKER) || old.contains(redact::SECRET_MARKER) {
        eprintln!(
            "old_string contains redaction markers — cannot edit hidden content. \
             Use safe-read to view what's visible, then edit only visible text."
        );
        return ExitCode::from(1);
    }

    let content = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cannot read {file_path}: {e}");
            return ExitCode::from(1);
        }
    };

    let count = content.matches(old).count();
    match count {
        0 => {
            eprintln!("old_string not found in {file_path}");
            ExitCode::from(1)
        }
        1 => {
            let result = content.replacen(old, new, 1);
            if let Err(e) = fs::write(file_path, &result) {
                eprintln!("Cannot write {file_path}: {e}");
                return ExitCode::from(1);
            }
            println!("{file_path}");
            ExitCode::SUCCESS
        }
        n => {
            eprintln!("old_string found {n} times in {file_path} — must be unique");
            ExitCode::from(1)
        }
    }
}

// ─── Write mode ───
//
// The AI sends content based on the safe-read view, which has [REDACTED] and
// [SECRET REDACTED] markers where hidden content was stripped. We must restore
// those markers with the original hidden content before writing to disk.
//
// Pipeline:
//   1. Read original from disk
//   2. Extract hidden chunks (TLP blocks, inline TLP, secrets)
//   3. Read new content from stdin
//   4. Replace markers in new content with original hidden chunks
//   5. Write the merged result to disk

fn cmd_write(file_path: &str) -> ExitCode {
    let mut new_content = String::new();
    if let Err(e) = io::stdin().read_to_string(&mut new_content) {
        eprintln!("Cannot read stdin: {e}");
        return ExitCode::from(1);
    }

    if new_content.is_empty() {
        eprintln!("Refusing to write empty content to {file_path}");
        return ExitCode::from(1);
    }

    // Read original to extract hidden content
    let original = match fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            // New file — no hidden content to preserve
            if let Err(e) = fs::write(file_path, &new_content) {
                eprintln!("Cannot write {file_path}: {e}");
                return ExitCode::from(1);
            }
            println!("{file_path}");
            return ExitCode::SUCCESS;
        }
        Err(e) => {
            eprintln!("Cannot read {file_path}: {e}");
            return ExitCode::from(1);
        }
    };

    // Extract hidden content from original (same pipeline order as safe-read)
    let tlp_blocks = redact::extract_tlp_blocks(&original);
    let inline_chunks = redact::extract_inline_tlp_chunks(&original);
    let tlp_redacted = redact::redact_tlp_sections(&original);
    let secrets = redact::extract_secret_matches(&tlp_redacted);

    let has_hidden = !tlp_blocks.is_empty() || !inline_chunks.is_empty() || !secrets.is_empty();

    if !has_hidden {
        // No hidden content — but check for spurious markers in new content
        let has_markers = new_content.contains(redact::REDACTED_MARKER)
            || new_content.contains(redact::SECRET_MARKER);
        if has_markers {
            eprintln!(
                "New content contains redaction markers but the original file has no \
                 hidden content to restore. This would write literal marker text."
            );
            return ExitCode::from(1);
        }
        if let Err(e) = fs::write(file_path, &new_content) {
            eprintln!("Cannot write {file_path}: {e}");
            return ExitCode::from(1);
        }
        println!("{file_path}");
        return ExitCode::SUCCESS;
    }

    // Restore hidden content into the new text
    match redact::restore_hidden(&new_content, &tlp_blocks, &inline_chunks, &secrets) {
        Ok(merged) => {
            if let Err(e) = fs::write(file_path, &merged) {
                eprintln!("Cannot write {file_path}: {e}");
                return ExitCode::from(1);
            }
            eprintln!(
                "Restored {} TLP block(s), {} inline chunk(s), {} secret(s)",
                tlp_blocks.len(),
                inline_chunks.len(),
                secrets.len()
            );
            println!("{file_path}");
            ExitCode::SUCCESS
        }
        Err(msg) => {
            eprintln!("Restoration failed: {msg}");
            eprintln!("The original file was NOT modified.");
            ExitCode::from(1)
        }
    }
}
