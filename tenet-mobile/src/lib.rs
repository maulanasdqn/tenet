pub mod apk;
pub mod findings;
pub mod native;
pub mod report;
pub mod signals;
pub mod strings;

use std::path::Path;

use tenet_types::{Endpoint, Finding};

pub use native::analyze_so;
pub use report::{MobileError, NativeReport, NativeSymbol, Signal};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryFormat {
    Apk,
    Ipa,
}

impl BinaryFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            BinaryFormat::Apk => "apk",
            BinaryFormat::Ipa => "ipa",
        }
    }

    pub fn from_reference(reference: &str) -> Option<Self> {
        let extension = Path::new(reference).extension()?.to_str()?;
        if extension.eq_ignore_ascii_case("apk") {
            return Some(BinaryFormat::Apk);
        }
        if extension.eq_ignore_ascii_case("ipa") {
            return Some(BinaryFormat::Ipa);
        }
        None
    }
}

#[derive(Debug, Clone, Default)]
pub struct BinaryReport {
    pub reference: String,
    pub libraries: usize,
    pub findings: Vec<Finding>,
    pub endpoints: Vec<Endpoint>,
}

pub fn analyze_binary(reference: &str, bytes: &[u8]) -> BinaryReport {
    let libraries = collect_libraries(reference, bytes);
    let mut report = BinaryReport {
        reference: reference.to_owned(),
        libraries: libraries.len(),
        ..BinaryReport::default()
    };
    for (name, native) in &libraries {
        report
            .findings
            .extend(findings::library_findings(name, native));
        report.endpoints.extend(findings::library_endpoints(native));
    }
    report
}

fn collect_libraries(reference: &str, bytes: &[u8]) -> Vec<(String, NativeReport)> {
    if apk::is_apk(bytes) {
        return apk::native_libs(bytes)
            .into_iter()
            .filter_map(|lib| analyze_so(&lib.bytes).ok().map(|report| (lib.name, report)))
            .collect();
    }
    match analyze_so(bytes) {
        Ok(report) => vec![(reference.to_owned(), report)],
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::BinaryFormat;

    #[test]
    fn an_apk_reference_is_recognised() {
        assert_eq!(
            BinaryFormat::from_reference("/tmp/App-release.APK"),
            Some(BinaryFormat::Apk)
        );
        assert_eq!(BinaryFormat::from_reference("com.example.app"), None);
    }
}
