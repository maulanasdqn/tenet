use tenet_types::{Endpoint, Finding, FindingKind, HttpMethod, Severity};

use crate::report::NativeReport;

pub fn library_findings(name: &str, report: &NativeReport) -> Vec<Finding> {
    let mut findings = vec![summary(name, report)];
    if is_anti_fraud(report) {
        findings.push(anti_fraud(name, report));
    }
    for kind in [
        "root-detection",
        "emulator-detection",
        "tls-pinning",
        "request-signing",
    ] {
        if report.has_signal(kind) {
            findings.push(behaviour(name, kind, report));
        }
    }
    findings
}

pub fn library_endpoints(report: &NativeReport) -> Vec<Endpoint> {
    report
        .strings
        .iter()
        .filter(|value| value.starts_with("http://") || value.starts_with("https://"))
        .filter_map(|url| as_endpoint(url))
        .collect()
}

fn summary(name: &str, report: &NativeReport) -> Finding {
    Finding {
        kind: FindingKind::Technology,
        name: "native library".to_owned(),
        value: Some(name.to_owned()),
        severity: Severity::Info,
        confidence: 0.95,
        evidence: Some(format!(
            "{} {}, {} JNI methods, {} imports, needs {:?}",
            report.arch,
            report.format,
            report.jni_methods().len(),
            report.imports.len(),
            report.needed_libs
        )),
    }
}

fn is_anti_fraud(report: &NativeReport) -> bool {
    let fingerprints = report.has_signal("fingerprint");
    let signs = report.has_signal("request-signing");
    let hardens = report.has_signal("root-detection") || report.has_signal("emulator-detection");
    (fingerprints || signs) && (hardens || report.has_signal("tls-pinning"))
}

fn anti_fraud(name: &str, report: &NativeReport) -> Finding {
    Finding {
        kind: FindingKind::Technology,
        name: "native anti-fraud module".to_owned(),
        value: Some("bot-protection".to_owned()),
        severity: Severity::Medium,
        confidence: 0.85,
        evidence: Some(format!(
            "{name} fingerprints the device, signs requests and hardens against tampering ({} JNI methods)",
            report.jni_methods().len()
        )),
    }
}

fn behaviour(name: &str, kind: &str, report: &NativeReport) -> Finding {
    let evidence = report
        .signals
        .iter()
        .find(|signal| signal.kind == kind)
        .map_or_else(|| name.to_owned(), |signal| signal.evidence.clone());
    Finding {
        kind: FindingKind::Technology,
        name: kind.to_owned(),
        value: Some("native".to_owned()),
        severity: Severity::Info,
        confidence: 0.8,
        evidence: Some(evidence),
    }
}

fn as_endpoint(url: &str) -> Option<Endpoint> {
    let rest = url.split("://").nth(1)?;
    let mut parts = rest.splitn(2, '/');
    let host = parts.next()?;
    let path = parts
        .next()
        .map_or_else(|| "/".to_owned(), |tail| format!("/{tail}"));
    let scheme = url.split("://").next()?;
    Some(
        Endpoint::new(HttpMethod::Post, path, "native-string", 0.5)
            .with_base_url(Some(format!("{scheme}://{host}"))),
    )
}

#[cfg(test)]
mod tests {
    use crate::report::{NativeReport, NativeSymbol, Signal};

    use super::{library_endpoints, library_findings};

    fn report() -> NativeReport {
        NativeReport {
            format: "elf",
            arch: "aarch64",
            exports: vec![NativeSymbol {
                name: "Java_co_id_rkm_security_DeviceGuard_deviceToken".to_owned(),
                jni: true,
            }],
            imports: Vec::new(),
            needed_libs: vec!["libc.so".to_owned()],
            strings: vec!["https://api.rkm.test/v1/sec/report".to_owned()],
            signals: vec![
                Signal {
                    kind: "fingerprint",
                    evidence: "x".to_owned(),
                },
                Signal {
                    kind: "request-signing",
                    evidence: "x".to_owned(),
                },
                Signal {
                    kind: "root-detection",
                    evidence: "magisk".to_owned(),
                },
            ],
        }
    }

    #[test]
    fn an_anti_fraud_native_lib_is_flagged() {
        let findings = library_findings("librkmsec.so", &report());
        assert!(findings
            .iter()
            .any(|finding| finding.name == "native anti-fraud module"));
        assert!(findings
            .iter()
            .any(|finding| finding.name == "root-detection"));
    }

    #[test]
    fn an_embedded_url_becomes_an_endpoint() {
        let endpoints = library_endpoints(&report());
        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].path, "/v1/sec/report");
        assert_eq!(
            endpoints[0].base_url,
            Some("https://api.rkm.test".to_owned())
        );
    }
}
