use tenet_types::{Finding, FindingKind, Severity};
use tenet_web::ScriptAsset;

use crate::application::wasm_analysis::findings_for;

const MAX_EMBEDDED: usize = 4;

pub fn embedded_findings(scripts: &[ScriptAsset]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for script in scripts {
        for bytes in tenet_wasm::extract_base64_wasm(&script.body)
            .into_iter()
            .take(MAX_EMBEDDED)
        {
            let label = format!("{} (embedded)", script.url);
            findings.extend(findings_for(&label, &bytes));
        }
        let calls = tenet_wasm::wasm_api_calls(&script.body);
        if !calls.is_empty() {
            findings.push(api_usage(&script.url, &calls));
        }
    }
    findings
}

fn api_usage(url: &str, calls: &[&str]) -> Finding {
    Finding {
        kind: FindingKind::Technology,
        name: "WebAssembly usage".to_owned(),
        value: Some(calls.join(", ")),
        severity: Severity::Info,
        confidence: 0.85,
        evidence: Some(format!("{url} calls the WebAssembly API")),
    }
}

#[cfg(test)]
mod tests {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use tenet_web::ScriptAsset;

    use super::embedded_findings;

    const RKM_SEC: &[u8] = include_bytes!("../../../tenet-wasm/tests/fixtures/rkm_sec.wasm");

    #[test]
    fn a_base64_wasm_hidden_in_a_bundle_is_reverse_engineered() {
        let encoded = STANDARD.encode(RKM_SEC);
        let script = ScriptAsset::new(
            "https://x.test/sensor.js",
            format!("var s=\"{encoded}\";WebAssembly.instantiate(s);"),
        );
        let findings = embedded_findings(&[script]);
        assert!(findings
            .iter()
            .any(|finding| finding.name == "WASM anti-fraud module"));
        assert!(findings
            .iter()
            .any(|finding| finding.name == "WebAssembly usage"));
    }

    #[test]
    fn an_ordinary_bundle_yields_no_wasm_findings() {
        let script = ScriptAsset::new("https://x.test/app.js", "const x = assemble(parts);");
        assert!(embedded_findings(&[script]).is_empty());
    }
}
