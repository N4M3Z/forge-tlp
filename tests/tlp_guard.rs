#![allow(deprecated)] // Command::cargo_bin is the standard assert_cmd API

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

// ─── Fixture constants ───

const CONFIG_RED_PDF: &str = include_str!("fixtures/configs/red_pdf.tlp");
const CONFIG_RED_CONTACTS: &str = include_str!("fixtures/configs/red_contacts.tlp");
const CONFIG_AMBER_JOURNALS: &str = include_str!("fixtures/configs/amber_journals.tlp");
const CONFIG_GREEN_TOPICS: &str = include_str!("fixtures/configs/green_topics.tlp");
const CONFIG_CLEAR_README: &str = include_str!("fixtures/configs/clear_readme.tlp");
const CONFIG_FIRST_MATCH_WINS: &str = include_str!("fixtures/configs/first_match_wins.tlp");

const CONTENT_FRONTMATTER_RED: &str = include_str!("fixtures/content/frontmatter_red.md");
const CONTENT_FRONTMATTER_GREEN: &str = include_str!("fixtures/content/frontmatter_green.md");
const CONTENT_FRONTMATTER_AMBER: &str = include_str!("fixtures/content/frontmatter_amber.md");
const CONTENT_FRONTMATTER_INVALID: &str = include_str!("fixtures/content/frontmatter_invalid.md");

// ─── Helpers ───

fn hook_input(tool_name: &str, file_path: &str) -> String {
    format!(r#"{{"tool_name":"{tool_name}","tool_input":{{"file_path":"{file_path}"}}}}"#)
}

struct TestVault {
    dir: tempfile::TempDir,
}

impl TestVault {
    fn new(tlp_config: &str) -> Self {
        let dir = tempdir().expect("create tempdir");
        fs::write(dir.path().join(".tlp"), tlp_config).expect("write .tlp");
        TestVault { dir }
    }

    fn path(&self) -> &std::path::Path {
        self.dir.path()
    }

    fn create_file(&self, rel_path: &str, content: &str) {
        let full = self.path().join(rel_path);
        if let Some(parent) = full.parent() {
            fs::create_dir_all(parent).expect("create parent dirs");
        }
        fs::write(&full, content).expect("write file");
    }

    fn abs(&self, rel_path: &str) -> String {
        self.path().join(rel_path).to_string_lossy().to_string()
    }
}

// ─── RED file tests ───

#[test]
fn red_file_blocks_read() {
    let vault = TestVault::new(CONFIG_RED_PDF);
    vault.create_file("secret.pdf", "binary");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("secret.pdf")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:RED"));
}

#[test]
fn red_file_blocks_edit() {
    let vault = TestVault::new(CONFIG_RED_PDF);
    vault.create_file("secret.pdf", "binary");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Edit", &vault.abs("secret.pdf")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:RED"));
}

#[test]
fn red_file_blocks_write() {
    let vault = TestVault::new(CONFIG_RED_PDF);
    vault.create_file("secret.pdf", "binary");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Write", &vault.abs("secret.pdf")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:RED"));
}

#[test]
fn red_dir_blocks_read() {
    let vault = TestVault::new(CONFIG_RED_CONTACTS);
    vault.create_file("Contacts/john.md", "phone");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Contacts/john.md")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:RED"));
}

// ─── RED: new file creation ───

#[test]
fn red_dir_allows_write_new_file() {
    let vault = TestVault::new(CONFIG_RED_CONTACTS);
    // Don't create the file — it shouldn't exist
    fs::create_dir_all(vault.path().join("Contacts")).expect("create dir");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Write", &vault.abs("Contacts/new_person.md")))
        .assert()
        .success()
        .stdout(predicate::str::contains("new file creation allowed"));
}

#[test]
fn red_dir_still_blocks_read_nonexistent_file() {
    let vault = TestVault::new(CONFIG_RED_CONTACTS);
    fs::create_dir_all(vault.path().join("Contacts")).expect("create dir");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Contacts/ghost.md")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:RED"));
}

#[test]
fn red_dir_still_blocks_edit_nonexistent_file() {
    let vault = TestVault::new(CONFIG_RED_CONTACTS);
    fs::create_dir_all(vault.path().join("Contacts")).expect("create dir");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Edit", &vault.abs("Contacts/ghost.md")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:RED"));
}

// ─── AMBER file tests ───

#[test]
fn amber_file_blocks_read_suggests_safe_read() {
    let vault = TestVault::new(CONFIG_AMBER_JOURNALS);
    vault.create_file("Journals/today.md", "diary entry");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Journals/today.md")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("safe-read"));
}

#[test]
fn amber_file_allows_edit() {
    let vault = TestVault::new(CONFIG_AMBER_JOURNALS);
    vault.create_file("Journals/today.md", "diary entry");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Edit", &vault.abs("Journals/today.md")))
        .assert()
        .success()
        .stdout(predicate::str::contains("TLP:AMBER"));
}

#[test]
fn amber_file_allows_write() {
    let vault = TestVault::new(CONFIG_AMBER_JOURNALS);
    vault.create_file("Journals/today.md", "diary entry");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Write", &vault.abs("Journals/today.md")))
        .assert()
        .success()
        .stdout(predicate::str::contains("TLP:AMBER"));
}

// ─── GREEN/CLEAR file tests ───

#[test]
fn green_file_allows_read() {
    let vault = TestVault::new(CONFIG_GREEN_TOPICS);
    vault.create_file("Topics/rust.md", "rust notes");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Topics/rust.md")))
        .assert()
        .success();
}

#[test]
fn clear_file_allows_read() {
    let vault = TestVault::new(CONFIG_CLEAR_README);
    vault.create_file("README.md", "hello");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("README.md")))
        .assert()
        .success();
}

#[test]
fn green_file_allows_edit() {
    let vault = TestVault::new(CONFIG_GREEN_TOPICS);
    vault.create_file("Topics/rust.md", "rust notes");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Edit", &vault.abs("Topics/rust.md")))
        .assert()
        .success();
}

// ─── Default AMBER for unmatched files ───

#[test]
fn unmatched_file_defaults_to_amber_blocks_read() {
    let vault = TestVault::new(CONFIG_GREEN_TOPICS);
    vault.create_file("random/notes.md", "stuff");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("random/notes.md")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:AMBER"));
}

// ─── Edge cases ───

#[test]
fn no_file_path_in_json_allows() {
    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(r#"{"tool_name":"Read","tool_input":{}}"#)
        .assert()
        .success();
}

#[test]
fn empty_file_path_allows() {
    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(r#"{"tool_name":"Read","tool_input":{"file_path":""}}"#)
        .assert()
        .success();
}

#[test]
fn invalid_json_allows() {
    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin("this is not json")
        .assert()
        .success();
}

#[test]
fn file_outside_any_vault_allows() {
    // /tmp has no .tlp file
    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", "/tmp/random_file.txt"))
        .assert()
        .success();
}

#[test]
fn first_match_wins() {
    let vault = TestVault::new(CONFIG_FIRST_MATCH_WINS);
    vault.create_file("Topics/rust.md", "rust notes");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Topics/rust.md")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:RED"));
}

// ─── Frontmatter override tests ───

#[test]
fn frontmatter_red_escalates_green_path() {
    let vault = TestVault::new(CONFIG_GREEN_TOPICS);
    vault.create_file("Topics/sensitive.md", CONTENT_FRONTMATTER_RED);

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Topics/sensitive.md")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:RED"));
}

#[test]
fn frontmatter_green_cannot_downgrade_amber_path() {
    let vault = TestVault::new(CONFIG_AMBER_JOURNALS);
    vault.create_file("Journals/today.md", CONTENT_FRONTMATTER_GREEN);

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Journals/today.md")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:AMBER"));
}

#[test]
fn frontmatter_amber_escalates_green_path() {
    let vault = TestVault::new(CONFIG_GREEN_TOPICS);
    vault.create_file("Topics/private.md", CONTENT_FRONTMATTER_AMBER);

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Topics/private.md")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:AMBER"));
}

#[test]
fn no_frontmatter_uses_path_level() {
    let vault = TestVault::new(CONFIG_GREEN_TOPICS);
    vault.create_file("Topics/normal.md", "No frontmatter here\n");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Topics/normal.md")))
        .assert()
        .success();
}

#[test]
fn invalid_frontmatter_tlp_ignored() {
    let vault = TestVault::new(CONFIG_GREEN_TOPICS);
    vault.create_file("Topics/weird.md", CONTENT_FRONTMATTER_INVALID);

    // Invalid TLP value in frontmatter is ignored — uses path level (GREEN)
    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Topics/weird.md")))
        .assert()
        .success();
}

// ─── Fail-closed tests ───

#[test]
fn unreadable_tlp_config_blocks() {
    // Create a vault where .tlp is a directory (can't be read as a file)
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".tlp")).unwrap();
    let file = dir.path().join("note.md");
    fs::write(&file, "content").unwrap();

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", file.to_str().unwrap()))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("Malformed .tlp config"));
}
