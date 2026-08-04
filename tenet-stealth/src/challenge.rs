#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Challenge {
    pub vendor: &'static str,
    pub evidence: String,
}

struct Marker {
    vendor: &'static str,
    needle: &'static str,
}

const URL_MARKERS: &[Marker] = &[
    Marker {
        vendor: "Shopee traffic verification",
        needle: "/verify/traffic",
    },
    Marker {
        vendor: "Cloudflare",
        needle: "/cdn-cgi/challenge-platform",
    },
    Marker {
        vendor: "PerimeterX",
        needle: "/px/captcha",
    },
    Marker {
        vendor: "DataDome",
        needle: "geo.captcha-delivery.com",
    },
];

const BODY_MARKERS: &[Marker] = &[
    Marker {
        vendor: "Cloudflare",
        needle: "just a moment",
    },
    Marker {
        vendor: "Cloudflare",
        needle: "cf-challenge",
    },
    Marker {
        vendor: "Generic bot wall",
        needle: "checking your browser",
    },
    Marker {
        vendor: "Generic bot wall",
        needle: "verify you are human",
    },
    Marker {
        vendor: "Generic bot wall",
        needle: "enable javascript and cookies to continue",
    },
    Marker {
        vendor: "Akamai",
        needle: "access denied",
    },
    Marker {
        vendor: "Akamai",
        needle: "reference #",
    },
    Marker {
        vendor: "PerimeterX",
        needle: "px-captcha",
    },
    Marker {
        vendor: "DataDome",
        needle: "datadome",
    },
    Marker {
        vendor: "hold-to-confirm bot wall",
        needle: "press & hold",
    },
];

pub fn detect_challenge(url: &str, html: &str) -> Option<Challenge> {
    let lower_url = url.to_lowercase();
    if let Some(marker) = URL_MARKERS
        .iter()
        .find(|marker| lower_url.contains(marker.needle))
    {
        return Some(Challenge {
            vendor: marker.vendor,
            evidence: format!("redirected to {url}"),
        });
    }

    let lower_body = html.to_lowercase();
    BODY_MARKERS
        .iter()
        .find(|marker| lower_body.contains(marker.needle))
        .map(|marker| Challenge {
            vendor: marker.vendor,
            evidence: format!("page reads \"{}\"", marker.needle),
        })
}

#[cfg(test)]
mod tests {
    use super::detect_challenge;

    #[test]
    fn a_verification_redirect_is_detected_from_the_url() {
        let found = detect_challenge("https://shopee.co.id/verify/traffic/error?x=1", "<html>");
        assert_eq!(found.map(|c| c.vendor), Some("Shopee traffic verification"));
    }

    #[test]
    fn a_cloudflare_interstitial_is_detected_from_the_body() {
        let found = detect_challenge("https://example.com/", "<title>Just a moment...</title>");
        assert_eq!(found.map(|c| c.vendor), Some("Cloudflare"));
    }

    #[test]
    fn a_press_and_hold_wall_is_detected() {
        let found = detect_challenge("https://example.com/", "Please press & hold to confirm");
        assert_eq!(found.map(|c| c.vendor), Some("hold-to-confirm bot wall"));
    }

    #[test]
    fn an_ordinary_page_is_not_a_challenge() {
        assert!(
            detect_challenge("https://example.com/", "<html><body>welcome</body></html>").is_none()
        );
    }

    #[test]
    fn the_url_is_checked_before_the_body() {
        let found = detect_challenge(
            "https://example.com/cdn-cgi/challenge-platform/x",
            "just a moment",
        );
        assert_eq!(found.map(|c| c.evidence.contains("redirected")), Some(true));
    }
}
