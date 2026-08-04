pub const STEALTH_ARGS: &[&str] = &[
    "disable-blink-features=AutomationControlled",
    "disable-infobars",
    "disable-features=TranslateUI,ChromeWhatsNewUI,IsolateOrigins,site-per-process",
    "disable-background-networking",
    "disable-background-timer-throttling",
    "disable-backgrounding-occluded-windows",
    "disable-renderer-backgrounding",
    "disable-client-side-phishing-detection",
    "disable-default-apps",
    "disable-hang-monitor",
    "disable-popup-blocking",
    "disable-prompt-on-repost",
    "disable-sync",
    "no-first-run",
    "no-default-browser-check",
    "no-service-autorun",
    "password-store=basic",
    "use-mock-keychain",
    "force-color-profile=srgb",
    "metrics-recording-only",
    "mute-audio",
];

#[cfg(test)]
mod tests {
    use super::STEALTH_ARGS;

    #[test]
    fn no_flag_carries_its_own_dashes() {
        for flag in STEALTH_ARGS {
            assert!(
                !flag.starts_with('-'),
                "{flag} would reach chromium as ----{flag}"
            );
        }
    }

    #[test]
    fn the_automation_flag_is_always_present() {
        assert!(STEALTH_ARGS.contains(&"disable-blink-features=AutomationControlled"));
    }
}
