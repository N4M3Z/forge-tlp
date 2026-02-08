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
    assert!(matches_pattern(
        "Resources/Contacts/john.md",
        "Resources/Contacts/**"
    ));
    assert!(matches_pattern(
        "Resources/Contacts/sub/deep.md",
        "Resources/Contacts/**"
    ));
    assert!(!matches_pattern(
        "Resources/ContactsExtra/john.md",
        "Resources/Contacts/**"
    ));
}

#[test]
fn test_exact_match() {
    assert!(matches_pattern("AI/Identity.md", "AI/Identity.md"));
    assert!(!matches_pattern("AI/Identity.md.bak", "AI/Identity.md"));
}

#[test]
fn test_classify_full_config() {
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
    assert_eq!(classify(Path::new("foo.pdf"), config), Tlp::Red);
    assert_eq!(
        classify(Path::new("Resources/Contacts/john.md"), config),
        Tlp::Red
    );
    assert_eq!(classify(Path::new("AI/Identity.md"), config), Tlp::Amber);
    assert_eq!(
        classify(Path::new("Pipeline/Fleeting/note.md"), config),
        Tlp::Amber
    );
    assert_eq!(classify(Path::new("Topics/rust.md"), config), Tlp::Green);
    assert_eq!(classify(Path::new("random/file.md"), config), Tlp::Amber);
}

#[test]
fn test_default_amber_for_unlisted() {
    let config = "GREEN:\n  - \"Topics/**\"\n";
    assert_eq!(classify(Path::new("other/file.md"), config), Tlp::Amber);
}

#[test]
fn test_empty_config() {
    assert_eq!(classify(Path::new("anything.md"), ""), Tlp::Amber);
}

#[test]
fn test_comments_ignored() {
    let config = "# This is a comment\nRED:\n  - \"*.pdf\"\n";
    assert_eq!(classify(Path::new("file.pdf"), config), Tlp::Red);
    assert_eq!(classify(Path::new("file.md"), config), Tlp::Amber);
}

#[test]
fn test_first_match_wins() {
    let config = "RED:\n  - \"*.md\"\n\nGREEN:\n  - \"Topics/**\"\n";
    assert_eq!(classify(Path::new("Topics/rust.md"), config), Tlp::Red);
}

#[test]
fn test_most_restrictive() {
    assert_eq!(most_restrictive(Tlp::Red, Tlp::Green), Tlp::Red);
    assert_eq!(most_restrictive(Tlp::Green, Tlp::Red), Tlp::Red);
    assert_eq!(most_restrictive(Tlp::Amber, Tlp::Clear), Tlp::Amber);
    assert_eq!(most_restrictive(Tlp::Green, Tlp::Green), Tlp::Green);
    assert_eq!(most_restrictive(Tlp::Clear, Tlp::Red), Tlp::Red);
}

#[test]
fn test_from_str() {
    assert_eq!(from_str("RED"), Some(Tlp::Red));
    assert_eq!(from_str("red"), Some(Tlp::Red));
    assert_eq!(from_str("Amber"), Some(Tlp::Amber));
    assert_eq!(from_str("GREEN"), Some(Tlp::Green));
    assert_eq!(from_str("CLEAR"), Some(Tlp::Clear));
    assert_eq!(from_str("invalid"), None);
    assert_eq!(from_str(""), None);
}

#[test]
fn test_pattern_edge_empty_pattern() {
    assert!(!matches_pattern("file.md", ""));
}

#[test]
fn test_dir_match_exact_prefix() {
    // "Contacts" matches "Contacts/**" because path.len() == prefix.len()
    // This is existing behavior — the pattern protects the directory itself too.
    assert!(matches_pattern("Contacts", "Contacts/**"));
}

#[test]
fn test_double_star_matches_everything() {
    assert!(matches_pattern("any/deep/path.md", "**"));
    assert!(matches_pattern("file.txt", "**"));
    assert!(matches_pattern("a", "**"));
}

#[test]
fn test_double_star_as_green_catchall() {
    let config = "AMBER:\n  - \"Players/**\"\n\nGREEN:\n  - \"**\"\n";
    assert_eq!(classify(Path::new("Players/card.md"), config), Tlp::Amber);
    assert_eq!(classify(Path::new("Campaigns/scene.md"), config), Tlp::Green);
    assert_eq!(classify(Path::new("anything.md"), config), Tlp::Green);
}
