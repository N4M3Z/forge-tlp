use std::env;
use std::fs;
use std::process::ExitCode;

const TLP_TAGS: &[&str] = &["#tlp/red", "#tlp/amber", "#tlp/green", "#tlp/clear"];

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: safe-read <file-path>");
        return ExitCode::from(1);
    }

    let content = match fs::read_to_string(&args[1]) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Cannot read {}: {}", args[1], e);
            return ExitCode::from(1);
        }
    };

    print!("{}", redact_content(&content));
    ExitCode::SUCCESS
}

/// Strip content between #tlp/red and any other #tlp/* marker.
/// Each RED section is replaced with a single [REDACTED] line.
fn redact_content(content: &str) -> String {
    let mut result = Vec::new();
    let mut in_redacted = false;
    let mut redaction_emitted = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "#tlp/red" {
            if !in_redacted {
                in_redacted = true;
                redaction_emitted = false;
            }
            continue;
        }

        if in_redacted && is_tlp_tag(trimmed) {
            if !redaction_emitted {
                result.push("[REDACTED]".to_string());
            }
            in_redacted = false;
            continue;
        }

        if in_redacted {
            if !redaction_emitted {
                result.push("[REDACTED]".to_string());
                redaction_emitted = true;
            }
        } else {
            result.push(line.to_string());
        }
    }

    // Handle unterminated RED block
    if in_redacted && !redaction_emitted {
        result.push("[REDACTED]".to_string());
    }

    let mut output = result.join("\n");
    if content.ends_with('\n') {
        output.push('\n');
    }
    output
}

/// Check if a trimmed line is a TLP tag (but not #tlp/red, which is handled separately).
fn is_tlp_tag(trimmed: &str) -> bool {
    TLP_TAGS.iter().any(|tag| *tag == trimmed && *tag != "#tlp/red")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_markers() {
        let input = "Line 1\nLine 2\nLine 3\n";
        assert_eq!(redact_content(input), "Line 1\nLine 2\nLine 3\n");
    }

    #[test]
    fn test_single_section_closed_by_amber() {
        let input = "Before\n#tlp/red\nSecret\nMore secrets\n#tlp/amber\nAfter\n";
        assert_eq!(redact_content(input), "Before\n[REDACTED]\nAfter\n");
    }

    #[test]
    fn test_closed_by_green() {
        let input = "Before\n#tlp/red\nSecret\n#tlp/green\nAfter\n";
        assert_eq!(redact_content(input), "Before\n[REDACTED]\nAfter\n");
    }

    #[test]
    fn test_closed_by_clear() {
        let input = "Before\n#tlp/red\nSecret\n#tlp/clear\nAfter\n";
        assert_eq!(redact_content(input), "Before\n[REDACTED]\nAfter\n");
    }

    #[test]
    fn test_multiple_sections() {
        let input = "A\n#tlp/red\nX\n#tlp/amber\nB\n#tlp/red\nY\n#tlp/green\nC\n";
        assert_eq!(redact_content(input), "A\n[REDACTED]\nB\n[REDACTED]\nC\n");
    }

    #[test]
    fn test_unterminated_block() {
        let input = "Before\n#tlp/red\nSecret\nMore secret\n";
        assert_eq!(redact_content(input), "Before\n[REDACTED]\n");
    }

    #[test]
    fn test_markers_with_whitespace() {
        let input = "Before\n  #tlp/red  \nSecret\n  #tlp/amber  \nAfter\n";
        assert_eq!(redact_content(input), "Before\n[REDACTED]\nAfter\n");
    }

    #[test]
    fn test_inline_not_triggered() {
        let input = "This has #tlp/red in the middle\n";
        assert_eq!(redact_content(input), "This has #tlp/red in the middle\n");
    }

    #[test]
    fn test_empty_section() {
        let input = "Before\n#tlp/red\n#tlp/amber\nAfter\n";
        assert_eq!(redact_content(input), "Before\n[REDACTED]\nAfter\n");
    }
}
