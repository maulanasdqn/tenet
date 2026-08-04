use crate::language::{accept_language, navigator_languages};

const HEADLESS_MARKER: &str = "HeadlessChrome";
const CHROME_MARKER: &str = "Chrome";
const FALLBACK_VERSION: &str = "131.0.0.0";
const VENDOR: &str = "Google Inc.";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Brand {
    pub brand: String,
    pub version: String,
}

#[derive(Debug, Clone)]
pub struct StealthProfile {
    pub user_agent: String,
    pub navigator_platform: String,
    pub ua_platform: String,
    pub platform_version: String,
    pub architecture: String,
    pub major_version: String,
    pub full_version: String,
    pub vendor: String,
    pub languages: Vec<String>,
    pub accept_language: String,
    pub hardware_concurrency: u32,
    pub device_memory: u32,
    pub webgl_vendor: String,
    pub webgl_renderer: String,
}

impl StealthProfile {
    pub fn from_user_agent(reported: &str, region: Option<&str>) -> Self {
        let user_agent = normalize(reported);
        let full_version = chrome_version(&user_agent);
        let major_version = full_version
            .split('.')
            .next()
            .unwrap_or(FALLBACK_VERSION)
            .to_owned();
        let platform = Platform::detect(&user_agent);
        Self {
            user_agent,
            navigator_platform: platform.navigator.to_owned(),
            ua_platform: platform.ua.to_owned(),
            platform_version: platform.version.to_owned(),
            architecture: platform.architecture.to_owned(),
            major_version,
            full_version,
            vendor: VENDOR.to_owned(),
            languages: navigator_languages(region),
            accept_language: accept_language(region),
            hardware_concurrency: 8,
            device_memory: 8,
            webgl_vendor: platform.webgl_vendor.to_owned(),
            webgl_renderer: platform.webgl_renderer.to_owned(),
        }
    }

    pub fn brands(&self) -> Vec<Brand> {
        vec![
            brand("Chromium", &self.major_version),
            brand("Google Chrome", &self.major_version),
            brand("Not?A_Brand", "24"),
        ]
    }

    pub fn full_versions(&self) -> Vec<Brand> {
        vec![
            brand("Chromium", &self.full_version),
            brand("Google Chrome", &self.full_version),
            brand("Not?A_Brand", "24.0.0.0"),
        ]
    }
}

fn brand(name: &str, version: &str) -> Brand {
    Brand {
        brand: name.to_owned(),
        version: version.to_owned(),
    }
}

fn normalize(reported: &str) -> String {
    if reported.trim().is_empty() {
        return default_user_agent();
    }
    reported.replace(HEADLESS_MARKER, CHROME_MARKER)
}

fn default_user_agent() -> String {
    format!(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
         (KHTML, like Gecko) Chrome/{FALLBACK_VERSION} Safari/537.36"
    )
}

fn chrome_version(user_agent: &str) -> String {
    user_agent
        .split("Chrome/")
        .nth(1)
        .and_then(|rest| rest.split_whitespace().next())
        .filter(|version| !version.is_empty())
        .unwrap_or(FALLBACK_VERSION)
        .to_owned()
}

struct Platform {
    navigator: &'static str,
    ua: &'static str,
    version: &'static str,
    architecture: &'static str,
    webgl_vendor: &'static str,
    webgl_renderer: &'static str,
}

impl Platform {
    fn detect(user_agent: &str) -> Self {
        if user_agent.contains("Windows") {
            return Self {
                navigator: "Win32",
                ua: "Windows",
                version: "15.0.0",
                architecture: "x86",
                webgl_vendor: "Google Inc. (NVIDIA)",
                webgl_renderer:
                    "ANGLE (NVIDIA, NVIDIA GeForce RTX 3060 Direct3D11 vs_5_0 ps_5_0, D3D11)",
            };
        }
        if user_agent.contains("Macintosh") {
            return Self {
                navigator: "MacIntel",
                ua: "macOS",
                version: "14.6.0",
                architecture: "arm",
                webgl_vendor: "Google Inc. (Apple)",
                webgl_renderer:
                    "ANGLE (Apple, ANGLE Metal Renderer: Apple M1, Unspecified Version)",
            };
        }
        Self {
            navigator: "Linux x86_64",
            ua: "Linux",
            version: "",
            architecture: "x86",
            webgl_vendor: "Google Inc. (Intel)",
            webgl_renderer: "ANGLE (Intel, Mesa Intel(R) UHD Graphics (TGL GT1), OpenGL 4.6)",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StealthProfile;

    const HEADLESS: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
                            (KHTML, like Gecko) HeadlessChrome/134.0.6998.0 Safari/537.36";

    #[test]
    fn the_headless_marker_is_erased_and_the_version_follows_the_browser() {
        let profile = StealthProfile::from_user_agent(HEADLESS, None);
        assert!(!profile.user_agent.contains("Headless"));
        assert!(profile.user_agent.contains("Chrome/134.0.6998.0"));
        assert_eq!(profile.major_version, "134");
        assert_eq!(profile.full_version, "134.0.6998.0");
        assert_eq!(profile.ua_platform, "macOS");
        assert_eq!(profile.navigator_platform, "MacIntel");
    }

    #[test]
    fn the_client_hint_brands_match_the_advertised_version() {
        let profile = StealthProfile::from_user_agent(HEADLESS, None);
        assert!(profile
            .brands()
            .iter()
            .any(|entry| entry.brand == "Google Chrome" && entry.version == "134"));
        assert!(profile
            .full_versions()
            .iter()
            .any(|entry| entry.version == "134.0.6998.0"));
    }

    #[test]
    fn a_region_sets_the_languages() {
        let profile = StealthProfile::from_user_agent(HEADLESS, Some("id"));
        assert_eq!(profile.languages, vec!["id-ID", "id", "en"]);
        assert_eq!(profile.accept_language, "id-ID,id;q=0.9,en;q=0.8");
    }

    #[test]
    fn an_empty_user_agent_falls_back_to_a_plausible_one() {
        let profile = StealthProfile::from_user_agent("", None);
        assert!(profile.user_agent.contains("Chrome/"));
        assert!(!profile.major_version.is_empty());
    }
}
