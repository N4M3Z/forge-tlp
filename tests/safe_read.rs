use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

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
    fs::write(&file, "Hello, world!\nLine two.\n").unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout("Hello, world!\nLine two.\n");
}

// ─── TLP redaction ───

#[test]
fn redacts_tlp_red_section() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(
        &file,
        "Public\n#tlp/red\nSecret line 1\nSecret line 2\n#tlp/amber\nMore public\n",
    )
    .unwrap();

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
    fs::write(
        &file,
        "A\n#tlp/red\nX\n#tlp/amber\nB\n#tlp/red\nY\n#tlp/green\nC\n",
    )
    .unwrap();

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
    fs::write(&file, "Before\n#tlp/red\nSecret to EOF\n").unwrap();

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
    fs::write(&file, "key: sk-ant-api03-AAAAAAAAAAAAAAAAAAAAAA\n").unwrap();

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
    fs::write(&file, "GITLAB_TOKEN=glpat-ABCDEFGHIJKLMNOPQRST\n").unwrap();

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
    fs::write(&file, "I want to sk-ip this line\n").unwrap();

    Command::cargo_bin("safe-read")
        .unwrap()
        .arg(file.to_str().unwrap())
        .assert()
        .success()
        .stdout("I want to sk-ip this line\n");
}

// ─── Combined TLP + secret redaction ───

#[test]
fn full_pipeline_redacts_both() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("test.md");
    fs::write(
        &file,
        "public\n#tlp/red\ntop secret\n#tlp/amber\nkey: sk-ant-api03-REALKEY12345678901234\n",
    )
    .unwrap();

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
    fs::write(dir.path().join(".tlp"), "RED:\n  - \"*.pdf\"\n").unwrap();
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
    fs::write(dir.path().join(".tlp"), "GREEN:\n  - \"*.md\"\n").unwrap();
    let file = dir.path().join("secret.md");
    fs::write(&file, "---\ntlp: RED\n---\nTop secret\n").unwrap();

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
    fs::write(dir.path().join(".tlp"), "AMBER:\n  - \"*.md\"\n").unwrap();
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
