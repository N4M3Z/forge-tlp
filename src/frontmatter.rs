use std::fs;
use std::path::{Path, PathBuf};

/// Extract a value from YAML frontmatter. Returns None if key not found.
pub fn get_value(content: &str, key: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.first().is_none_or(|l| *l != "---") {
        return None;
    }
    let prefix = format!("{}: ", key);
    for line in lines.iter().skip(1) {
        if *line == "---" {
            break;
        }
        if let Some(rest) = line.strip_prefix(&prefix) {
            return Some(rest.to_string());
        }
    }
    None
}

/// Set a frontmatter key. Creates frontmatter if missing, updates if exists.
pub fn set_value(content: &str, key: &str, value: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let new_line = format!("{}: {}", key, value);

    // No frontmatter — prepend it
    if lines.first().is_none_or(|l| *l != "---") {
        return format!("---\n{}\n---\n\n{}", new_line, content);
    }

    // Find end of frontmatter
    let mut end_idx = None;
    let mut key_idx = None;
    let prefix = format!("{}:", key);

    for (i, line) in lines.iter().enumerate().skip(1) {
        if *line == "---" {
            end_idx = Some(i);
            break;
        }
        if line.starts_with(&prefix) {
            key_idx = Some(i);
        }
    }

    let end_idx = match end_idx {
        Some(i) => i,
        None => return format!("---\n{}\n---\n\n{}", new_line, content),
    };

    let mut result: Vec<String> = lines.iter().map(|l| l.to_string()).collect();

    match key_idx {
        Some(i) => result[i] = new_line,          // update existing
        None => result.insert(end_idx, new_line), // add before closing ---
    }

    result.join("\n")
}

/// List .md files in a directory (non-recursive), sorted by name.
pub fn read_md_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") && path.is_file() {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_value_basic() {
        let content = "---\ntitle: Hello\ntlp: RED\n---\nBody";
        assert_eq!(get_value(content, "tlp"), Some("RED".into()));
        assert_eq!(get_value(content, "title"), Some("Hello".into()));
        assert_eq!(get_value(content, "missing"), None);
    }

    #[test]
    fn test_get_no_frontmatter() {
        assert_eq!(get_value("Just a plain file", "tlp"), None);
    }

    #[test]
    fn test_set_existing_key() {
        let content = "---\ntitle: Hello\ntlp: GREEN\n---\nBody";
        let result = set_value(content, "tlp", "RED");
        assert!(result.contains("tlp: RED"));
        assert!(!result.contains("tlp: GREEN"));
    }

    #[test]
    fn test_set_new_key() {
        let content = "---\ntitle: Hello\n---\nBody";
        let result = set_value(content, "tlp", "RED");
        assert!(result.contains("tlp: RED"));
        assert!(result.contains("title: Hello"));
    }

    #[test]
    fn test_set_no_frontmatter() {
        let content = "Just a plain file";
        let result = set_value(content, "tlp", "RED");
        assert!(result.starts_with("---\ntlp: RED\n---"));
        assert!(result.contains("Just a plain file"));
    }

    #[test]
    fn test_colon_in_value() {
        let content = "---\nurl: https://example.com\n---\n";
        assert_eq!(
            get_value(content, "url"),
            Some("https://example.com".into())
        );
    }

    #[test]
    fn test_trailing_newline_preserved() {
        let content = "---\ntitle: Hello\n---\nBody";
        let result = set_value(content, "tlp", "RED");
        // Should not add extra newlines
        assert!(!result.contains("\n\n\n"));
    }

    #[test]
    fn test_empty_content_set() {
        let result = set_value("", "tlp", "RED");
        assert!(result.contains("tlp: RED"));
    }
}
