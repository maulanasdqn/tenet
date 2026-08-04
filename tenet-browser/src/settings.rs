use std::path::PathBuf;

const CANDIDATES: [&str; 6] = [
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/usr/bin/google-chrome",
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/opt/homebrew/bin/chromium",
];

#[derive(Debug, Clone)]
pub struct RenderSettings {
    pub chrome_bin: String,
    pub chrome_ws_url: String,
    pub headful: bool,
    pub no_sandbox: bool,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub nav_timeout_seconds: u64,
    pub settle_ms: u64,
    pub quiet_ms: u64,
}

impl RenderSettings {
    pub fn executable(&self) -> Option<String> {
        if !self.chrome_bin.is_empty() {
            return Some(self.chrome_bin.clone());
        }
        CANDIDATES
            .iter()
            .find(|path| PathBuf::from(path).exists())
            .map(|path| (*path).to_owned())
    }

    pub fn is_remote(&self) -> bool {
        !self.chrome_ws_url.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::RenderSettings;

    fn settings(chrome_bin: &str, chrome_ws_url: &str) -> RenderSettings {
        RenderSettings {
            chrome_bin: chrome_bin.to_owned(),
            chrome_ws_url: chrome_ws_url.to_owned(),
            headful: false,
            no_sandbox: true,
            viewport_width: 1280,
            viewport_height: 800,
            nav_timeout_seconds: 30,
            settle_ms: 1500,
            quiet_ms: 500,
        }
    }

    #[test]
    fn an_explicit_binary_wins_over_the_candidates() {
        let chosen = settings("/opt/my-chrome", "").executable();
        assert_eq!(chosen, Some("/opt/my-chrome".to_owned()));
    }

    #[test]
    fn a_websocket_url_marks_the_browser_as_remote() {
        assert!(settings("", "ws://localhost:9222/devtools/browser/x").is_remote());
        assert!(!settings("", "").is_remote());
    }
}
