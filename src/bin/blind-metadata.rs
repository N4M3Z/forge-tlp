use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage:");
        eprintln!("  blind-metadata set <directory> <key> <value>");
        eprintln!("  blind-metadata get <directory> <key>");
        eprintln!("  blind-metadata has <directory> <key>");
        std::process::exit(1);
    }

    let action = &args[1];
    let dir = &args[2];
    let key = &args[3];
    let value = args.get(4).map(|s| s.as_str());

    // Resolve directory: if relative, look for vault by walking up from cwd
    let target = if Path::new(dir).is_absolute() {
        dir.to_string()
    } else {
        match find_vault_from_cwd() {
            Some(vault) => format!("{}/{}", vault.display(), dir),
            None => {
                eprintln!("Cannot find vault root (no .tlp file in parent directories)");
                std::process::exit(1);
            }
        }
    };

    let target_path = Path::new(&target);
    if !target_path.is_dir() {
        eprintln!("Directory not found: {}", dir);
        std::process::exit(1);
    }

    match action.as_str() {
        "set" => cmd_set(target_path, key, value),
        "get" => cmd_get(target_path, key),
        "has" => cmd_has(target_path, key),
        _ => {
            eprintln!("Unknown action: {} (use set, get, or has)", action);
            std::process::exit(1);
        }
    }
}

fn cmd_set(dir: &Path, key: &str, value: Option<&str>) {
    let value = match value {
        Some(v) => v,
        None => {
            eprintln!("set requires a value");
            std::process::exit(1);
        }
    };

    let (mut count, mut total) = (0usize, 0usize);

    for entry in read_md_files(dir) {
        total += 1;
        let name = entry.file_name().unwrap_or_default().to_string_lossy().to_string();
        let content = match fs::read_to_string(&entry) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let new_content = set_frontmatter_key(&content, key, value);
        if let Err(e) = fs::write(&entry, &new_content) {
            eprintln!("  error:   {} ({})", name, e);
            continue;
        }

        if new_content != content {
            println!("  updated: {}", name);
        } else {
            println!("  ok:      {}", name);
        }
        count += 1;
    }

    println!();
    println!("Done: {}/{} files processed with {}: {}", count, total, key, value);
}

fn cmd_get(dir: &Path, key: &str) {
    let (mut count, mut total) = (0usize, 0usize);

    for entry in read_md_files(dir) {
        total += 1;
        let name = entry.file_stem().unwrap_or_default().to_string_lossy().to_string();
        let content = match fs::read_to_string(&entry) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if let Some(val) = get_frontmatter_value(&content, key) {
            println!("  {}: {}", name, val);
            count += 1;
        }
    }

    println!();
    println!("{}/{} files have {} set", count, total, key);
}

fn cmd_has(dir: &Path, key: &str) {
    let (mut missing, mut total) = (0usize, 0usize);

    println!("Files missing {}:", key);

    for entry in read_md_files(dir) {
        total += 1;
        let name = entry.file_name().unwrap_or_default().to_string_lossy().to_string();
        let content = match fs::read_to_string(&entry) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if get_frontmatter_value(&content, key).is_none() {
            println!("  {}", name);
            missing += 1;
        }
    }

    println!();
    println!("{}/{} files missing {}", missing, total, key);
}

/// List .md files in a directory (non-recursive).
fn read_md_files(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "md") && path.is_file() {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

/// Walk up from cwd looking for a directory containing .tlp
fn find_vault_from_cwd() -> Option<std::path::PathBuf> {
    let cwd = env::current_dir().ok()?;
    let mut dir = cwd.as_path();
    loop {
        if dir.join(".tlp").exists() {
            return Some(dir.to_path_buf());
        }
        dir = dir.parent()?;
    }
}

/// Extract a value from YAML frontmatter. Returns None if key not found.
fn get_frontmatter_value(content: &str, key: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.first().map_or(true, |l| *l != "---") {
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
fn set_frontmatter_key(content: &str, key: &str, value: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let new_line = format!("{}: {}", key, value);

    // No frontmatter — prepend it
    if lines.first().map_or(true, |l| *l != "---") {
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
        Some(i) => result[i] = new_line, // update existing
        None => result.insert(end_idx, new_line), // add before closing ---
    }

    result.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_value() {
        let content = "---\ntitle: Hello\ntlp: RED\n---\nBody";
        assert_eq!(get_frontmatter_value(content, "tlp"), Some("RED".into()));
        assert_eq!(get_frontmatter_value(content, "title"), Some("Hello".into()));
        assert_eq!(get_frontmatter_value(content, "missing"), None);
    }

    #[test]
    fn test_get_no_frontmatter() {
        let content = "Just a plain file";
        assert_eq!(get_frontmatter_value(content, "tlp"), None);
    }

    #[test]
    fn test_set_existing_key() {
        let content = "---\ntitle: Hello\ntlp: GREEN\n---\nBody";
        let result = set_frontmatter_key(content, "tlp", "RED");
        assert!(result.contains("tlp: RED"));
        assert!(!result.contains("tlp: GREEN"));
    }

    #[test]
    fn test_set_new_key() {
        let content = "---\ntitle: Hello\n---\nBody";
        let result = set_frontmatter_key(content, "tlp", "RED");
        assert!(result.contains("tlp: RED"));
        assert!(result.contains("title: Hello"));
    }

    #[test]
    fn test_set_no_frontmatter() {
        let content = "Just a plain file";
        let result = set_frontmatter_key(content, "tlp", "RED");
        assert!(result.starts_with("---\ntlp: RED\n---"));
        assert!(result.contains("Just a plain file"));
    }
}
