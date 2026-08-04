use crate::report::{NativeSymbol, Signal};

struct Marker {
    kind: &'static str,
    needles: &'static [&'static str],
}

const MARKERS: &[Marker] = &[
    Marker {
        kind: "fingerprint",
        needles: &["fingerprint", "device", "sensor", "canvas"],
    },
    Marker {
        kind: "request-signing",
        needles: &["sign", "hmac", "signature", "secret", "signing_key"],
    },
    Marker {
        kind: "root-detection",
        needles: &["root", "magisk", "supersu", "superuser", "/su"],
    },
    Marker {
        kind: "emulator-detection",
        needles: &[
            "emulator",
            "goldfish",
            "ranchu",
            "qemu",
            "vbox",
            "genymotion",
        ],
    },
    Marker {
        kind: "tls-pinning",
        needles: &["sha256/", "pinning", "pinned", "x509"],
    },
];

const CRYPTO_LIBS: &[&str] = &["libcrypto", "libssl", "boringssl", "conscrypt"];

pub fn detect(exports: &[NativeSymbol], imports: &[String], strings: &[String]) -> Vec<Signal> {
    let mut signals = Vec::new();
    detect_jni(exports, &mut signals);
    detect_behaviour(exports, strings, &mut signals);
    detect_crypto(imports, &mut signals);
    signals
}

fn detect_jni(exports: &[NativeSymbol], signals: &mut Vec<Signal>) {
    for method in exports.iter().filter(|symbol| symbol.jni) {
        signals.push(Signal {
            kind: "jni-export",
            evidence: format!("exports {}", method.name),
        });
    }
}

fn detect_behaviour(exports: &[NativeSymbol], strings: &[String], signals: &mut Vec<Signal>) {
    let haystack: Vec<String> = exports
        .iter()
        .map(|symbol| symbol.name.to_lowercase())
        .chain(strings.iter().map(|value| value.to_lowercase()))
        .collect();
    for marker in MARKERS {
        if let Some(hit) = haystack
            .iter()
            .find(|value| marker.needles.iter().any(|needle| value.contains(needle)))
        {
            signals.push(Signal {
                kind: marker.kind,
                evidence: format!("references \"{}\"", truncate(hit)),
            });
        }
    }
}

fn detect_crypto(imports: &[String], signals: &mut Vec<Signal>) {
    let uses_crypto = imports.iter().any(|name| {
        CRYPTO_LIBS
            .iter()
            .any(|lib| name.to_lowercase().contains(lib))
    });
    if uses_crypto {
        signals.push(Signal {
            kind: "crypto",
            evidence: "links a native crypto library".to_owned(),
        });
    }
}

fn truncate(value: &str) -> String {
    value.chars().take(48).collect()
}

#[cfg(test)]
mod tests {
    use crate::report::NativeSymbol;

    use super::detect;

    fn export(name: &str) -> NativeSymbol {
        NativeSymbol {
            name: name.to_owned(),
            jni: name.starts_with("Java_"),
        }
    }

    #[test]
    fn a_jni_export_is_flagged() {
        let signals = detect(&[export("Java_com_x_Guard_token")], &[], &[]);
        assert!(signals.iter().any(|s| s.kind == "jni-export"));
    }

    #[test]
    fn root_and_signing_and_pinning_markers_are_flagged() {
        let strings = vec![
            "magisk".to_owned(),
            "signing_key".to_owned(),
            "sha256/AAAA".to_owned(),
        ];
        let signals = detect(&[], &[], &strings);
        for kind in ["root-detection", "request-signing", "tls-pinning"] {
            assert!(signals.iter().any(|s| s.kind == kind), "missing {kind}");
        }
    }

    #[test]
    fn a_crypto_import_is_flagged() {
        let signals = detect(&[], &["libcrypto.so".to_owned()], &[]);
        assert!(signals.iter().any(|s| s.kind == "crypto"));
    }

    #[test]
    fn an_inert_library_raises_nothing() {
        assert!(detect(
            &[export("add")],
            &["libc.so".to_owned()],
            &["hello".to_owned()]
        )
        .is_empty());
    }
}
