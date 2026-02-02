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
