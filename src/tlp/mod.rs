use crate::frontmatter;
use crate::vault;
use std::fs;
use std::path::Path;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Tlp {
    Red,
    Amber,
    Green,
    Clear,
}

/// Result of classifying a file's TLP level.
pub struct Classification {
    pub level: Tlp,
    pub rel_path: String,
    pub config_error: bool,
}

/// Classify a file's TLP level using vault config + frontmatter override.
/// Returns None if no vault is found (file outside any vault).
/// If .tlp exists but can't be read, returns RED with `config_error` = true.
pub fn classify_file(file_path: &str) -> Option<Classification> {
    let vault_root = vault::find_vault(file_path)?;
    let abs_path = Path::new(file_path);
    let rel_path = abs_path.strip_prefix(&vault_root).ok()?;
    let rel_str = rel_path.to_string_lossy().to_string();

    let config_path = vault_root.join(".tlp");
    let Ok(config) = fs::read_to_string(&config_path) else {
        return Some(Classification {
            level: Tlp::Red,
            rel_path: rel_str,
            config_error: true,
        });
    };

    let path_level = classify(rel_path, &config);

    // Check frontmatter override: take the more restrictive of path and frontmatter
    let level = match fs::read_to_string(abs_path) {
        Ok(content) => {
            if let Some(val) = frontmatter::get_value(&content, "tlp") {
                if let Some(fm_level) = from_str(&val) {
                    most_restrictive(path_level, fm_level)
                } else {
                    path_level
                }
            } else {
                path_level
            }
        }
        Err(_) => path_level,
    };

    Some(Classification {
        level,
        rel_path: rel_str,
        config_error: false,
    })
}

/// Parse .tlp config and classify a relative path. First match wins.
/// Returns Amber for files not matched by any pattern.
pub fn classify(rel_path: &Path, config: &str) -> Tlp {
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
            if let Some(level) = current_level {
                if matches_pattern(&rel_str, pattern) {
                    return level;
                }
            }
        }
    }

    Tlp::Amber
}

/// Return the more restrictive of two TLP levels.
/// RED > AMBER > GREEN > CLEAR.
pub fn most_restrictive(a: Tlp, b: Tlp) -> Tlp {
    let rank = |t: Tlp| -> u8 {
        match t {
            Tlp::Red => 3,
            Tlp::Amber => 2,
            Tlp::Green => 1,
            Tlp::Clear => 0,
        }
    };
    if rank(a) >= rank(b) {
        a
    } else {
        b
    }
}

/// Parse a TLP level from a string (case-insensitive).
pub fn from_str(s: &str) -> Option<Tlp> {
    match s.trim().to_uppercase().as_str() {
        "RED" => Some(Tlp::Red),
        "AMBER" => Some(Tlp::Amber),
        "GREEN" => Some(Tlp::Green),
        "CLEAR" => Some(Tlp::Clear),
        _ => None,
    }
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
pub fn matches_pattern(path: &str, pattern: &str) -> bool {
    if pattern.starts_with('*') && !pattern.contains('/') {
        let suffix = &pattern[1..];
        return path.ends_with(suffix);
    }

    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path.starts_with(prefix)
            && (path.len() == prefix.len() || path.as_bytes()[prefix.len()] == b'/');
    }

    path == pattern
}

#[cfg(test)]
mod tests;
