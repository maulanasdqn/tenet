use url::Url;

#[derive(Debug, Clone, Default)]
pub struct PageSnapshot {
    pub url: String,
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl PageSnapshot {
    pub fn new(
        url: impl Into<String>,
        status: u16,
        headers: Vec<(String, String)>,
        body: impl Into<String>,
    ) -> Self {
        Self {
            url: url.into(),
            status,
            headers,
            body: body.into(),
        }
    }

    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    pub fn header_values(&self, name: &str) -> Vec<&str> {
        self.headers
            .iter()
            .filter(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
            .collect()
    }

    pub fn cookies(&self) -> Vec<&str> {
        self.header_values("set-cookie")
    }

    pub fn origin(&self) -> Option<String> {
        let parsed = Url::parse(&self.url).ok()?;
        let origin = parsed.origin();
        if origin.is_tuple() {
            return Some(origin.ascii_serialization());
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct ScriptAsset {
    pub url: String,
    pub body: String,
}

impl ScriptAsset {
    pub fn new(url: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            body: body.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PageSnapshot;

    fn page() -> PageSnapshot {
        PageSnapshot::new(
            "https://example.com/app",
            200,
            vec![
                ("Server".to_owned(), "nginx".to_owned()),
                ("Set-Cookie".to_owned(), "sid=1".to_owned()),
                ("set-cookie".to_owned(), "csrftoken=2".to_owned()),
            ],
            "<html></html>",
        )
    }

    #[test]
    fn a_header_lookup_ignores_case() {
        assert_eq!(page().header("server"), Some("nginx"));
    }

    #[test]
    fn every_repeated_header_is_returned() {
        assert_eq!(page().cookies().len(), 2);
    }

    #[test]
    fn the_origin_drops_the_path() {
        assert_eq!(page().origin(), Some("https://example.com".to_owned()));
    }
}
