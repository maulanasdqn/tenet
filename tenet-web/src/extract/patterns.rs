use std::sync::LazyLock;

use regex::Regex;

pub static METHOD_CALL: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)\.(get|post|put|patch|delete|head|options)\s*\(\s*["'`]([^"'`]{1,512})["'`]\s*[,)]"#,
    )
    .ok()
});

pub static FETCH_CALL: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r#"(?i)\bfetch\s*\(\s*["'`]([^"'`]{1,512})["'`]\s*[,)]"#).ok());

pub static PATH_LITERAL: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r#"["'`](/(?:api|rest|graphql|gateway|v[0-9]{1,2})(?:/[^"'`\s]{0,256})?)["'`]"#).ok()
});

pub static ABSOLUTE_LITERAL: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(
        r#"["'`](https?://[^"'`\s/]{1,255}(?::[0-9]{1,5})?/(?:api|rest|graphql|gateway|v[0-9]{1,2})(?:/[^"'`\s]{0,256})?)["'`]"#,
    )
    .ok()
});

#[cfg(test)]
mod tests {
    use super::{ABSOLUTE_LITERAL, FETCH_CALL, METHOD_CALL, PATH_LITERAL};

    #[test]
    fn every_pattern_compiles() {
        assert!(METHOD_CALL.is_some());
        assert!(FETCH_CALL.is_some());
        assert!(PATH_LITERAL.is_some());
        assert!(ABSOLUTE_LITERAL.is_some());
    }
}
