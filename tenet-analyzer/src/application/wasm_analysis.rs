use std::sync::Arc;

use tenet_types::{Finding, FindingKind, Severity};
use tenet_wasm::WasmReport;

use crate::domain::ports::PageFetcher;

const MAX_MODULES: usize = 4;

pub async fn analyze_wasm(fetcher: &Arc<dyn PageFetcher>, modules: &[String]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for module in modules.iter().take(MAX_MODULES) {
        match fetcher.fetch_bytes(module).await {
            Ok(bytes) => findings.extend(findings_for(module, &bytes)),
            Err(err) => {
                tracing::warn!(url = %module, error = %err, "wasm fetch failed");
                findings.push(unfetched(module));
            }
        }
    }
    findings
}

fn unfetched(module: &str) -> Finding {
    Finding {
        kind: FindingKind::Technology,
        name: "WebAssembly module".to_owned(),
        value: Some(module.to_owned()),
        severity: Severity::Info,
        confidence: 0.7,
        evidence: Some("loaded at runtime but could not be fetched for analysis".to_owned()),
    }
}

pub(crate) fn findings_for(module: &str, bytes: &[u8]) -> Vec<Finding> {
    let Ok(report) = tenet_wasm::analyze(bytes) else {
        return Vec::new();
    };
    let mut findings = vec![summary(module, &report)];
    if let Some(toolchain) = toolchain(&report) {
        findings.push(toolchain);
    }
    if is_security_module(&report) {
        findings.push(security(module, &report));
    }
    findings
}

fn summary(module: &str, report: &WasmReport) -> Finding {
    Finding {
        kind: FindingKind::Technology,
        name: "WebAssembly module".to_owned(),
        value: Some(module.to_owned()),
        severity: Severity::Info,
        confidence: 0.95,
        evidence: Some(format!(
            "{} functions, {} imports, {} exports, host modules {:?}",
            report.function_count,
            report.imports.len(),
            report.exports.len(),
            report.import_modules()
        )),
    }
}

fn toolchain(report: &WasmReport) -> Option<Finding> {
    let toolchain = report.toolchain.as_ref()?;
    Some(Finding {
        kind: FindingKind::Technology,
        name: "WASM toolchain".to_owned(),
        value: Some(toolchain.clone()),
        severity: Severity::Info,
        confidence: 0.8,
        evidence: Some("read from the wasm producers section".to_owned()),
    })
}

fn is_security_module(report: &WasmReport) -> bool {
    let has_fingerprint = report
        .signals
        .iter()
        .any(|signal| signal.kind == "fingerprint-signal");
    let has_token = report
        .signals
        .iter()
        .any(|signal| signal.kind == "token-surface");
    has_fingerprint && has_token
}

fn security(module: &str, report: &WasmReport) -> Finding {
    let host_calls = report
        .signals
        .iter()
        .filter(|signal| signal.kind == "host-call")
        .count();
    Finding {
        kind: FindingKind::Technology,
        name: "WASM anti-fraud module".to_owned(),
        value: Some("bot-protection".to_owned()),
        severity: Severity::Medium,
        confidence: 0.85,
        evidence: Some(format!(
            "{module} fingerprints the client and mints a token ({host_calls} host calls)"
        )),
    }
}

#[cfg(test)]
mod tests {
    use tenet_wasm::analyze;

    use super::{is_security_module, summary};

    const RKM_SEC: &[u8] = include_bytes!("../../../tenet-wasm/tests/fixtures/rkm_sec.wasm");

    fn report() -> tenet_wasm::WasmReport {
        match analyze(RKM_SEC) {
            Ok(report) => report,
            Err(err) => unreachable!("fixture must parse: {err}"),
        }
    }

    #[test]
    fn the_rkm_module_is_recognised_as_a_security_module() {
        assert!(is_security_module(&report()));
    }

    #[test]
    fn the_summary_names_the_host_import_module() {
        let finding = summary("https://x.test/rkm_sec.wasm", &report());
        assert!(finding
            .evidence
            .is_some_and(|evidence| evidence.contains("rkm_host")));
    }
}
