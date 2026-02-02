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

// ─── Inline TLP redaction tests ───

#[test]
fn test_inline_single_marker() {
    let input = "Normal text #tlp/red secret text #tlp/amber more normal text\n";
    assert_eq!(
        redact_tlp_sections(input),
        "Normal text [REDACTED] more normal text\n"
    );
}

#[test]
fn test_inline_unterminated() {
    let input = "Text #tlp/red secret to end of line\n";
    assert_eq!(redact_tlp_sections(input), "Text [REDACTED]\n");
}

#[test]
fn test_inline_multiple_markers_same_line() {
    let input = "A #tlp/red secret1 #tlp/amber B #tlp/red secret2 #tlp/green C\n";
    assert_eq!(redact_tlp_sections(input), "A [REDACTED] B [REDACTED] C\n");
}

#[test]
fn test_inline_with_green_boundary() {
    let input = "Start #tlp/red hidden #tlp/green visible\n";
    assert_eq!(redact_tlp_sections(input), "Start [REDACTED] visible\n");
}

#[test]
fn test_inline_with_clear_boundary() {
    let input = "Start #tlp/red hidden #tlp/clear visible\n";
    assert_eq!(redact_tlp_sections(input), "Start [REDACTED] visible\n");
}

#[test]
fn test_inline_mixed_with_block() {
    let input =
        "Before\n#tlp/red\nBlock secret\n#tlp/amber\nMiddle #tlp/red inline secret\nAfter\n";
    assert_eq!(
        redact_tlp_sections(input),
        "Before\n[REDACTED]\nMiddle [REDACTED]\nAfter\n"
    );
}

#[test]
fn test_no_inline_false_positive() {
    // Lines without #tlp/red should pass through unchanged
    let input = "Normal #tlp/amber text\n";
    assert_eq!(redact_tlp_sections(input), "Normal #tlp/amber text\n");
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

#[test]
fn test_aws_access_key() {
    let input = "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE\n";
    let (output, found) = redact_secrets(input);
    assert!(found);
    assert!(!output.contains("AKIA"));
    assert!(output.contains("[SECRET REDACTED]"));
}

#[test]
fn test_stripe_key() {
    let input = "key: rk_prod_abcdefghijklmnopqrstuvwx\n";
    let (output, found) = redact_secrets(input);
    assert!(found);
    assert!(!output.contains("rk_prod_"));
    assert!(output.contains("[SECRET REDACTED]"));
}

#[test]
fn test_github_pat() {
    let input = "GITHUB_TOKEN=ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij\n";
    let (output, found) = redact_secrets(input);
    assert!(found);
    assert!(!output.contains("ghp_"));
    assert!(output.contains("[SECRET REDACTED]"));
}

#[test]
fn test_multiple_secrets_per_line() {
    let input = "AKIAIOSFODNN7EXAMPLE and glpat-ABCDEFGHIJKLMNOPQRST\n";
    let (output, found) = redact_secrets(input);
    assert!(found);
    assert!(!output.contains("AKIA"));
    assert!(!output.contains("glpat-"));
    assert_eq!(output.matches("[SECRET REDACTED]").count(), 2);
}

#[test]
fn test_pem_private_key_header() {
    let input = "-----BEGIN RSA PRIVATE KEY-----\n";
    let (output, found) = redact_secrets(input);
    assert!(found);
    assert!(output.contains("[SECRET REDACTED]"));
}

#[test]
fn test_slack_app_token() {
    let input = "SLACK_TOKEN=xoxa-0000000000-0000000000000-aaaaaaaaaaaaaaaaaaaaaaaa\n";
    let (output, found) = redact_secrets(input);
    assert!(found);
    assert!(!output.contains("xoxa-"));
    assert!(output.contains("[SECRET REDACTED]"));
}

#[test]
fn test_mongodb_connection_string() {
    let input = "MONGO_URI=mongodb+srv://admin:s3cret@cluster0.example.net/db\n";
    let (output, found) = redact_secrets(input);
    assert!(found);
    assert!(!output.contains("s3cret"));
    assert!(output.contains("[SECRET REDACTED]"));
}

#[test]
fn test_gcp_api_key() {
    let input = "GCP_KEY=AIzaSyA1234567890abcdefghijklmnopqrstuv\n";
    let (output, found) = redact_secrets(input);
    assert!(found);
    assert!(!output.contains("AIza"));
    assert!(output.contains("[SECRET REDACTED]"));
}
