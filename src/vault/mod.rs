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
mod tests;
