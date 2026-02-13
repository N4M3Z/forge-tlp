use context_tlp::tlp;
use serde::Deserialize;
use std::io::Read;
use std::process::ExitCode;

/// JSON payload from Claude Code's `PreToolUse` hook.
#[derive(Deserialize)]
struct HookInput {
    tool_name: Option<String>,
    tool_input: Option<ToolInput>,
}

#[derive(Deserialize)]
struct ToolInput {
    file_path: Option<String>,
}

fn main() -> ExitCode {
    let mut buf = String::new();
    if std::io::stdin().read_to_string(&mut buf).is_err() {
        return ExitCode::SUCCESS;
    }

    let Ok(input) = serde_json::from_str::<HookInput>(&buf) else {
        return ExitCode::SUCCESS; // JSON parse error is Claude Code's responsibility
    };

    let tool_name = input.tool_name.unwrap_or_default();
    let file_path = input
        .tool_input
        .and_then(|ti| ti.file_path)
        .unwrap_or_default();

    if file_path.is_empty() {
        return ExitCode::SUCCESS; // Some tool calls legitimately have no file path
    }

    let Some(classification) = tlp::classify_file(&file_path) else {
        return ExitCode::SUCCESS; // File outside any vault — not our problem
    };

    if classification.config_error {
        eprintln!("Malformed .tlp config. All files treated as RED until fixed.");
        return ExitCode::from(2);
    }

    match classification.level {
        tlp::Tlp::Red => {
            // Allow creating new files — nothing to leak if the file doesn't exist yet
            if tool_name == "Write" && !std::path::Path::new(&file_path).exists() {
                println!(
                    "TLP:RED — new file creation allowed in: {}",
                    classification.rel_path
                );
                ExitCode::SUCCESS
            } else {
                eprintln!(
                    "TLP:RED — access blocked for: {}",
                    classification.rel_path
                );
                ExitCode::from(2)
            }
        }
        tlp::Tlp::Amber => {
            if tool_name == "Read" {
                eprintln!(
                    "TLP:AMBER — this file requires approval. Ask the user, then use:\n\
                     safe-read \"{file_path}\""
                );
                ExitCode::from(2)
            } else {
                println!(
                    "TLP:AMBER — editing allowed, but never output content verbatim from: {}",
                    classification.rel_path
                );
                ExitCode::SUCCESS
            }
        }
        _ => ExitCode::SUCCESS,
    }
}
