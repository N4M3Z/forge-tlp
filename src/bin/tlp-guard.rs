use serde::Deserialize;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::fs;

/// JSON payload from Claude Code's PreToolUse hook.
#[derive(Deserialize)]
struct HookInput {
    tool_name: Option<String>,
    tool_input: Option<ToolInput>,
}

#[derive(Deserialize)]
struct ToolInput {
    file_path: Option<String>,
}

#[derive(Debug, PartialEq)]
enum Tlp {
    Red,
    Amber,
    Green,
    Clear,
    None,
}

fn main() -> ExitCode {
    let mut buf = String::new();
    if std::io::stdin().read_to_string(&mut buf).is_err() {
        return ExitCode::SUCCESS;
    }

    let input = match serde_json::from_str::<HookInput>(&buf) {
        Ok(i) => i,
        Err(_) => return ExitCode::SUCCESS,
    };

    let tool_name = input.tool_name.unwrap_or_default();
    let file_path = input
        .tool_input
        .and_then(|ti| ti.file_path)
        .unwrap_or_default();

    if file_path.is_empty() {
        return ExitCode::SUCCESS;
    }

    let vault = match find_vault(&file_path) {
        Some(v) => v,
        None => return ExitCode::SUCCESS,
    };

    let abs_path = Path::new(&file_path);
    let rel_path = match abs_path.strip_prefix(&vault) {
        Ok(r) => r,
        Err(_) => return ExitCode::SUCCESS,
    };

    let config_path = vault.join(".tlp");
    let config = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return ExitCode::SUCCESS,
    };

    let level = find_tlp(rel_path, &config);

    match level {
        Tlp::Red => {
            eprintln!("TLP:RED — access blocked for: {}", rel_path.display());
            ExitCode::from(2)
        }
        Tlp::Amber => {
            if tool_name == "Read" {
                let safe_read = find_sibling_binary("safe-read")
                    .unwrap_or_else(|| "safe-read".to_string());
                eprintln!(
                    "TLP:AMBER — this file requires approval. Ask the user, then use:\n\
                     {} \"{}\"",
                    safe_read, file_path
                );
                ExitCode::from(2)
            } else {
                println!(
                    "TLP:AMBER — editing allowed, but never output content verbatim from: {}",
                    rel_path.display()
                );
                ExitCode::SUCCESS
            }
        }
        _ => ExitCode::SUCCESS,
    }
}

/// Find a sibling binary next to the current executable.
fn find_sibling_binary(name: &str) -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    let sibling = dir.join(name);
    if sibling.exists() {
        Some(sibling.to_string_lossy().to_string())
    } else {
        None
    }
}

/// Walk up from the file path looking for a directory containing .tlp
fn find_vault(file_path: &str) -> Option<PathBuf> {
    let mut dir = Path::new(file_path).parent()?;
    loop {
        if dir.join(".tlp").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// Parse .tlp config and find classification for a path. First match wins.
fn find_tlp(rel_path: &Path, config: &str) -> Tlp {
    let rel_str = rel_path.to_string_lossy();
    let mut current_level: Option<Tlp> = None;

    for line in config.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(level) = parse_level_header(trimmed) {
            current_level = Some(level);
            continue;
        }

        if let Some(pattern) = parse_pattern_line(trimmed) {
            if let Some(ref level) = current_level {
                if matches_pattern(&rel_str, pattern) {
                    return match level {
                        Tlp::Red => Tlp::Red,
                        Tlp::Amber => Tlp::Amber,
                        Tlp::Green => Tlp::Green,
                        Tlp::Clear => Tlp::Clear,
                        Tlp::None => Tlp::None,
                    };
                }
            }
        }
    }

    Tlp::Amber
}

fn parse_level_header(line: &str) -> Option<Tlp> {
    match line {
        "RED:" => Some(Tlp::Red),
        "AMBER:" => Some(Tlp::Amber),
        "GREEN:" => Some(Tlp::Green),
        "CLEAR:" => Some(Tlp::Clear),
        _ => None,
    }
}

fn parse_pattern_line(line: &str) -> Option<&str> {
    let stripped = line.trim_start_matches(|c: char| c == '-' || c.is_whitespace());
    if stripped.starts_with('"') && stripped.ends_with('"') && stripped.len() >= 2 {
        Some(&stripped[1..stripped.len() - 1])
    } else {
        None
    }
}

/// Match a relative path against a glob pattern.
/// Supports: *.ext (extension anywhere), dir/** (prefix), exact match.
fn matches_pattern(path: &str, pattern: &str) -> bool {
    if pattern.starts_with('*') && !pattern.contains('/') {
        let suffix = &pattern[1..];
        return path.ends_with(suffix);
    }

    if pattern.ends_with("/**") {
        let prefix = &pattern[..pattern.len() - 3];
        return path.starts_with(prefix)
            && (path.len() == prefix.len() || path.as_bytes()[prefix.len()] == b'/');
    }

    path == pattern
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_extension_match() {
        assert!(matches_pattern("foo/bar.pdf", "*.pdf"));
        assert!(matches_pattern("deep/nested/file.xlsx", "*.xlsx"));
        assert!(!matches_pattern("foo/bar.txt", "*.pdf"));
    }

    #[test]
    fn test_dir_match() {
        assert!(matches_pattern("Resources/Contacts/john.md", "Resources/Contacts/**"));
        assert!(matches_pattern("Resources/Contacts/sub/deep.md", "Resources/Contacts/**"));
        assert!(!matches_pattern("Resources/ContactsExtra/john.md", "Resources/Contacts/**"));
    }

    #[test]
    fn test_exact_match() {
        assert!(matches_pattern("AI/Identity.md", "AI/Identity.md"));
        assert!(!matches_pattern("AI/Identity.md.bak", "AI/Identity.md"));
    }

    #[test]
    fn test_find_tlp() {
        let config = r#"
RED:
  - "*.pdf"
  - "Resources/Contacts/**"

AMBER:
  - "AI/Identity.md"
  - "Pipeline/**"

GREEN:
  - "Topics/**"
"#;
        assert_eq!(find_tlp(Path::new("foo.pdf"), config), Tlp::Red);
        assert_eq!(find_tlp(Path::new("Resources/Contacts/john.md"), config), Tlp::Red);
        assert_eq!(find_tlp(Path::new("AI/Identity.md"), config), Tlp::Amber);
        assert_eq!(find_tlp(Path::new("Pipeline/Fleeting/note.md"), config), Tlp::Amber);
        assert_eq!(find_tlp(Path::new("Topics/rust.md"), config), Tlp::Green);
        assert_eq!(find_tlp(Path::new("random/file.md"), config), Tlp::Amber);
    }
}
