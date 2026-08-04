#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeSymbol {
    pub name: String,
    pub jni: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signal {
    pub kind: &'static str,
    pub evidence: String,
}

#[derive(Debug, Clone, Default)]
pub struct NativeReport {
    pub format: &'static str,
    pub arch: &'static str,
    pub exports: Vec<NativeSymbol>,
    pub imports: Vec<String>,
    pub needed_libs: Vec<String>,
    pub strings: Vec<String>,
    pub signals: Vec<Signal>,
}

impl NativeReport {
    pub fn jni_methods(&self) -> Vec<&str> {
        self.exports
            .iter()
            .filter(|symbol| symbol.jni)
            .map(|symbol| symbol.name.as_str())
            .collect()
    }

    pub fn has_signal(&self, kind: &str) -> bool {
        self.signals.iter().any(|signal| signal.kind == kind)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MobileError(pub String);

impl std::fmt::Display for MobileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "not a parseable native binary: {}", self.0)
    }
}
