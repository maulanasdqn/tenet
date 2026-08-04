use crate::report::{Export, Import};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signal {
    pub kind: &'static str,
    pub evidence: String,
}

const FINGERPRINT_MARKERS: &[&str] = &[
    "canvas",
    "webgl",
    "navigator",
    "useragent",
    "screen",
    "audiocontext",
    "timezone",
    "hardwareconcurrency",
    "devicememory",
    "fingerprint",
];

const TOKEN_MARKERS: &[&str] = &["token", "device", "risk", "sec", "signature", "nonce"];

pub fn detect(imports: &[Import], exports: &[Export], strings: &[String]) -> Vec<Signal> {
    let mut signals = Vec::new();
    detect_host_calls(imports, &mut signals);
    detect_fingerprinting(strings, &mut signals);
    detect_token_surface(exports, strings, &mut signals);
    signals
}

fn detect_host_calls(imports: &[Import], signals: &mut Vec<Signal>) {
    for import in imports {
        signals.push(Signal {
            kind: "host-call",
            evidence: format!("imports {}", import.qualified()),
        });
    }
}

fn detect_fingerprinting(strings: &[String], signals: &mut Vec<Signal>) {
    for value in strings {
        let lower = value.to_lowercase();
        if FINGERPRINT_MARKERS
            .iter()
            .any(|marker| lower.contains(marker))
        {
            signals.push(Signal {
                kind: "fingerprint-signal",
                evidence: format!("references \"{value}\""),
            });
        }
    }
}

fn detect_token_surface(exports: &[Export], strings: &[String], signals: &mut Vec<Signal>) {
    let names = exports
        .iter()
        .map(|export| export.name.to_lowercase())
        .chain(strings.iter().map(|value| value.to_lowercase()));
    for name in names {
        if TOKEN_MARKERS.iter().any(|marker| name.contains(marker)) {
            signals.push(Signal {
                kind: "token-surface",
                evidence: format!("names \"{name}\""),
            });
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::report::{Export, Import};

    use super::detect;

    fn import(module: &str, name: &str) -> Import {
        Import {
            module: module.to_owned(),
            name: name.to_owned(),
            kind: "func",
        }
    }

    fn export(name: &str) -> Export {
        Export {
            name: name.to_owned(),
            kind: "func",
        }
    }

    #[test]
    fn a_host_import_is_a_host_call_signal() {
        let signals = detect(&[import("rkm_host", "host_now_ms")], &[], &[]);
        assert!(signals
            .iter()
            .any(|signal| signal.kind == "host-call" && signal.evidence.contains("host_now_ms")));
    }

    #[test]
    fn a_fingerprint_marker_string_is_flagged() {
        let signals = detect(&[], &[], &["navigator.userAgent".to_owned()]);
        assert!(signals
            .iter()
            .any(|signal| signal.kind == "fingerprint-signal"));
    }

    #[test]
    fn a_token_export_is_a_token_surface() {
        let signals = detect(&[], &[export("rkm_device_token")], &[]);
        assert!(signals.iter().any(|signal| signal.kind == "token-surface"));
    }

    #[test]
    fn an_inert_module_raises_nothing() {
        assert!(detect(&[], &[export("add")], &["hello".to_owned()]).is_empty());
    }
}
