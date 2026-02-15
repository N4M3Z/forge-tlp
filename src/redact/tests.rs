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

// ─── TLP block extraction tests ───

#[test]
fn test_extract_no_blocks() {
    assert!(extract_tlp_blocks("Normal content\nNo blocks here\n").is_empty());
}

#[test]
fn test_extract_single_block() {
    let input = "Before\n#tlp/red\nSecret A\nSecret B\n#tlp/amber\nAfter\n";
    let blocks = extract_tlp_blocks(input);
    assert_eq!(blocks.len(), 1);
    assert!(blocks[0].contains("#tlp/red"));
    assert!(blocks[0].contains("Secret A"));
    assert!(blocks[0].contains("Secret B"));
    assert!(blocks[0].contains("#tlp/amber"));
}

#[test]
fn test_extract_multiple_blocks() {
    let input = "A\n#tlp/red\nX\n#tlp/amber\nB\n#tlp/red\nY\n#tlp/green\nC\n";
    let blocks = extract_tlp_blocks(input);
    assert_eq!(blocks.len(), 2);
    assert!(blocks[0].contains("X"));
    assert!(blocks[1].contains("Y"));
    assert!(blocks[1].contains("#tlp/green"));
}

#[test]
fn test_extract_unterminated_block() {
    let input = "Before\n#tlp/red\nSecret to end\n";
    let blocks = extract_tlp_blocks(input);
    assert_eq!(blocks.len(), 1);
    assert!(blocks[0].contains("Secret to end"));
}

#[test]
fn test_extract_empty_block() {
    let input = "Before\n#tlp/red\n#tlp/amber\nAfter\n";
    let blocks = extract_tlp_blocks(input);
    assert_eq!(blocks.len(), 1);
    assert!(blocks[0].contains("#tlp/red"));
    assert!(blocks[0].contains("#tlp/amber"));
}

#[test]
fn test_extract_block_with_clear_boundary() {
    let input = "#tlp/red\nHidden\n#tlp/clear\nVisible\n";
    let blocks = extract_tlp_blocks(input);
    assert_eq!(blocks.len(), 1);
    assert!(blocks[0].contains("#tlp/clear"));
}

// ─── Inline TLP extraction tests ───

#[test]
fn test_extract_inline_no_markers() {
    assert!(extract_inline_tlp_chunks("Normal line\nAnother line\n").is_empty());
}

#[test]
fn test_extract_inline_single() {
    let input = "Text #tlp/red secret stuff #tlp/amber more text\n";
    let chunks = extract_inline_tlp_chunks(input);
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].contains("#tlp/red"));
    assert!(chunks[0].contains("secret stuff"));
    assert!(chunks[0].contains("#tlp/amber"));
}

#[test]
fn test_extract_inline_unterminated() {
    let input = "Start #tlp/red secret to end\n";
    let chunks = extract_inline_tlp_chunks(input);
    assert_eq!(chunks.len(), 1);
    assert!(chunks[0].starts_with("#tlp/red"));
    assert!(chunks[0].contains("secret to end"));
}

#[test]
fn test_extract_inline_multiple_on_one_line() {
    let input = "A #tlp/red s1 #tlp/amber B #tlp/red s2 #tlp/green C\n";
    let chunks = extract_inline_tlp_chunks(input);
    assert_eq!(chunks.len(), 2);
    assert!(chunks[0].contains("s1"));
    assert!(chunks[1].contains("s2"));
}

#[test]
fn test_extract_inline_skips_block_content() {
    // Inline markers inside a block should NOT be extracted as inline
    let input = "Before\n#tlp/red\nBlock #tlp/red not-inline\n#tlp/amber\nAfter\n";
    let chunks = extract_inline_tlp_chunks(input);
    assert!(chunks.is_empty());
}

#[test]
fn test_extract_inline_mixed_with_blocks() {
    let input =
        "A #tlp/red i1 #tlp/amber B\n#tlp/red\nBlock\n#tlp/amber\nC #tlp/red i2 #tlp/green D\n";
    let chunks = extract_inline_tlp_chunks(input);
    assert_eq!(chunks.len(), 2);
    assert!(chunks[0].contains("i1"));
    assert!(chunks[1].contains("i2"));
}

// ─── Secret match extraction tests ───

#[test]
fn test_extract_no_secrets() {
    assert!(extract_secret_matches("Normal content\n").is_empty());
}

#[test]
fn test_extract_single_secret() {
    let input = "KEY=sk-ant-api03-AAAAAAAAAAAAAAAAAAAAAA\n";
    let matches = extract_secret_matches(input);
    assert_eq!(matches.len(), 1);
    assert!(matches[0].starts_with("sk-ant-api03-"));
}

#[test]
fn test_extract_multiple_secrets_different_lines() {
    let input = "A=AKIAIOSFODNN7EXAMPLE\nB=glpat-ABCDEFGHIJKLMNOPQRST\n";
    let matches = extract_secret_matches(input);
    assert_eq!(matches.len(), 2);
}

#[test]
fn test_extract_multiple_secrets_same_line() {
    let input = "AKIAIOSFODNN7EXAMPLE and glpat-ABCDEFGHIJKLMNOPQRST\n";
    let matches = extract_secret_matches(input);
    assert_eq!(matches.len(), 2);
}

// ─── Restoration tests ───

#[test]
fn test_restore_no_hidden() {
    let result = restore_hidden("Plain text\n", &[], &[], &[]).unwrap();
    assert_eq!(result, "Plain text\n");
}

#[test]
fn test_restore_single_block() {
    let new = "Before\n[REDACTED]\nAfter\n";
    let blocks = vec!["#tlp/red\nSecret\n#tlp/amber".to_string()];
    let result = restore_hidden(new, &blocks, &[], &[]).unwrap();
    assert_eq!(result, "Before\n#tlp/red\nSecret\n#tlp/amber\nAfter\n");
}

#[test]
fn test_restore_multiple_blocks() {
    let new = "A\n[REDACTED]\nB\n[REDACTED]\nC\n";
    let blocks = vec![
        "#tlp/red\nX\n#tlp/amber".to_string(),
        "#tlp/red\nY\n#tlp/green".to_string(),
    ];
    let result = restore_hidden(new, &blocks, &[], &[]).unwrap();
    assert!(result.contains("X"));
    assert!(result.contains("Y"));
    assert!(!result.contains("[REDACTED]"));
}

#[test]
fn test_restore_inline_chunks() {
    let new = "Text [REDACTED] continues\n";
    let inlines = vec!["#tlp/red secret #tlp/amber".to_string()];
    let result = restore_hidden(new, &[], &inlines, &[]).unwrap();
    assert_eq!(result, "Text #tlp/red secret #tlp/amber continues\n");
}

#[test]
fn test_restore_secrets() {
    let new = "Key: [SECRET REDACTED]\n";
    let secrets = vec!["sk-ant-api03-REALKEY".to_string()];
    let result = restore_hidden(new, &[], &[], &secrets).unwrap();
    assert_eq!(result, "Key: sk-ant-api03-REALKEY\n");
}

#[test]
fn test_restore_mixed_all_types() {
    let new = "A [REDACTED] B\n[REDACTED]\nC: [SECRET REDACTED]\n";
    let blocks = vec!["#tlp/red\nHidden\n#tlp/amber".to_string()];
    let inlines = vec!["#tlp/red inline-secret #tlp/green".to_string()];
    let secrets = vec!["ghp_REALTOKEN12345678901234567890abcdef".to_string()];
    let result = restore_hidden(new, &blocks, &inlines, &secrets).unwrap();
    assert!(result.contains("#tlp/red inline-secret #tlp/green"));
    assert!(result.contains("#tlp/red\nHidden\n#tlp/amber"));
    assert!(result.contains("ghp_REALTOKEN"));
}

#[test]
fn test_restore_fails_too_few_blocks() {
    let new = "A\nB\n";
    let blocks = vec!["#tlp/red\nX\n#tlp/amber".to_string()];
    assert!(restore_hidden(new, &blocks, &[], &[]).is_err());
}

#[test]
fn test_restore_fails_too_many_block_markers() {
    let new = "[REDACTED]\n[REDACTED]\n";
    let blocks = vec!["#tlp/red\nX\n#tlp/amber".to_string()];
    assert!(restore_hidden(new, &blocks, &[], &[]).is_err());
}

#[test]
fn test_restore_fails_too_few_secrets() {
    let new = "Key: [SECRET REDACTED]\n";
    assert!(restore_hidden(new, &[], &[], &[]).is_err());
}

#[test]
fn test_restore_fails_too_many_secret_markers() {
    let new = "[SECRET REDACTED] and [SECRET REDACTED]\n";
    let secrets = vec!["one-secret".to_string()];
    assert!(restore_hidden(new, &[], &[], &secrets).is_err());
}

#[test]
fn test_restore_preserves_trailing_newline() {
    let result = restore_hidden("Text\n", &[], &[], &[]).unwrap();
    assert!(result.ends_with('\n'));
}

#[test]
fn test_restore_no_trailing_newline_when_input_has_none() {
    let result = restore_hidden("Text", &[], &[], &[]).unwrap();
    assert!(!result.ends_with('\n'));
}

// ─── Round-trip tests (redact → extract → restore = original) ───

#[test]
fn test_roundtrip_block() {
    let original = "Before\n#tlp/red\nSecret line\n#tlp/amber\nAfter\n";
    let safe_view = redact_tlp_sections(original);
    let blocks = extract_tlp_blocks(original);
    let inlines = extract_inline_tlp_chunks(original);
    let secrets = extract_secret_matches(&safe_view);

    let restored = restore_hidden(&safe_view, &blocks, &inlines, &secrets).unwrap();
    assert_eq!(restored, original);
}

#[test]
fn test_roundtrip_inline() {
    let original = "Public #tlp/red private stuff #tlp/amber end\n";
    let safe_view = redact_tlp_sections(original);
    let blocks = extract_tlp_blocks(original);
    let inlines = extract_inline_tlp_chunks(original);
    let secrets = extract_secret_matches(&safe_view);

    let restored = restore_hidden(&safe_view, &blocks, &inlines, &secrets).unwrap();
    assert_eq!(restored, original);
}

#[test]
fn test_roundtrip_secret() {
    let original = "Token: sk-ant-api03-AAAAAAAAAAAAAAAAAAAAAA\n";
    let safe_view = redact_tlp_sections(original);
    let (secret_redacted, _) = redact_secrets(&safe_view);
    let blocks = extract_tlp_blocks(original);
    let inlines = extract_inline_tlp_chunks(original);
    let secrets = extract_secret_matches(&safe_view);

    let restored = restore_hidden(&secret_redacted, &blocks, &inlines, &secrets).unwrap();
    assert_eq!(restored, original);
}

#[test]
fn test_roundtrip_complex() {
    let original = "\
---
title: Complex
tlp: amber
---

Visible A.
#tlp/red
Hidden block 1.
Hidden block 2.
#tlp/amber
Middle text #tlp/red inline-secret #tlp/green visible again.
Key: sk-ant-api03-AAAAAAAAAAAAAAAAAAAAAA
#tlp/red
Another hidden section.
#tlp/clear
End.
";
    let safe_view = redact_tlp_sections(original);
    let (full_safe, _) = redact_secrets(&safe_view);
    let blocks = extract_tlp_blocks(original);
    let inlines = extract_inline_tlp_chunks(original);
    let secrets = extract_secret_matches(&safe_view);

    let restored = restore_hidden(&full_safe, &blocks, &inlines, &secrets).unwrap();
    assert_eq!(restored, original);
}
