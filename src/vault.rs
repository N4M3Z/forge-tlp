use std::path::{Path, PathBuf};

/// Walk up from a starting directory looking for .tlp.
fn find_vault_from_dir(start: &Path) -> Option<PathBuf> {
    let mut dir = start;
    loop {
        if dir.join(".tlp").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// Walk up from the file's parent directory looking for .tlp.
pub fn find_vault(file_path: &str) -> Option<PathBuf> {
    let parent = Path::new(file_path).parent()?;
    find_vault_from_dir(parent)
}

/// Walk up from the current working directory looking for .tlp.
pub fn find_vault_from_cwd() -> Option<PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    find_vault_from_dir(&cwd)
}

#[cfg(test)]
mod tests {
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
}
