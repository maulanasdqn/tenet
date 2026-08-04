use crate::signals::Signal;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Import {
    pub module: String,
    pub name: String,
    pub kind: &'static str,
}

impl Import {
    pub fn qualified(&self) -> String {
        format!("{}.{}", self.module, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Export {
    pub name: String,
    pub kind: &'static str,
}

#[derive(Debug, Clone, Default)]
pub struct WasmReport {
    pub version: u32,
    pub function_count: usize,
    pub memory_pages: Option<u64>,
    pub data_segments: usize,
    pub toolchain: Option<String>,
    pub imports: Vec<Import>,
    pub exports: Vec<Export>,
    pub strings: Vec<String>,
    pub signals: Vec<Signal>,
}

impl WasmReport {
    pub fn import_modules(&self) -> Vec<String> {
        let mut modules: Vec<String> = self
            .imports
            .iter()
            .map(|import| import.module.clone())
            .collect();
        modules.sort();
        modules.dedup();
        modules
    }

    pub fn exported_functions(&self) -> Vec<&str> {
        self.exports
            .iter()
            .filter(|export| export.kind == "func")
            .map(|export| export.name.as_str())
            .collect()
    }
}
