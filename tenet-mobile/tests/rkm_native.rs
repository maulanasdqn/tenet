use tenet_mobile::{analyze_binary, analyze_so};

const LIBRKMSEC: &[u8] = include_bytes!("fixtures/librkmsec.so");
const RKM_APK: &[u8] = include_bytes!("fixtures/rkm-market.apk");

fn report() -> tenet_mobile::NativeReport {
    match analyze_so(LIBRKMSEC) {
        Ok(report) => report,
        Err(err) => unreachable!("the fixture must parse: {err}"),
    }
}

#[test]
fn the_so_is_recognised_as_an_arm64_elf() {
    let report = report();
    assert_eq!(report.format, "elf");
    assert_eq!(report.arch, "aarch64");
}

#[test]
fn the_jni_bridge_is_recovered() {
    let report = report();
    let methods = report.jni_methods();
    assert!(methods.contains(&"Java_co_id_rkm_security_DeviceGuard_deviceToken"));
    assert!(methods.contains(&"Java_co_id_rkm_security_DeviceGuard_signRequest"));
    assert!(methods.contains(&"Java_co_id_rkm_security_DeviceGuard_integrityCheck"));
}

#[test]
fn the_embedded_secrets_and_endpoint_are_extracted() {
    let strings = report().strings;
    assert!(strings
        .iter()
        .any(|value| value.contains("rkm-sec-native/1.0")));
    assert!(strings
        .iter()
        .any(|value| value.contains("RKM_NATIVE_SIGNING_KEY")));
    assert!(strings
        .iter()
        .any(|value| value.contains("api.market.rajawalikaryamulya.co.id")));
}

#[test]
fn the_security_behaviour_is_flagged() {
    let report = report();
    for kind in [
        "jni-export",
        "fingerprint",
        "request-signing",
        "root-detection",
        "emulator-detection",
        "tls-pinning",
    ] {
        assert!(report.has_signal(kind), "missing signal: {kind}");
    }
}

#[test]
fn the_apk_is_unpacked_and_its_native_lib_reversed() {
    let report = analyze_binary("rkm-market.apk", RKM_APK);
    assert_eq!(report.libraries, 1);
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.name == "native anti-fraud module"));
    assert!(report
        .endpoints
        .iter()
        .any(|endpoint| endpoint.path == "/v1/sec/report"));
}
