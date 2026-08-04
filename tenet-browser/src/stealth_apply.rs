use chromiumoxide::cdp::browser_protocol::emulation::{
    SetUserAgentOverrideParams, UserAgentBrandVersion, UserAgentMetadata,
};
use chromiumoxide::cdp::browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams;
use chromiumoxide::Page;
use tenet_stealth::{Brand, StealthProfile};

pub async fn apply(page: &Page, profile: &StealthProfile) {
    if let Err(err) = page.execute(user_agent_override(profile)).await {
        tracing::debug!(error = %err, "user agent override failed");
    }
    let script = tenet_stealth::init_script(profile);
    if let Err(err) = page
        .execute(AddScriptToEvaluateOnNewDocumentParams::new(script))
        .await
    {
        tracing::debug!(error = %err, "installing the stealth script failed");
    }
}

fn user_agent_override(profile: &StealthProfile) -> SetUserAgentOverrideParams {
    SetUserAgentOverrideParams {
        user_agent: profile.user_agent.clone(),
        accept_language: Some(profile.accept_language.clone()),
        platform: Some(profile.ua_platform.clone()),
        user_agent_metadata: Some(metadata(profile)),
    }
}

fn metadata(profile: &StealthProfile) -> UserAgentMetadata {
    UserAgentMetadata {
        brands: Some(brands(&profile.brands())),
        full_version_list: Some(brands(&profile.full_versions())),
        platform: profile.ua_platform.clone(),
        platform_version: profile.platform_version.clone(),
        architecture: profile.architecture.clone(),
        model: String::new(),
        mobile: false,
        bitness: Some("64".to_owned()),
        wow64: Some(false),
        form_factors: None,
    }
}

fn brands(source: &[Brand]) -> Vec<UserAgentBrandVersion> {
    source
        .iter()
        .map(|entry| UserAgentBrandVersion {
            brand: entry.brand.clone(),
            version: entry.version.clone(),
        })
        .collect()
}
