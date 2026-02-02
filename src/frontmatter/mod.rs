use serde_yaml::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// Extract the YAML block between `---` delimiters and the remaining body.
/// Returns `(yaml_text, body)` or `None` if no frontmatter found.
fn split_frontmatter(content: &str) -> Option<(&str, &str)> {
    if !content.starts_with("---") {
        return None;
    }

    // Find the closing ---
    let after_first = &content[3..];
    let after_first = after_first.strip_prefix('\n').unwrap_or(after_first);

    let end = after_first.find("\n---")?;
    let yaml = &after_first[..end];
    let rest = &after_first[end + 4..]; // skip past \n---
    let body = rest.strip_prefix('\n').unwrap_or(rest);

    Some((yaml, body))
}

/// Extract a value from YAML frontmatter. Returns None if key not found.
pub fn get_value(content: &str, key: &str) -> Option<String> {
    let (yaml_text, _) = split_frontmatter(content)?;
    let value: Value = serde_yaml::from_str(yaml_text).ok()?;
    let mapping = value.as_mapping()?;
    let key_value = mapping.get(Value::String(key.to_string()))?;

    match key_value {
        Value::String(s) => Some(s.clone()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        _ => Some(serde_yaml::to_string(key_value).ok()?.trim().to_string()),
    }
}

/// Set a frontmatter key. Creates frontmatter if missing, updates if exists.
pub fn set_value(content: &str, key: &str, value: &str) -> String {
    let yaml_key = Value::String(key.to_string());
    let yaml_value = Value::String(value.to_string());

    if let Some((yaml_text, body)) = split_frontmatter(content) {
        let mut mapping = match serde_yaml::from_str::<Value>(yaml_text) {
            Ok(Value::Mapping(m)) => m,
            _ => serde_yaml::Mapping::new(),
        };

        mapping.insert(yaml_key, yaml_value);

        let serialized = serde_yaml::to_string(&Value::Mapping(mapping)).unwrap_or_default();

        if body.is_empty() {
            format!("---\n{serialized}---")
        } else {
            format!("---\n{serialized}---\n{body}")
        }
    } else {
        let mut mapping = serde_yaml::Mapping::new();
        mapping.insert(yaml_key, yaml_value);
        let serialized = serde_yaml::to_string(&Value::Mapping(mapping)).unwrap_or_default();
        format!("---\n{serialized}---\n\n{content}")
    }
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
mod tests;
