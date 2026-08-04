use crate::profile::StealthProfile;

const TEMPLATE: &str = include_str!("stealth.js");

pub fn init_script(profile: &StealthProfile) -> String {
    TEMPLATE
        .replace("__TENET_LANGUAGES__", &languages_json(&profile.languages))
        .replace("__TENET_VENDOR__", &json_string(&profile.vendor))
        .replace(
            "__TENET_HARDWARE__",
            &profile.hardware_concurrency.to_string(),
        )
        .replace("__TENET_MEMORY__", &profile.device_memory.to_string())
        .replace(
            "__TENET_WEBGL_VENDOR__",
            &json_string(&profile.webgl_vendor),
        )
        .replace(
            "__TENET_WEBGL_RENDERER__",
            &json_string(&profile.webgl_renderer),
        )
}

fn languages_json(languages: &[String]) -> String {
    let items: Vec<String> = languages
        .iter()
        .map(|language| json_string(language))
        .collect();
    format!("[{}]", items.join(", "))
}

fn json_string(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

#[cfg(test)]
mod tests {
    use crate::profile::StealthProfile;

    use super::init_script;

    fn script(region: Option<&str>) -> String {
        let profile = StealthProfile::from_user_agent(
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) HeadlessChrome/134.0.0.0 Safari/537.36",
            region,
        );
        init_script(&profile)
    }

    #[test]
    fn no_placeholder_survives_rendering() {
        assert!(!script(Some("id")).contains("__TENET_"));
    }

    #[test]
    fn the_languages_are_woven_into_the_script() {
        let rendered = script(Some("id"));
        assert!(rendered.contains(r#"["id-ID", "id", "en"]"#));
    }

    #[test]
    fn the_webdriver_flag_is_masked() {
        assert!(script(None).contains("webdriver"));
    }

    #[test]
    fn a_value_with_a_quote_cannot_break_out_of_its_string() {
        let mut profile = StealthProfile::from_user_agent("Chrome/134.0.0.0", None);
        profile.webgl_renderer = "evil\" + attack".to_owned();
        let rendered = init_script(&profile);
        assert!(rendered.contains(r#"evil\" + attack"#));
    }
}
