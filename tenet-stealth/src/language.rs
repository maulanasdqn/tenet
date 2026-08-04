const FALLBACK_LANGUAGE: &str = "en-US,en;q=0.9";

pub fn accept_language(region: Option<&str>) -> String {
    let Some(region) = region.filter(|region| region.len() == 2) else {
        return FALLBACK_LANGUAGE.to_owned();
    };
    let language = language_of(region);
    let country = region.to_uppercase();
    format!("{language}-{country},{language};q=0.9,en;q=0.8")
}

pub fn navigator_languages(region: Option<&str>) -> Vec<String> {
    accept_language(region)
        .split(',')
        .map(|tag| tag.split(';').next().unwrap_or(tag).to_owned())
        .collect()
}

fn language_of(region: &str) -> String {
    match region {
        "id" => "id".to_owned(),
        "br" => "pt".to_owned(),
        "mx" | "ar" | "es" | "cl" | "co" | "pe" => "es".to_owned(),
        "jp" => "ja".to_owned(),
        "kr" => "ko".to_owned(),
        "th" => "th".to_owned(),
        "vn" => "vi".to_owned(),
        "cn" | "tw" | "hk" => "zh".to_owned(),
        "gb" | "us" | "au" | "ca" | "nz" | "ie" | "sg" | "ph" => "en".to_owned(),
        other => other.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::{accept_language, navigator_languages};

    #[test]
    fn the_browser_speaks_the_language_of_the_region() {
        assert_eq!(accept_language(Some("id")), "id-ID,id;q=0.9,en;q=0.8");
        assert_eq!(accept_language(Some("br")), "pt-BR,pt;q=0.9,en;q=0.8");
        assert_eq!(accept_language(Some("mx")), "es-MX,es;q=0.9,en;q=0.8");
    }

    #[test]
    fn the_script_gets_the_same_languages_as_the_header() {
        assert_eq!(navigator_languages(Some("id")), vec!["id-ID", "id", "en"]);
        assert_eq!(navigator_languages(None), vec!["en-US", "en"]);
    }

    #[test]
    fn without_a_region_it_stays_english() {
        assert_eq!(accept_language(None), "en-US,en;q=0.9");
        assert_eq!(accept_language(Some("auto")), "en-US,en;q=0.9");
    }
}
