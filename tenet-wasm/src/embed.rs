use base64::engine::general_purpose::STANDARD_NO_PAD;
use base64::Engine;

const B64_MAGIC: &str = "AGFzbQ";
const MIN_MODULE_BYTES: usize = 8;
const WASM_METHODS: &[&str] = &[
    "instantiateStreaming",
    "compileStreaming",
    "instantiate",
    "compile",
    "validate",
    "Module",
    "Memory",
    "Table",
];

pub fn extract_base64_wasm(source: &str) -> Vec<Vec<u8>> {
    let mut modules = Vec::new();
    let mut cursor = 0;
    while let Some(offset) = source[cursor..].find(B64_MAGIC) {
        let start = cursor + offset;
        let end = start + base64_run_len(&source[start..]);
        if let Some(bytes) = decode(&source[start..end]) {
            if bytes.len() >= MIN_MODULE_BYTES && crate::is_wasm(&bytes) {
                modules.push(bytes);
            }
        }
        cursor = end.max(start + B64_MAGIC.len());
    }
    modules
}

pub fn wasm_api_calls(source: &str) -> Vec<&'static str> {
    if !source.contains("WebAssembly") {
        return Vec::new();
    }
    let matched: Vec<&'static str> = WASM_METHODS
        .iter()
        .copied()
        .filter(|method| source.contains(&["WebAssembly.", method].concat()))
        .collect();
    matched
        .iter()
        .copied()
        .filter(|method| {
            !matched
                .iter()
                .any(|other| other != method && other.starts_with(method))
        })
        .collect()
}

fn base64_run_len(source: &str) -> usize {
    source.bytes().take_while(|byte| is_base64(*byte)).count()
}

fn is_base64(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'=')
}

fn decode(run: &str) -> Option<Vec<u8>> {
    STANDARD_NO_PAD.decode(run.trim_end_matches('=')).ok()
}

#[cfg(test)]
mod tests {
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;

    use super::{extract_base64_wasm, wasm_api_calls};

    const RKM_SEC: &[u8] = include_bytes!("../tests/fixtures/rkm_sec.wasm");

    #[test]
    fn a_base64_wasm_blob_inside_a_script_is_recovered() {
        let encoded = STANDARD.encode(RKM_SEC);
        let script = format!("var sensor = \"{encoded}\"; loadModule(sensor);");
        let modules = extract_base64_wasm(&script);
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0], RKM_SEC);
    }

    #[test]
    fn a_script_without_embedded_wasm_yields_nothing() {
        assert!(extract_base64_wasm("const x = btoa('hello world');").is_empty());
    }

    #[test]
    fn two_embedded_modules_are_both_recovered() {
        let encoded = STANDARD.encode(RKM_SEC);
        let script = format!("a=\"{encoded}\";b=\"{encoded}\";");
        assert_eq!(extract_base64_wasm(&script).len(), 2);
    }

    #[test]
    fn the_streaming_api_is_reported_without_the_bare_name() {
        let source = "const m = await WebAssembly.instantiateStreaming(fetch(u), imports);";
        assert_eq!(wasm_api_calls(source), vec!["instantiateStreaming"]);
    }

    #[test]
    fn the_compile_and_module_apis_are_reported() {
        let source = "new WebAssembly.Module(bytes); WebAssembly.compile(bytes);";
        let calls = wasm_api_calls(source);
        assert!(calls.contains(&"Module"));
        assert!(calls.contains(&"compile"));
    }

    #[test]
    fn a_script_that_never_touches_wasm_reports_no_calls() {
        assert!(wasm_api_calls("const x = assemble(parts);").is_empty());
    }
}
