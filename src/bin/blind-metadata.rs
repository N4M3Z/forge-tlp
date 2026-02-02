use context_tlp::frontmatter;
use context_tlp::vault;
use std::env;
use std::fs;
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage:");
        eprintln!("  blind-metadata set <directory> <key> <value>");
        eprintln!("  blind-metadata get <directory> <key>");
        eprintln!("  blind-metadata has <directory> <key>");
        return ExitCode::from(1);
    }

    let action = &args[1];
    let dir = &args[2];
    let key = &args[3];
    let value = args.get(4).map(|s| s.as_str());

    // Resolve directory: if relative, look for vault by walking up from cwd
    let target = if Path::new(dir).is_absolute() {
        dir.to_string()
    } else {
        match vault::find_vault_from_cwd() {
            Some(v) => format!("{}/{}", v.display(), dir),
            None => {
                eprintln!("Cannot find vault root (no .tlp file in parent directories)");
                return ExitCode::from(1);
            }
        }
    };

    let target_path = Path::new(&target);
    if !target_path.is_dir() {
        eprintln!("Directory not found: {}", dir);
        return ExitCode::from(1);
    }

    match action.as_str() {
        "set" => cmd_set(target_path, key, value),
        "get" => cmd_get(target_path, key),
        "has" => cmd_has(target_path, key),
        _ => {
            eprintln!("Unknown action: {} (use set, get, or has)", action);
            ExitCode::from(1)
        }
    }
}

fn cmd_set(dir: &Path, key: &str, value: Option<&str>) -> ExitCode {
    let value = match value {
        Some(v) => v,
        None => {
            eprintln!("set requires a value");
            return ExitCode::from(1);
        }
    };

    let (mut count, mut total) = (0usize, 0usize);

    for entry in frontmatter::read_md_files(dir) {
        total += 1;
        let name = entry
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let content = match fs::read_to_string(&entry) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let new_content = frontmatter::set_value(&content, key, value);
        if new_content == content {
            println!("  ok:      {}", name);
            count += 1;
            continue;
        }
        if let Err(e) = fs::write(&entry, &new_content) {
            eprintln!("  error:   {} ({})", name, e);
            continue;
        }
        println!("  updated: {}", name);
        count += 1;
    }

    println!();
    println!(
        "Done: {}/{} files processed with {}: {}",
        count, total, key, value
    );
    ExitCode::SUCCESS
}

fn cmd_get(dir: &Path, key: &str) -> ExitCode {
    let (mut count, mut total) = (0usize, 0usize);

    for entry in frontmatter::read_md_files(dir) {
        total += 1;
        let name = entry
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let content = match fs::read_to_string(&entry) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if let Some(val) = frontmatter::get_value(&content, key) {
            println!("  {}: {}", name, val);
            count += 1;
        }
    }

    println!();
    println!("{}/{} files have {} set", count, total, key);
    ExitCode::SUCCESS
}

fn cmd_has(dir: &Path, key: &str) -> ExitCode {
    let (mut missing, mut total) = (0usize, 0usize);

    println!("Files missing {}:", key);

    for entry in frontmatter::read_md_files(dir) {
        total += 1;
        let name = entry
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let content = match fs::read_to_string(&entry) {
            Ok(c) => c,
            Err(_) => continue,
        };

        if frontmatter::get_value(&content, key).is_none() {
            println!("  {}", name);
            missing += 1;
        }
    }

    println!();
    println!("{}/{} files missing {}", missing, total, key);
    ExitCode::SUCCESS
}
