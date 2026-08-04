use tenet_wasm::analyze;

const RKM_SEC: &[u8] = include_bytes!("fixtures/rkm_sec.wasm");

fn report() -> tenet_wasm::WasmReport {
    match analyze(RKM_SEC) {
        Ok(report) => report,
        Err(err) => unreachable!("the fixture must parse: {err}"),
    }
}

#[test]
fn the_fixture_is_recognised_as_wasm() {
    assert!(tenet_wasm::is_wasm(RKM_SEC));
    assert_eq!(report().version, 1);
}

#[test]
fn the_host_imports_are_recovered() {
    let modules = report().import_modules();
    assert!(modules.contains(&"rkm_host".to_owned()));
    let names: Vec<String> = report()
        .imports
        .iter()
        .map(tenet_wasm::Import::qualified)
        .collect();
    assert!(names.contains(&"rkm_host.host_now_ms".to_owned()));
    assert!(names.contains(&"rkm_host.host_entropy".to_owned()));
}

#[test]
fn the_exported_surface_is_recovered() {
    let report = report();
    let functions = report.exported_functions();
    assert!(functions.contains(&"rkm_device_token"));
    assert!(functions.contains(&"rkm_fingerprint"));
    assert!(functions.contains(&"rkm_version"));
}

#[test]
fn the_embedded_markers_are_extracted() {
    let strings = report().strings;
    assert!(strings.iter().any(|value| value.contains("rkm-sec/v1")));
    assert!(strings
        .iter()
        .any(|value| value.contains("navigator.userAgent")));
}

#[test]
fn the_security_behaviour_is_flagged() {
    let signals = report().signals;
    assert!(signals.iter().any(|signal| signal.kind == "host-call"));
    assert!(signals
        .iter()
        .any(|signal| signal.kind == "fingerprint-signal"));
    assert!(signals.iter().any(|signal| signal.kind == "token-surface"));
}
