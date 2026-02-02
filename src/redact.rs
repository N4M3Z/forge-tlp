const TLP_RED_MARKER: &str = "#tlp/red";
const TLP_BOUNDARY_TAGS: &[&str] = &["#tlp/amber", "#tlp/green", "#tlp/clear"];

/// Known API key prefixes. Ordered longest-first so more specific prefixes match first.
const SECRET_PREFIXES: &[(&str, usize)] = &[
    ("sk-ant-api", 20), // Anthropic
    ("sk-proj-", 20),   // OpenAI project
    ("sk-or-", 20),     // OpenRouter
    ("ghp_", 20),       // GitHub personal access token
    ("gho_", 20),       // GitHub OAuth token
    ("ghs_", 20),       // GitHub server-to-server token
    ("ghu_", 20),       // GitHub user-to-server token
    ("glpat-", 20),     // GitLab personal access token
    ("xoxb-", 20),      // Slack bot token
    ("xoxp-", 20),      // Slack user token
    ("AKIA", 16),       // AWS access key ID
    ("sk-", 30),        // Generic (OpenAI, Stripe) — higher min length to reduce false positives
];

/// Check if a trimmed line is a TLP boundary tag (not #tlp/red, which starts sections).
fn is_tlp_boundary(trimmed: &str) -> bool {
    TLP_BOUNDARY_TAGS.contains(&trimmed)
}

/// Strip content between #tlp/red and any other #tlp/* boundary marker.
/// Each RED section is replaced with a single [REDACTED] line.
pub fn redact_tlp_sections(content: &str) -> String {
    let mut result = Vec::new();
    let mut in_redacted = false;
    let mut redaction_emitted = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == TLP_RED_MARKER {
            if !in_redacted {
                in_redacted = true;
                redaction_emitted = false;
            }
            continue;
        }

        if in_redacted && is_tlp_boundary(trimmed) {
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

/// Scan content for known secret patterns and redact them.
/// Returns (redacted_content, secrets_found).
pub fn redact_secrets(content: &str) -> (String, bool) {
    let mut result = Vec::new();
    let mut found = false;

    for line in content.lines() {
        let (redacted, had_secret) = redact_line_secrets(line);
        if had_secret {
            found = true;
        }
        result.push(redacted);
    }

    let mut output = result.join("\n");
    if content.ends_with('\n') {
        output.push('\n');
    }
    (output, found)
}

/// Check a single line for secret prefixes and redact the containing token.
fn redact_line_secrets(line: &str) -> (String, bool) {
    let mut output = line.to_string();
    let mut found = false;

    for &(prefix, min_len) in SECRET_PREFIXES {
        let mut search_from = 0;
        while search_from < output.len() {
            let haystack = &output[search_from..];
            let rel_start = match haystack.find(prefix) {
                Some(pos) => pos,
                None => break,
            };
            let abs_start = search_from + rel_start;

            let abs_end = find_token_end(&output, abs_start);
            let token_len = abs_end - abs_start;

            if token_len >= min_len {
                let replacement = "[SECRET REDACTED]";
                output = format!(
                    "{}{}{}",
                    &output[..abs_start],
                    replacement,
                    &output[abs_end..]
                );
                found = true;
                search_from = abs_start + replacement.len();
            } else {
                search_from = abs_start + prefix.len();
            }
        }
    }

    (output, found)
}

/// Find where a token ends — stops at delimiters common in JSON, YAML, env files.
fn find_token_end(content: &str, start: usize) -> usize {
    let bytes = content.as_bytes();
    let mut i = start;
    while i < bytes.len() {
        match bytes[i] {
            b'"' | b'\'' | b' ' | b'\t' | b'\n' | b'\r' | b',' | b';' | b'}' | b']' => break,
            _ => i += 1,
        }
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── TLP redaction tests ───

    #[test]
    fn test_no_markers() {
        let input = "Line 1\nLine 2\nLine 3\n";
        assert_eq!(redact_tlp_sections(input), "Line 1\nLine 2\nLine 3\n");
    }

    #[test]
    fn test_single_section() {
        let input = "Before\n#tlp/red\nSecret\n#tlp/amber\nAfter\n";
        assert_eq!(redact_tlp_sections(input), "Before\n[REDACTED]\nAfter\n");
    }

    #[test]
    fn test_unterminated_block() {
        let input = "Before\n#tlp/red\nSecret\n";
        assert_eq!(redact_tlp_sections(input), "Before\n[REDACTED]\n");
    }

    #[test]
    fn test_empty_file() {
        assert_eq!(redact_tlp_sections(""), "");
    }

    #[test]
    fn test_lone_red_marker() {
        let input = "#tlp/red\n";
        assert_eq!(redact_tlp_sections(input), "[REDACTED]\n");
    }

    // ─── Secret detection tests ───

    #[test]
    fn test_gitlab_token() {
        let input = "GITLAB_TOKEN=glpat-ABCDEFGHIJKLMNOPQRST\n";
        let (output, found) = redact_secrets(input);
        assert!(found);
        assert!(!output.contains("glpat-"));
        assert!(output.contains("[SECRET REDACTED]"));
    }

    #[test]
    fn test_secret_at_line_start() {
        let input = "sk-ant-api03-AAAAAAAAAAAAAAAAAAAAAA\n";
        let (output, found) = redact_secrets(input);
        assert!(found);
        assert!(output.starts_with("[SECRET REDACTED]"));
    }

    #[test]
    fn test_short_token_ignored() {
        let input = "sk-ip this\n";
        let (output, found) = redact_secrets(input);
        assert!(!found);
        assert_eq!(output, input);
    }

    #[test]
    fn test_empty_content() {
        let (output, found) = redact_secrets("");
        assert!(!found);
        assert_eq!(output, "");
    }
}
