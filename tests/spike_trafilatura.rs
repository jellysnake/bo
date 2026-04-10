use trafilatura::{extract, Options};

const SIMPLE_ARTICLE: &str = r#"
<html>
<head><title>Rust Ownership Explained</title></head>
<body>
  <nav><a href="/">Home</a> | <a href="/blog">Blog</a></nav>
  <article>
    <h1>Rust Ownership Explained</h1>
    <p>Ownership is one of Rust's most unique features. It enables Rust to make memory safety guarantees without needing a garbage collector.</p>
    <h2>What is Ownership?</h2>
    <p>Ownership is a set of rules that govern how a Rust program manages memory. All programs have to manage the way they use a computer's memory while running.</p>
    <h3>The Stack and the Heap</h3>
    <p>Both the stack and the heap are parts of memory available to your code to use at runtime, but they are structured in different ways.</p>
    <p>For more info, see <a href="https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html">The Rust Book</a> and <a href="https://doc.rust-lang.org/nomicon/">The Rustonomicon</a>.</p>
  </article>
  <footer>Copyright 2024</footer>
</body>
</html>
"#;

const LINK_HEAVY: &str = r#"
<html>
<head><title>Link Collection</title></head>
<body>
  <article>
    <h1>Useful Resources</h1>
    <p>Here are some <a href="https://example.com/1">important links</a> that you should check out.</p>
    <ul>
      <li><a href="https://rust-lang.org">Rust Programming Language</a> - A systems language</li>
      <li><a href="https://python.org">Python</a> - A scripting language</li>
      <li><a href="https://golang.org">Go</a> - A compiled language by <a href="https://google.com">Google</a></li>
    </ul>
    <p>See also: <a href="/page1">page 1</a>, <a href="/page2">page 2</a>, and <a href="/page3">page 3</a>.</p>
  </article>
</body>
</html>
"#;

const WITH_TABLES: &str = r#"
<html>
<head><title>Comparison Table</title></head>
<body>
  <article>
    <h1>Language Comparison</h1>
    <p>Below is a comparison of programming languages:</p>
    <table>
      <thead>
        <tr><th>Language</th><th>Type System</th><th>GC</th></tr>
      </thead>
      <tbody>
        <tr><td>Rust</td><td>Static</td><td>No</td></tr>
        <tr><td>Go</td><td>Static</td><td>Yes</td></tr>
        <tr><td>Python</td><td>Dynamic</td><td>Yes</td></tr>
      </tbody>
    </table>
  </article>
</body>
</html>
"#;

const EMPTY_PAGE: &str = r#"
<html>
<head><title></title></head>
<body>
  <nav>Menu</nav>
  <footer>Footer</footer>
</body>
</html>
"#;

const MINIMAL_BOILERPLATE: &str = r#"
<html>
<head><title>Login Required</title></head>
<body>
  <nav><a href="/">Home</a></nav>
  <div class="login-wall">
    <h1>Please log in to continue</h1>
    <form><input type="email"><input type="password"><button>Log in</button></form>
  </div>
  <footer>Terms of Service | Privacy Policy</footer>
</body>
</html>
"#;

const NON_UTF8_CONTENT: &str = r#"
<html>
<head>
  <meta charset="iso-8859-1">
  <title>Caf&eacute; Culture</title>
</head>
<body>
  <article>
    <h1>Caf&eacute; Culture in Europe</h1>
    <p>The caf&eacute; has been a cornerstone of European social life for centuries.</p>
    <p>From the Wiener Kaffeehaus to the Parisian bistro, caf&eacute;s serve as gathering places.</p>
  </article>
</body>
</html>
"#;

#[test]
fn spike_simple_article_text() {
    let opts = Options::default();
    let result = extract(SIMPLE_ARTICLE, &opts);
    assert!(result.is_ok(), "extract() returned None for a simple article");
    let result = result.unwrap();
    assert!(!result.content_text.is_empty(), "content_text is empty");
    assert!(
        result.content_text.contains("Ownership"),
        "content_text missing key content: {}",
        &result.content_text[..200.min(result.content_text.len())]
    );
    println!("=== TITLE ===\n{}", result.metadata.title);
    println!("=== CONTENT_TEXT (first 500) ===\n{}", &result.content_text[..500.min(result.content_text.len())]);
}

#[test]
fn spike_simple_article_markdown() {
    let opts = Options::default();
    let result = extract(SIMPLE_ARTICLE, &opts).unwrap();
    let md = result.content_markdown();
    println!("=== CONTENT_MARKDOWN ===\n{md}");

    // Check heading hierarchy preserved
    assert!(md.contains("# ") || md.contains("## "), "markdown has no headings: {md}");
    // Ideally we see ## for h2 and ### for h3
    let has_h2 = md.contains("## ");
    let has_h3 = md.contains("### ");
    println!("Has ## headings: {has_h2}");
    println!("Has ### headings: {has_h3}");
}

#[test]
fn spike_links_stripped_by_default() {
    // Default is with_links(false)
    let opts = Options::default();
    let result = extract(LINK_HEAVY, &opts).unwrap();
    let md = result.content_markdown();
    let text = &result.content_text;
    println!("=== LINK_HEAVY MARKDOWN ===\n{md}");
    println!("=== LINK_HEAVY TEXT ===\n{text}");

    // Should NOT contain markdown links
    let has_md_links = md.contains("](http") || md.contains("](/)");
    println!("Markdown contains links: {has_md_links}");

    // The anchor text should still be present
    assert!(text.contains("Rust Programming Language") || text.contains("Python"),
        "anchor text was lost entirely");
}

#[test]
fn spike_links_enabled() {
    // Verify with_links(true) does include them (so we know the toggle works)
    let opts = Options::default().with_links(true);
    let result = extract(LINK_HEAVY, &opts).unwrap();
    let md = result.content_markdown();
    println!("=== LINK_HEAVY WITH LINKS ===\n{md}");
    let has_links = md.contains("](http") || md.contains("](/)");
    println!("Markdown contains links when enabled: {has_links}");
}

#[test]
fn spike_tables() {
    let opts = Options::default();
    let result = extract(WITH_TABLES, &opts);
    assert!(result.is_ok(), "extract() returned None for table content");
    let result = result.unwrap();
    let md = result.content_markdown();
    let text = &result.content_text;
    println!("=== TABLE MARKDOWN ===\n{md}");
    println!("=== TABLE TEXT ===\n{text}");

    // At minimum the cell content should be present
    assert!(text.contains("Rust") && text.contains("Python"),
        "table cell content missing from text output");
}

#[test]
fn spike_empty_page() {
    let opts = Options::default();
    let result = extract(EMPTY_PAGE, &opts);
    println!("=== EMPTY PAGE RESULT ===\n{:?}", result.as_ref().ok().map(|r| &r.content_text));
    // Should return None or empty content — either is acceptable
    if let Ok(r) = &result {
        println!("Got content (len={}): {:?}", r.content_text.len(), &r.content_text);
    }
}

#[test]
fn spike_login_wall() {
    let opts = Options::default();
    let result = extract(MINIMAL_BOILERPLATE, &opts);
    println!("=== LOGIN WALL RESULT ===\n{:?}", result.as_ref().ok().map(|r| &r.content_text));
    // Likely returns None or very minimal content
}

#[test]
fn spike_non_utf8_html_entities() {
    let opts = Options::default();
    let result = extract(NON_UTF8_CONTENT, &opts);
    assert!(result.is_ok(), "extract() returned None for HTML-entity content");
    let result = result.unwrap();
    println!("=== NON-UTF8 TEXT ===\n{}", result.content_text);
    println!("=== NON-UTF8 TITLE ===\n{}", result.metadata.title);
    // café should be decoded from &eacute;
    assert!(result.content_text.contains("caf") , "content missing café reference");
}

#[test]
fn spike_large_document() {
    // Generate a 100KB+ HTML document
    let mut body = String::from("<html><head><title>Large Document</title></head><body><article>");
    body.push_str("<h1>Large Document</h1>");
    for i in 0..2000 {
        body.push_str(&format!(
            "<p>Paragraph {i}. This is a reasonably long paragraph of text that contributes to the overall size of the document. It contains multiple sentences to simulate real content.</p>"
        ));
        if i % 100 == 0 {
            body.push_str(&format!("<h2>Section {}</h2>", i / 100));
        }
    }
    body.push_str("</article></body></html>");

    let html_size = body.len();
    println!("Generated HTML size: {html_size} bytes ({:.1} KB)", html_size as f64 / 1024.0);
    assert!(html_size > 100_000, "test HTML should be >100KB, got {html_size}");

    let opts = Options::default();
    let result = extract(&body, &opts);
    assert!(result.is_ok(), "extract() returned None for large document");
    let result = result.unwrap();
    let text_len = result.content_text.len();
    let md = result.content_markdown();
    let md_len = md.len();
    println!("Extracted text size: {text_len} bytes ({:.1} KB)", text_len as f64 / 1024.0);
    println!("Extracted markdown size: {md_len} bytes ({:.1} KB)", md_len as f64 / 1024.0);

    // Should extract a substantial portion of the content
    assert!(text_len > 50_000, "extracted text suspiciously small: {text_len} bytes");

    // Check no truncation — last paragraphs should be present
    assert!(result.content_text.contains("Paragraph 1999"),
        "last paragraph missing — content may be truncated");
}

#[test]
fn spike_metadata_extraction() {
    let opts = Options::default();
    let result = extract(SIMPLE_ARTICLE, &opts).unwrap();
    println!("=== METADATA ===");
    println!("title: {:?}", result.metadata.title);
    println!("author: {:?}", result.metadata.author);
    println!("date: {:?}", result.metadata.date);
    println!("description: {:?}", result.metadata.description);
    // Title should be extracted
    assert!(!result.metadata.title.is_empty(), "title not extracted");
}
