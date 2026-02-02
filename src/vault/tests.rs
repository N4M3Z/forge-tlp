use super::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_find_vault_basic() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".tlp"), "GREEN:\n  - \"*.md\"\n").unwrap();
    fs::create_dir_all(dir.path().join("sub/deep")).unwrap();
    let file = dir.path().join("sub/deep/note.md");
    fs::write(&file, "content").unwrap();

    let vault = find_vault(file.to_str().unwrap());
    assert_eq!(vault.unwrap(), dir.path());
}

#[test]
fn test_find_vault_immediate_parent() {
    let dir = tempdir().unwrap();
    fs::write(dir.path().join(".tlp"), "").unwrap();
    let file = dir.path().join("note.md");
    fs::write(&file, "content").unwrap();

    let vault = find_vault(file.to_str().unwrap());
    assert_eq!(vault.unwrap(), dir.path());
}

#[test]
fn test_find_vault_no_tlp_returns_none() {
    let dir = tempdir().unwrap();
    let file = dir.path().join("note.md");
    fs::write(&file, "content").unwrap();

    // No .tlp anywhere in the temp dir hierarchy
    let vault = find_vault(file.to_str().unwrap());
    assert!(vault.is_none());
}
