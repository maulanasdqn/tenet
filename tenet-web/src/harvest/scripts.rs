use scraper::{Html, Selector};
use url::Url;

pub fn script_urls(html: &str, base_url: &str) -> Vec<String> {
    let Ok(base) = Url::parse(base_url) else {
        return Vec::new();
    };
    let Ok(selector) = Selector::parse("script[src]") else {
        return Vec::new();
    };

    let document = Html::parse_document(html);
    let mut found: Vec<String> = Vec::new();
    for element in document.select(&selector) {
        let Some(src) = element.value().attr("src") else {
            continue;
        };
        let Ok(resolved) = base.join(src) else {
            continue;
        };
        if !matches!(resolved.scheme(), "http" | "https") {
            continue;
        }
        let url = resolved.to_string();
        if !found.contains(&url) {
            found.push(url);
        }
    }
    found
}

pub fn inline_scripts(html: &str) -> Vec<String> {
    let Ok(selector) = Selector::parse("script:not([src])") else {
        return Vec::new();
    };

    let document = Html::parse_document(html);
    document
        .select(&selector)
        .map(|element| element.text().collect::<String>())
        .filter(|body| !body.trim().is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{inline_scripts, script_urls};

    const PAGE: &str = r#"
        <html><head>
          <script src="/static/app.js"></script>
          <script src="https://cdn.example.net/vendor.js"></script>
          <script src="/static/app.js"></script>
          <script>window.API = "/api/v1";</script>
        </head></html>
    "#;

    #[test]
    fn a_relative_source_resolves_against_the_page() {
        let found = script_urls(PAGE, "https://example.com/app/index.html");
        assert!(found.contains(&"https://example.com/static/app.js".to_owned()));
    }

    #[test]
    fn an_absolute_source_is_kept_as_is() {
        let found = script_urls(PAGE, "https://example.com/");
        assert!(found.contains(&"https://cdn.example.net/vendor.js".to_owned()));
    }

    #[test]
    fn a_repeated_source_is_listed_once() {
        assert_eq!(script_urls(PAGE, "https://example.com/").len(), 2);
    }

    #[test]
    fn an_inline_script_keeps_its_body() {
        let found = inline_scripts(PAGE);
        assert_eq!(found.len(), 1);
        assert!(found[0].contains("/api/v1"));
    }

    #[test]
    fn a_broken_base_url_yields_nothing() {
        assert!(script_urls(PAGE, "not a url").is_empty());
    }
}
