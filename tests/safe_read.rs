#![allow(deprecated)] // Command::cargo_bin is the standard assert_cmd API

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ─── Fixture constants ───

const CONFIG_RED_PDF: &str = include_str!("fixtures/configs/red_pdf.tlp");
const CONFIG_GREEN_MD: &str = include_str!("fixtures/configs/green_md.tlp");
const CONFIG_AMBER_MD: &str = include_str!("fixtures/configs/amber_md.tlp");

const CONTENT_PLAIN: &str = include_str!("fixtures/content/plain.md");
const CONTENT_WITH_REDACTION: &str = include_str!("fixtures/content/with_redaction.md");
const CONTENT_MULTIPLE_REDACTIONS: &str = include_str!("fixtures/content/multiple_redactions.md");
const CONTENT_UNTERMINATED_RED: &str = include_str!("fixtures/content/unterminated_red.md");
const CONTENT_WITH_API_KEY: &str = include_str!("fixtures/content/with_api_key.md");
const CONTENT_WITH_GITLAB_TOKEN: &str = include_str!("fixtures/content/with_gitlab_token.md");
const CONTENT_SHORT_SK: &str = include_str!("fixtures/content/short_sk_false_positive.md");
const CONTENT_FULL_PIPELINE: &str = include_str!("fixtures/content/full_pipeline.md");
const CONTENT_FRONTMATTER_RED: &str = include_str!("fixtures/content/frontmatter_red.md");
const CONTENT_COMPLEX_MARKDOWN: &str = include_str!("fixtures/content/complex_markdown.md");
const CONTENT_SOURCE_CODE: &str = include_str!("fixtures/content/source_code.md");
const CONTENT_INLINE_REDACTION: &str = include_str!("fixtures/content/inline_redaction.md");
const CONTENT_INLINE_UNTERMINATED: &str = include_str!("fixtures/content/inline_unterminated.md");
const CONTENT_WITH_STRIPE_KEY: &str = include_str!("fixtures/content/with_stripe_key.md");
const CONTENT_WITH_AWS_KEY: &str = include_str!("fixtures/content/with_aws_key.md");
const CONTENT_MULTIPLE_SECRETS: &str = include_str!("fixtures/content/multiple_secrets.md");

// ─── Basic usage ───

#[test]
fn no_args_exits_1_with_usage() {
    Command::cargo_bin("safe-read")
        .unwrap()
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn nonexistent_file_exits_1() {
    Command::cargo_bin("safe-read")
        .unwrap()
        .arg("/tmp/this-file-does-not-exist-12345.md")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Cannot read"));
}

// ─── Plain file passthrough ───

#[test]
fn plain_file_outputs_content() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_PLAIN).unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(CONTENT_PLAIN);
}

// ─── TLP redaction ───

#[test]
fn redacts_tlp_red_section() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_REDACTION).unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("[REDACTED]"))
        .stdout(predicate::str::contains("Secret").not());
}

#[test]
fn redacts_multiple_sections() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_MULTIPLE_REDACTIONS).unwrap();

    let output = Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches("[REDACTED]").count(), 2);
    assert!(stdout.contains("A\n"));
    assert!(stdout.contains("B\n"));
    assert!(stdout.contains("C\n"));
}

#[test]
fn redacts_unterminated_block() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_UNTERMINATED_RED).unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("[REDACTED]"))
        .stdout(predicate::str::contains("Secret").not());
}

// ─── Secret redaction ───

#[test]
fn redacts_api_keys() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_API_KEY).unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("[SECRET REDACTED]"))
        .stdout(predicate::str::contains("sk-ant-api03").not())
        .stderr(predicate::str::contains("WARNING"));
}

#[test]
fn redacts_gitlab_token() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_GITLAB_TOKEN).unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("[SECRET REDACTED]"))
        .stdout(predicate::str::contains("glpat-").not());
}

#[test]
fn no_false_positive_on_short_sk() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_SHORT_SK).unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(CONTENT_SHORT_SK);
}

// ─── Combined TLP + secret redaction ───

#[test]
fn full_pipeline_redacts_both() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_FULL_PIPELINE).unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("[REDACTED]"))
        .stdout(predicate::str::contains("[SECRET REDACTED]"))
        .stdout(predicate::str::contains("top secret").not())
        .stdout(predicate::str::contains("sk-ant-api03").not());
}

// ─── RED file refusal ───

#[test]
fn refuses_red_file_by_path() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".tlp"), CONFIG_RED_PDF).unwrap();
    let file = dir.path().join("secret.pdf");
    fs::write(&file, "binary content").unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .code(1)
        .stderr(predicate::str::contains("TLP:RED"));
}

#[test]
fn refuses_red_file_by_frontmatter() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".tlp"), CONFIG_GREEN_MD).unwrap();
    let file = dir.path().join("secret.md");
    fs::write(&file, CONTENT_FRONTMATTER_RED).unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .code(1)
        .stderr(predicate::str::contains("TLP:RED"));
}

#[test]
fn allows_amber_file() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".tlp"), CONFIG_AMBER_MD).unwrap();
    let file = dir.path().join("journal.md");
    fs::write(&file, "diary entry\n").unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("diary entry"));
}

#[test]
fn file_outside_vault_still_works() {
    // Files outside any vault (no .tlp found) should still be readable
    let dir = tempdir().unwrap();
    let file = dir.path().join("standalone.md");
    fs::write(&file, "no vault here\n").unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout("no vault here\n");
}

// ─── Inline TLP redaction ───

#[test]
fn redacts_inline_marker() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_INLINE_REDACTION).unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("[REDACTED]"))
        .stdout(predicate::str::contains("secret information").not())
        .stdout(predicate::str::contains("that resumes here"));
}

#[test]
fn redacts_inline_unterminated_marker() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_INLINE_UNTERMINATED).unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("[REDACTED]"))
        .stdout(predicate::str::contains("secret to end").not())
        .stdout(predicate::str::contains("Back to normal content"));
}

// ─── Expanded secret detection (gitleaks patterns) ───

#[test]
fn redacts_stripe_key() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_STRIPE_KEY).unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("[SECRET REDACTED]"))
        .stdout(predicate::str::contains("rk_prod_").not());
}

#[test]
fn redacts_aws_key() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_AWS_KEY).unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("[SECRET REDACTED]"))
        .stdout(predicate::str::contains("AKIA").not());
}

#[test]
fn multiple_secrets_redacted() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_MULTIPLE_SECRETS).unwrap();

    let output = Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(stdout.matches("[SECRET REDACTED]").count(), 2);
    assert!(!stdout.contains("ghp_"));
    assert!(!stdout.contains("glpat-"));
}

// ─── File corruption tests ───

#[test]
fn complex_markdown_passes_through_unchanged() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("complex.md");
    fs::write(&file, CONTENT_COMPLEX_MARKDOWN).unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(CONTENT_COMPLEX_MARKDOWN);
}

#[test]
fn source_code_with_hashes_passes_through_unchanged() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("code.md");
    fs::write(&file, CONTENT_SOURCE_CODE).unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout(CONTENT_SOURCE_CODE);
}
