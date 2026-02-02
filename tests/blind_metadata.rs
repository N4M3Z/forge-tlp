use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ─── Basic usage ───

#[test]
fn no_args_exits_1_with_usage() {
    Command::cargo_bin("blind-metadata")
        .unwrap()
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Usage"));
}

#[test]
fn unknown_action_exits_1() {
    let dir = tempdir().unwrap();
    Command::cargo_bin("blind-metadata")
        .unwrap()
        .args(["delete", dir.path().to_str().unwrap(), "key"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Unknown action"));
}

// ─── Set command ───

#[test]
fn set_creates_frontmatter() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("note.md");
    fs::write(&file, "Hello body").unwrap();

    Command::cargo_bin("blind-metadata")
        .unwrap()
        .args(["set", dir.path().to_str().unwrap(), "tlp", "RED"])
        .assert()
        .success()
        .stdout(predicate::str::contains("updated: note.md"));

    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("tlp: RED"));
    assert!(content.contains("Hello body"));
}

#[test]
fn set_updates_existing_key() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("note.md");
    fs::write(&file, "---\ntlp: GREEN\n---\nBody").unwrap();

    Command::cargo_bin("blind-metadata")
        .unwrap()
        .args(["set", dir.path().to_str().unwrap(), "tlp", "RED"])
        .assert()
        .success();

    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("tlp: RED"));
    assert!(!content.contains("tlp: GREEN"));
}

#[test]
fn set_adds_new_key() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("note.md");
    fs::write(&file, "---\ntitle: Hello\n---\nBody").unwrap();

    Command::cargo_bin("blind-metadata")
        .unwrap()
        .args(["set", dir.path().to_str().unwrap(), "tlp", "AMBER"])
        .assert()
        .success();

    let content = fs::read_to_string(&file).unwrap();
    assert!(content.contains("tlp: AMBER"));
    assert!(content.contains("title: Hello"));
}

#[test]
fn set_without_value_exits_1() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("note.md"), "body").unwrap();

    Command::cargo_bin("blind-metadata")
        .unwrap()
        .args(["set", dir.path().to_str().unwrap(), "tlp"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("set requires a value"));
}

// ─── Get command ───

#[test]
fn get_shows_values() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("note.md"), "---\ntlp: RED\n---\nBody").unwrap();

    Command::cargo_bin("blind-metadata")
        .unwrap()
        .args(["get", dir.path().to_str().unwrap(), "tlp"])
        .assert()
        .success()
        .stdout(predicate::str::contains("RED"));
}

#[test]
fn get_missing_key_shows_zero() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("note.md"), "---\ntitle: Hi\n---\nBody").unwrap();

    Command::cargo_bin("blind-metadata")
        .unwrap()
        .args(["get", dir.path().to_str().unwrap(), "tlp"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0/1 files have tlp set"));
}

// ─── Has command ───

#[test]
fn has_lists_missing_files() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("a.md"), "---\ntitle: A\n---\n").unwrap();
    fs::write(dir.path().join("b.md"), "---\ntlp: RED\n---\n").unwrap();

    Command::cargo_bin("blind-metadata")
        .unwrap()
        .args(["has", dir.path().to_str().unwrap(), "tlp"])
        .assert()
        .success()
        .stdout(predicate::str::contains("a.md"))
        .stdout(predicate::str::contains("1/2 files missing tlp"));
}

// ─── Only processes .md files ───

#[test]
fn ignores_non_md_files() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join("note.md"), "---\ntlp: RED\n---\n").unwrap();
    fs::write(dir.path().join("data.json"), r#"{"key":"value"}"#).unwrap();

    Command::cargo_bin("blind-metadata")
        .unwrap()
        .args(["get", dir.path().to_str().unwrap(), "tlp"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1/1 files have tlp set"));
}

// ─── Non-existent directory ───

#[test]
fn nonexistent_dir_exits_1() {
    Command::cargo_bin("blind-metadata")
        .unwrap()
        .args(["get", "/tmp/nonexistent-dir-12345", "tlp"])
        .assert()
        .code(1)
        .stderr(predicate::str::contains("Directory not found"));
}
