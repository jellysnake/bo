// Test URL constants for integration testing

/// Standard articles — happy path
pub const ARTICLE_WIKIPEDIA_1: &str = "https://en.wikipedia.org/wiki/Jaya_Sri_Maha_Bodhi";
pub const ARTICLE_WIKIPEDIA_2: &str = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
pub const ARTICLE_BLOG: &str = "https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/";

/// Link-heavy page
pub const LINK_HEAVY: &str = "https://en.wikipedia.org/wiki/Hyperlink";

/// Very long page (100KB+ body)
pub const VERY_LONG: &str = "https://en.wikipedia.org/wiki/United_States";

/// Slug collision pair — two pages that will produce similar slugs
pub const SLUG_COLLISION_1: &str = "https://en.wikipedia.org/wiki/Introduction";
pub const SLUG_COLLISION_2: &str = "https://en.wiktionary.org/wiki/introduction";

/// Near-duplicate URLs (same base, different query params)
pub const NEAR_DUP_BASE: &str = "https://en.wikipedia.org/wiki/Rust_(programming_language)";
pub const NEAR_DUP_VARIANT: &str =
    "https://en.wikipedia.org/wiki/Rust_(programming_language)?ref=twitter";

/// Paywalled / auth-gated
pub const PAYWALLED: &str = "https://www.wsj.com/articles/some-premium-article-that-requires-login";

/// JS-rendered SPA (React app)
pub const JS_SPA: &str = "https://react.dev/learn";

/// Dead URLs
pub const DEAD_404: &str = "https://httpbin.org/status/404";
pub const DEAD_500: &str = "https://httpbin.org/status/500";

/// Non-HTML content
pub const NON_HTML_PDF: &str =
    "https://www.w3.org/WAI/ER/tests/xhtml/testfiles/resources/pdf/dummy.pdf";
pub const NON_HTML_BINARY: &str = "https://httpbin.org/bytes/1024";
