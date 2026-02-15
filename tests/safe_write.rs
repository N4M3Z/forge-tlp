#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ─── Fixtures ───

const CONTENT_PLAIN: &str = "\
---
title: Test
tlp: amber
---

Hello world.
Some content here.
";

const CONTENT_WITH_TLP_BLOCK: &str = "\
---
title: Test
tlp: amber
---

Visible top.
#tlp/red
This is secret line A.
This is secret line B.
#tlp/amber
Visible bottom.
";

const CONTENT_WITH_INLINE_TLP: &str = "\
---
title: Test
tlp: amber
---

Public info #tlp/red secret stuff #tlp/amber continues here.
";

const CONTENT_WITH_SECRET: &str = "\
---
title: Test
tlp: amber
---

API key: sk-ant-api03-AAAAAAAAAAAAAAAAAAAAAAAAAAAA
Normal line.
";

const CONTENT_WITH_BLOCK_AND_SECRET: &str = "\
---
title: Test
tlp: amber
---

Visible top.
#tlp/red
Hidden block content.
#tlp/amber
Line with key: ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij
Visible bottom.
";

const CONTENT_FRONTMATTER_RED: &str = "\
---
title: Secret
tlp: red
---

This should be inaccessible.
";

// ─── Edit mode: basic ───

#[test]
fn edit_no_args_exits_1() {
    Command::cargo_bin("safe-write")
        .unwrap()
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn edit_replaces_unique_string() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_PLAIN).unwrap();

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["edit", file.to_str().unwrap(), "--old", "Hello world.", "--new", "Goodbye world."])
        .assert()
        .success()
        .stdout(predicate::str::contains(file.to_str().unwrap()));

    let result = fs::read_to_string(&file).unwrap();
    assert!(result.contains("Goodbye world."));
    assert!(!result.contains("Hello world."));
}

#[test]
fn edit_fails_on_missing_string() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_PLAIN).unwrap();

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["edit", file.to_str().unwrap(), "--old", "nonexistent", "--new", "replacement"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn edit_fails_on_ambiguous_string() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    // "content" appears multiple times in CONTENT_PLAIN-like text
    fs::write(&file, "AAA\nBBB\nAAA\n").unwrap();

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["edit", file.to_str().unwrap(), "--old", "AAA", "--new", "CCC"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("2 times"));
}

// ─── Edit mode: preserves redacted content ───

#[test]
fn edit_preserves_tlp_block_when_editing_visible_text() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_TLP_BLOCK).unwrap();

    // Edit visible text — hidden block should survive
    Command::cargo_bin("safe-write")
        .unwrap()
        .args([
            "edit",
            file.to_str().unwrap(),
            "--old",
            "Visible top.",
            "--new",
            "Modified top.",
        ])
        .assert()
        .success();

    let result = fs::read_to_string(&file).unwrap();
    assert!(result.contains("Modified top."));
    assert!(result.contains("#tlp/red"));
    assert!(result.contains("This is secret line A."));
    assert!(result.contains("This is secret line B."));
    assert!(result.contains("#tlp/amber"));
    assert!(result.contains("Visible bottom."));
}

#[test]
fn edit_rejects_redaction_markers_in_old_string() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_PLAIN).unwrap();

    Command::cargo_bin("safe-write")
        .unwrap()
        .args([
            "edit",
            file.to_str().unwrap(),
            "--old",
            "[REDACTED]",
            "--new",
            "replaced",
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("redaction markers"));

    Command::cargo_bin("safe-write")
        .unwrap()
        .args([
            "edit",
            file.to_str().unwrap(),
            "--old",
            "[SECRET REDACTED]",
            "--new",
            "replaced",
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("redaction markers"));
}

// ─── Edit mode: RED refusal ───

#[test]
fn edit_refuses_red_file() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".tlp"), "**/*.md RED\n").unwrap();
    let file = dir.path().join("secret.md");
    fs::write(&file, CONTENT_FRONTMATTER_RED).unwrap();

    Command::cargo_bin("safe-write")
        .unwrap()
        .args([
            "edit",
            file.to_str().unwrap(),
            "--old",
            "inaccessible",
            "--new",
            "replaced",
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("TLP:RED"));
}

// ─── Write mode: plain file ───

#[test]
fn write_overwrites_plain_file() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_PLAIN).unwrap();

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin("New content.\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(file.to_str().unwrap()));

    assert_eq!(fs::read_to_string(&file).unwrap(), "New content.\n");
}

#[test]
fn write_creates_new_file() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("brand_new.md");

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin("Fresh content.\n")
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&file).unwrap(), "Fresh content.\n");
}

#[test]
fn write_refuses_empty_stdin() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_PLAIN).unwrap();

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin("")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("empty"));
}

// ─── Write mode: TLP block restoration ───

#[test]
fn write_restores_tlp_block() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_TLP_BLOCK).unwrap();

    // Simulate what the AI would send: safe-read output with modifications
    let new_content = "\
---
title: Test
tlp: amber
---

Modified top.
[REDACTED]
Modified bottom.
";

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin(new_content)
        .assert()
        .success()
        .stderr(predicate::str::contains("Restored 1 TLP block(s)"));

    let result = fs::read_to_string(&file).unwrap();
    assert!(result.contains("Modified top."));
    assert!(result.contains("#tlp/red"));
    assert!(result.contains("This is secret line A."));
    assert!(result.contains("This is secret line B."));
    assert!(result.contains("#tlp/amber"));
    assert!(result.contains("Modified bottom."));
    assert!(!result.contains("[REDACTED]"));
}

// ─── Write mode: secret restoration ───

#[test]
fn write_restores_secrets() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_SECRET).unwrap();

    let new_content = "\
---
title: Test
tlp: amber
---

API key: [SECRET REDACTED]
Modified line.
";

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin(new_content)
        .assert()
        .success()
        .stderr(predicate::str::contains("1 secret(s)"));

    let result = fs::read_to_string(&file).unwrap();
    assert!(result.contains("sk-ant-api03-AAAAAAAAAAAAAAAAAAAAAAAAAAAA"));
    assert!(result.contains("Modified line."));
    assert!(!result.contains("[SECRET REDACTED]"));
}

// ─── Write mode: combined TLP block + secret ───

#[test]
fn write_restores_block_and_secret() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_BLOCK_AND_SECRET).unwrap();

    let new_content = "\
---
title: Test
tlp: amber
---

Modified top.
[REDACTED]
Line with key: [SECRET REDACTED]
Modified bottom.
";

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin(new_content)
        .assert()
        .success();

    let result = fs::read_to_string(&file).unwrap();
    assert!(result.contains("Modified top."));
    assert!(result.contains("#tlp/red"));
    assert!(result.contains("Hidden block content."));
    assert!(result.contains("#tlp/amber"));
    assert!(result.contains("ghp_ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghij"));
    assert!(result.contains("Modified bottom."));
}

// ─── Write mode: marker count mismatch ───

#[test]
fn write_fails_on_missing_redacted_marker() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_TLP_BLOCK).unwrap();

    // New content is missing the [REDACTED] marker — would lose the hidden block
    let new_content = "\
---
title: Test
tlp: amber
---

Modified top.
Modified bottom.
";

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin(new_content)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Restoration failed"));

    // Original should be untouched
    let result = fs::read_to_string(&file).unwrap();
    assert!(result.contains("Visible top."));
}

#[test]
fn write_fails_on_extra_redacted_marker() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_PLAIN).unwrap();

    // New content has [REDACTED] but original has no TLP blocks
    let new_content = "Before.\n[REDACTED]\nAfter.\n";

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin(new_content)
        .assert()
        .code(1)
        .stderr(predicate::str::contains("no hidden content to restore"));
}

// ─── Write mode: inline TLP restoration ───

#[test]
fn write_restores_inline_tlp() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_INLINE_TLP).unwrap();

    let new_content = "\
---
title: Test
tlp: amber
---

Modified info [REDACTED] continues here.
";

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin(new_content)
        .assert()
        .success();

    let result = fs::read_to_string(&file).unwrap();
    assert!(result.contains("Modified info"));
    assert!(result.contains("#tlp/red"));
    assert!(result.contains("secret stuff"));
    assert!(result.contains("#tlp/amber"));
    assert!(result.contains("continues here."));
}

// ─── Write mode: RED refusal ───

#[test]
fn write_refuses_red_file() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".tlp"), "**/*.md RED\n").unwrap();
    let file = dir.path().join("secret.md");
    fs::write(&file, CONTENT_FRONTMATTER_RED).unwrap();

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin("overwrite attempt\n")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("TLP:RED"));
}

// ─── Write mode: multiple TLP blocks ───

#[test]
fn write_restores_multiple_tlp_blocks() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    let original = "\
Top.
#tlp/red
Secret block 1.
#tlp/amber
Middle.
#tlp/red
Secret block 2.
#tlp/green
Bottom.
";
    fs::write(&file, original).unwrap();

    let new_content = "\
Modified top.
[REDACTED]
Modified middle.
[REDACTED]
Modified bottom.
";
    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin(new_content)
        .assert()
        .success()
        .stderr(predicate::str::contains("2 TLP block(s)"));

    let result = fs::read_to_string(&file).unwrap();
    assert!(result.contains("Modified top."));
    assert!(result.contains("Secret block 1."));
    assert!(result.contains("Secret block 2."));
    assert!(result.contains("#tlp/green"));
    assert!(result.contains("Modified bottom."));
}

// ─── Write mode: unterminated TLP block ───

#[test]
fn write_restores_unterminated_block() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    let original = "Before.\n#tlp/red\nUnterminated secret.\n";
    fs::write(&file, original).unwrap();

    let new_content = "Modified before.\n[REDACTED]\n";

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin(new_content)
        .assert()
        .success();

    let result = fs::read_to_string(&file).unwrap();
    assert!(result.contains("Modified before."));
    assert!(result.contains("#tlp/red"));
    assert!(result.contains("Unterminated secret."));
}

// ─── Write mode: multiple secrets ───

#[test]
fn write_restores_multiple_secrets() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    let original = "\
---
title: Secrets
---

AWS: AKIAIOSFODNN7EXAMPLE
Slack: xoxa-0000000000-0000000000000-aaaaaaaaaaaaaaaaaaaaaaaa
Done.
";
    fs::write(&file, original).unwrap();

    let new_content = "\
---
title: Secrets
---

AWS: [SECRET REDACTED]
Slack: [SECRET REDACTED]
Modified done.
";
    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin(new_content)
        .assert()
        .success()
        .stderr(predicate::str::contains("2 secret(s)"));

    let result = fs::read_to_string(&file).unwrap();
    assert!(result.contains("AKIAIOSFODNN7EXAMPLE"));
    assert!(result.contains("xoxa-0000000000"));
    assert!(result.contains("Modified done."));
}

// ─── Write mode: idempotency (safe-read → safe-write = original) ───

#[test]
fn write_roundtrip_preserves_original() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_TLP_BLOCK).unwrap();

    // Step 1: safe-read to get the view
    let safe_output = Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .output()
        .unwrap();
    let safe_view = String::from_utf8_lossy(&safe_output.stdout);

    // Step 2: safe-write back unchanged
    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin(safe_view.as_ref())
        .assert()
        .success();

    // Original should be perfectly preserved
    let result = fs::read_to_string(&file).unwrap();
    assert_eq!(result, CONTENT_WITH_TLP_BLOCK);
}

#[test]
fn write_roundtrip_with_secret_preserves_original() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_SECRET).unwrap();

    let safe_output = Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .output()
        .unwrap();
    let safe_view = String::from_utf8_lossy(&safe_output.stdout);

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin(safe_view.as_ref())
        .assert()
        .success();

    let result = fs::read_to_string(&file).unwrap();
    assert_eq!(result, CONTENT_WITH_SECRET);
}

#[test]
fn write_roundtrip_complex_preserves_original() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_BLOCK_AND_SECRET).unwrap();

    let safe_output = Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .output()
        .unwrap();
    let safe_view = String::from_utf8_lossy(&safe_output.stdout);

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin(safe_view.as_ref())
        .assert()
        .success();

    let result = fs::read_to_string(&file).unwrap();
    assert_eq!(result, CONTENT_WITH_BLOCK_AND_SECRET);
}

// ─── Edit mode: edge cases ───

#[test]
fn edit_multiline_old_string() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, "Line 1\nLine 2\nLine 3\n").unwrap();

    Command::cargo_bin("safe-write")
        .unwrap()
        .args([
            "edit",
            file.to_str().unwrap(),
            "--old",
            "Line 1\nLine 2",
            "--new",
            "Modified 1\nModified 2",
        ])
        .assert()
        .success();

    let result = fs::read_to_string(&file).unwrap();
    assert_eq!(result, "Modified 1\nModified 2\nLine 3\n");
}

#[test]
fn edit_preserves_secrets_in_file() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, CONTENT_WITH_SECRET).unwrap();

    Command::cargo_bin("safe-write")
        .unwrap()
        .args([
            "edit",
            file.to_str().unwrap(),
            "--old",
            "Normal line.",
            "--new",
            "Changed line.",
        ])
        .assert()
        .success();

    let result = fs::read_to_string(&file).unwrap();
    assert!(result.contains("sk-ant-api03-AAAAAAAAAAAAAAAAAAAAAAAAAAAA"));
    assert!(result.contains("Changed line."));
}

#[test]
fn edit_nonexistent_file() {
    Command::cargo_bin("safe-write")
        .unwrap()
        .args([
            "edit",
            "/tmp/definitely-does-not-exist-12345.md",
            "--old",
            "x",
            "--new",
            "y",
        ])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Cannot read"));
}

#[test]
fn edit_missing_flags() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, "content").unwrap();

    // Missing --new
    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["edit", file.to_str().unwrap(), "--old", "content"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("--new is required"));

    // Missing --old
    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["edit", file.to_str().unwrap(), "--new", "replacement"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("--old is required"));
}

#[test]
fn edit_unknown_flag() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, "content").unwrap();

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["edit", file.to_str().unwrap(), "--bad", "value"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Unknown flag"));
}

#[test]
fn unknown_mode_exits_1() {
    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["delete", "/tmp/file.md"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Unknown mode"));
}

// ─── Write mode: secret in same line as other text ───

#[test]
fn write_restores_secret_preserving_surrounding_text() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    let original = "Config: token=sk-ant-api03-AAAAAAAAAAAAAAAAAAAAAA endpoint=prod\n";
    fs::write(&file, original).unwrap();

    let new_content = "Config: token=[SECRET REDACTED] endpoint=staging\n";

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin(new_content)
        .assert()
        .success();

    let result = fs::read_to_string(&file).unwrap();
    assert!(result.contains("sk-ant-api03-AAAAAAAAAAAAAAAAAAAAAA"));
    assert!(result.contains("endpoint=staging"));
}

// ─── Write mode: spurious [SECRET REDACTED] in plain file ───

#[test]
fn write_fails_on_extra_secret_marker_in_plain_file() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(&file, "Normal content.\n").unwrap();

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin("Has [SECRET REDACTED] spuriously.\n")
        .assert()
        .code(1)
        .stderr(predicate::str::contains("no hidden content"));
}

// ─── Write mode: file outside vault ───

#[test]
fn write_works_on_file_outside_vault() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("standalone.md");
    fs::write(&file, "Original.\n").unwrap();

    Command::cargo_bin("safe-write")
        .unwrap()
        .args(["write", file.to_str().unwrap()])
        .write_stdin("Replaced.\n")
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&file).unwrap(), "Replaced.\n");
}

#[test]
fn edit_works_on_file_outside_vault() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("standalone.md");
    fs::write(&file, "Hello world.\n").unwrap();

    Command::cargo_bin("safe-write")
        .unwrap()
        .args([
            "edit",
            file.to_str().unwrap(),
            "--old",
            "Hello",
            "--new",
            "Goodbye",
        ])
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&file).unwrap(), "Goodbye world.\n");
}
