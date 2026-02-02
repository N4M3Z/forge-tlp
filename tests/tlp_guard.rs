use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

/// Helper: build a JSON hook input string for tlp-guard.
fn hook_input(tool_name: &str, file_path: &str) -> String {
    format!(
        r#"{{"tool_name":"{}","tool_input":{{"file_path":"{}"}}}}"#,
        tool_name, file_path
    )
}

/// Helper: create a temp vault with a .tlp config and optional files.
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
    let vault = TestVault::new("RED:\n  - \"*.pdf\"\n");
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
    let vault = TestVault::new("RED:\n  - \"*.pdf\"\n");
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
    let vault = TestVault::new("RED:\n  - \"*.pdf\"\n");
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
    let vault = TestVault::new("RED:\n  - \"Contacts/**\"\n");
    vault.create_file("Contacts/john.md", "phone");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Contacts/john.md")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:RED"));
}

// ─── AMBER file tests ───

#[test]
fn amber_file_blocks_read_suggests_safe_read() {
    let vault = TestVault::new("AMBER:\n  - \"Journals/**\"\n");
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
    let vault = TestVault::new("AMBER:\n  - \"Journals/**\"\n");
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
    let vault = TestVault::new("AMBER:\n  - \"Journals/**\"\n");
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
    let vault = TestVault::new("GREEN:\n  - \"Topics/**\"\n");
    vault.create_file("Topics/rust.md", "rust notes");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Topics/rust.md")))
        .assert()
        .success();
}

#[test]
fn clear_file_allows_read() {
    let vault = TestVault::new("CLEAR:\n  - \"README.md\"\n");
    vault.create_file("README.md", "hello");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("README.md")))
        .assert()
        .success();
}

#[test]
fn green_file_allows_edit() {
    let vault = TestVault::new("GREEN:\n  - \"Topics/**\"\n");
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
    let vault = TestVault::new("GREEN:\n  - \"Topics/**\"\n");
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
    // File matches RED before GREEN
    let vault = TestVault::new("RED:\n  - \"*.md\"\n\nGREEN:\n  - \"Topics/**\"\n");
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
    let vault = TestVault::new("GREEN:\n  - \"Topics/**\"\n");
    vault.create_file(
        "Topics/sensitive.md",
        "---\ntlp: RED\n---\nThis should be blocked\n",
    );

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Topics/sensitive.md")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:RED"));
}

#[test]
fn frontmatter_green_cannot_downgrade_amber_path() {
    let vault = TestVault::new("AMBER:\n  - \"Journals/**\"\n");
    vault.create_file("Journals/today.md", "---\ntlp: GREEN\n---\nStill AMBER\n");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Journals/today.md")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:AMBER"));
}

#[test]
fn frontmatter_amber_escalates_green_path() {
    let vault = TestVault::new("GREEN:\n  - \"Topics/**\"\n");
    vault.create_file(
        "Topics/private.md",
        "---\ntlp: AMBER\n---\nNeeds approval\n",
    );

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Topics/private.md")))
        .assert()
        .code(2)
        .stderr(predicate::str::contains("TLP:AMBER"));
}

#[test]
fn no_frontmatter_uses_path_level() {
    let vault = TestVault::new("GREEN:\n  - \"Topics/**\"\n");
    vault.create_file("Topics/normal.md", "No frontmatter here\n");

    Command::cargo_bin("tlp-guard")
        .unwrap()
        .write_stdin(hook_input("Read", &vault.abs("Topics/normal.md")))
        .assert()
        .success();
}

#[test]
fn invalid_frontmatter_tlp_ignored() {
    let vault = TestVault::new("GREEN:\n  - \"Topics/**\"\n");
    vault.create_file("Topics/weird.md", "---\ntlp: PURPLE\n---\nBad value\n");

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
