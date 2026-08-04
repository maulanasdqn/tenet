use wasmparser::{ExternalKind, Parser, Payload, TypeRef};

use crate::report::{Export, Import, WasmReport};
use crate::signals::detect;
use crate::strings::printable_runs;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmError(pub String);

impl std::fmt::Display for WasmError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "not a valid wasm module: {}", self.0)
    }
}

pub fn analyze(bytes: &[u8]) -> Result<WasmReport, WasmError> {
    if !crate::is_wasm(bytes) {
        return Err(WasmError("missing the \\0asm magic number".to_owned()));
    }

    let mut report = WasmReport::default();
    let mut data = Vec::new();
    for payload in Parser::new(0).parse_all(bytes) {
        let payload = payload.map_err(|err| WasmError(err.to_string()))?;
        absorb(payload, &mut report, &mut data);
    }

    report.strings = printable_runs(&data);
    report.signals = detect(&report.imports, &report.exports, &report.strings);
    Ok(report)
}

fn absorb(payload: Payload, report: &mut WasmReport, data: &mut Vec<u8>) {
    match payload {
        Payload::Version { num, .. } => report.version = u32::from(num),
        Payload::ImportSection(reader) => read_imports(reader, report),
        Payload::ExportSection(reader) => read_exports(reader, report),
        Payload::FunctionSection(reader) => report.function_count = reader.count() as usize,
        Payload::MemorySection(reader) => read_memory(reader, report),
        Payload::DataSection(reader) => read_data(reader, report, data),
        Payload::CustomSection(reader) => read_custom(&reader, report),
        _ => {}
    }
}

fn read_imports(reader: wasmparser::ImportSectionReader, report: &mut WasmReport) {
    for import in reader.into_imports().flatten() {
        report.imports.push(Import {
            module: import.module.to_owned(),
            name: import.name.to_owned(),
            kind: type_ref_kind(&import.ty),
        });
    }
}

fn read_exports(reader: wasmparser::ExportSectionReader, report: &mut WasmReport) {
    for export in reader.into_iter().flatten() {
        report.exports.push(Export {
            name: export.name.to_owned(),
            kind: external_kind(export.kind),
        });
    }
}

fn read_memory(reader: wasmparser::MemorySectionReader, report: &mut WasmReport) {
    if let Some(memory) = reader.into_iter().flatten().next() {
        report.memory_pages = Some(memory.initial);
    }
}

fn read_data(reader: wasmparser::DataSectionReader, report: &mut WasmReport, data: &mut Vec<u8>) {
    for segment in reader.into_iter().flatten() {
        report.data_segments += 1;
        data.extend_from_slice(segment.data);
    }
}

fn read_custom(reader: &wasmparser::CustomSectionReader, report: &mut WasmReport) {
    if reader.name() != "producers" {
        return;
    }
    let toolchain: String = reader
        .data()
        .iter()
        .filter(|byte| matches!(byte, 0x20..=0x7e))
        .map(|byte| *byte as char)
        .collect();
    if !toolchain.is_empty() {
        report.toolchain = Some(toolchain);
    }
}

fn type_ref_kind(ty: &TypeRef) -> &'static str {
    match ty {
        TypeRef::Func(_) | TypeRef::FuncExact(_) => "func",
        TypeRef::Table(_) => "table",
        TypeRef::Memory(_) => "memory",
        TypeRef::Global(_) => "global",
        TypeRef::Tag(_) => "tag",
    }
}

fn external_kind(kind: ExternalKind) -> &'static str {
    match kind {
        ExternalKind::Func | ExternalKind::FuncExact => "func",
        ExternalKind::Table => "table",
        ExternalKind::Memory => "memory",
        ExternalKind::Global => "global",
        ExternalKind::Tag => "tag",
    }
}

#[cfg(test)]
mod tests {
    use super::analyze;

    #[test]
    fn a_non_wasm_input_is_rejected() {
        assert!(analyze(b"<html></html>").is_err());
    }

    #[test]
    fn the_smallest_valid_module_parses() {
        let empty = [0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
        let report = analyze(&empty).ok().filter(|report| report.version == 1);
        assert!(report.is_some_and(|report| report.imports.is_empty()));
    }
}
