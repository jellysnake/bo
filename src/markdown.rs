// Markdown document formatting and file writing

use std::fs;
use std::io;
use std::path::Path;

/// Format a markdown document with YAML frontmatter.
pub fn format_document(title: Option<&str>, url: &str, fetched_at: &str, body: &str) -> String {
    let title_yaml = match title {
        Some(t) => format!("\"{}\"", t.replace('\\', "\\\\").replace('"', "\\\"")),
        None => "\"\"".to_string(),
    };

    let mut doc = String::new();
    doc.push_str("---\n");
    doc.push_str(&format!("title: {}\n", title_yaml));
    doc.push_str(&format!("url: {}\n", url));
    doc.push_str(&format!("fetched: {}\n", fetched_at));
    doc.push_str("---\n\n");

    if let Some(t) = title {
        doc.push_str(&format!("# {}\n\n", t));
    }

    doc.push_str(body);
    doc.push('\n');

    doc
}

/// Write a markdown document to the output directory.
pub fn write_document(output_dir: &Path, filename: &str, content: &str) -> io::Result<()> {
    fs::create_dir_all(output_dir)?;
    let path = output_dir.join(format!("{}.md", filename));
    fs::write(path, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn format_with_all_fields() {
        let doc = format_document(
            Some("My Article"),
            "https://example.com",
            "2025-01-15T09:32:00Z",
            "Some content here.",
        );
        assert!(doc.starts_with("---\n"));
        assert!(doc.contains("title: \"My Article\""));
        assert!(doc.contains("url: https://example.com"));
        assert!(doc.contains("fetched: 2025-01-15T09:32:00Z"));
        assert!(doc.contains("# My Article"));
        assert!(doc.contains("Some content here."));
    }

    #[test]
    fn format_with_no_title() {
        let doc = format_document(
            None,
            "https://example.com",
            "2025-01-15T09:32:00Z",
            "Body only.",
        );
        assert!(doc.contains("title: \"\""));
        assert!(!doc.contains("# ")); // No heading
    }

    #[test]
    fn format_escapes_yaml_special_chars() {
        let doc = format_document(
            Some("Rust: A \"Fast\" Language"),
            "https://example.com",
            "2025-01-15T09:32:00Z",
            "Content.",
        );
        assert!(doc.contains(r#"title: "Rust: A \"Fast\" Language""#));
    }

    #[test]
    fn write_creates_file() {
        let dir = TempDir::new().unwrap();
        write_document(dir.path(), "test-article", "# Hello\n\nWorld.").unwrap();
        let path = dir.path().join("test-article.md");
        assert!(path.exists());
        assert_eq!(fs::read_to_string(path).unwrap(), "# Hello\n\nWorld.");
    }

    #[test]
    fn write_creates_missing_dirs() {
        let dir = TempDir::new().unwrap();
        let nested = dir.path().join("sub").join("dir");
        write_document(&nested, "article", "content").unwrap();
        assert!(nested.join("article.md").exists());
    }
}
