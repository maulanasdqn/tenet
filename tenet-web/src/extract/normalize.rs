use url::Url;

const ASSET_EXTENSIONS: &[&str] = &[
    ".js", ".mjs", ".cjs", ".css", ".map", ".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp",
    ".ico", ".woff", ".woff2", ".ttf", ".eot", ".mp4", ".webm", ".pdf", ".txt", ".xml",
];

pub fn split_target(raw: &str) -> Option<(Option<String>, String)> {
    if raw.starts_with("http://") || raw.starts_with("https://") {
        let parsed = Url::parse(raw).ok()?;
        let origin = parsed.origin();
        if !origin.is_tuple() {
            return None;
        }
        return accept(Some(origin.ascii_serialization()), parsed.path());
    }
    if raw.starts_with('/') && !raw.starts_with("//") {
        return accept(None, raw);
    }
    None
}

fn accept(base_url: Option<String>, raw_path: &str) -> Option<(Option<String>, String)> {
    let path = normalize_path(raw_path);
    if path == "/" || is_asset(&path) {
        return None;
    }
    Some((base_url, path))
}

pub fn normalize_path(raw: &str) -> String {
    let bare = raw.split(['?', '#']).next().unwrap_or(raw);
    let joined = bare
        .split('/')
        .map(templatize)
        .collect::<Vec<String>>()
        .join("/");
    let trimmed = if joined.len() > 1 {
        joined.trim_end_matches('/')
    } else {
        joined.as_str()
    };
    if trimmed.is_empty() {
        return "/".to_owned();
    }
    trimmed.to_owned()
}

fn templatize(segment: &str) -> String {
    if let Some(name) = segment.strip_prefix(':') {
        return format!("{{{}}}", sanitize(name));
    }
    if let Some(start) = segment.find("${") {
        let rest = &segment[start + 2..];
        let name = rest.split('}').next().unwrap_or(rest);
        return format!("{{{}}}", sanitize(name));
    }
    segment.to_owned()
}

fn sanitize(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .filter(|letter| letter.is_alphanumeric() || *letter == '_')
        .collect();
    if cleaned.is_empty() {
        return "param".to_owned();
    }
    cleaned
}

pub fn is_asset(path: &str) -> bool {
    let lower = path.to_lowercase();
    ASSET_EXTENSIONS
        .iter()
        .any(|extension| lower.ends_with(extension))
}

#[cfg(test)]
mod tests {
    use super::{is_asset, normalize_path, split_target};

    #[test]
    fn an_absolute_url_splits_into_origin_and_path() {
        let split = split_target("https://api.example.com/v1/users");
        assert_eq!(
            split,
            Some((
                Some("https://api.example.com".to_owned()),
                "/v1/users".to_owned()
            ))
        );
    }

    #[test]
    fn a_relative_path_has_no_origin() {
        assert_eq!(
            split_target("/api/users"),
            Some((None, "/api/users".to_owned()))
        );
    }

    #[test]
    fn a_protocol_relative_target_is_rejected() {
        assert_eq!(split_target("//cdn.example.com/api/x"), None);
    }

    #[test]
    fn an_asset_path_is_rejected() {
        assert_eq!(split_target("/api/bundle.js"), None);
        assert!(is_asset("/static/app.CSS"));
    }

    #[test]
    fn a_query_string_is_dropped() {
        assert_eq!(normalize_path("/api/users?page=2#top"), "/api/users");
    }

    #[test]
    fn an_express_parameter_becomes_a_template() {
        assert_eq!(
            normalize_path("/api/users/:userId/posts"),
            "/api/users/{userId}/posts"
        );
    }

    #[test]
    fn a_template_literal_becomes_a_template() {
        assert_eq!(normalize_path("/api/users/${id}"), "/api/users/{id}");
    }

    #[test]
    fn a_trailing_slash_is_dropped() {
        assert_eq!(normalize_path("/api/users/"), "/api/users");
        assert_eq!(normalize_path("/"), "/");
    }
}
