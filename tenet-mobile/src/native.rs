use object::{Architecture, BinaryFormat, Object, ObjectSection};

use crate::report::{MobileError, NativeReport, NativeSymbol};
use crate::signals::detect;
use crate::strings::printable_runs;

pub fn analyze_so(bytes: &[u8]) -> Result<NativeReport, MobileError> {
    let file = object::File::parse(bytes).map_err(|err| MobileError(err.to_string()))?;
    let mut report = NativeReport {
        format: format_label(file.format()),
        arch: arch_label(file.architecture()),
        ..NativeReport::default()
    };

    for export in file.exports().unwrap_or_default() {
        let name = text(export.name());
        let jni = name.starts_with("Java_");
        report.exports.push(NativeSymbol { name, jni });
    }
    for import in file.imports().unwrap_or_default() {
        report.imports.push(text(import.name()));
        let library = text(import.library());
        if !library.is_empty() && !report.needed_libs.contains(&library) {
            report.needed_libs.push(library);
        }
    }

    let mut data = Vec::new();
    for section in file.sections() {
        let Ok(name) = section.name() else {
            continue;
        };
        if !name.contains("data") && !name.contains("rodata") {
            continue;
        }
        if let Ok(bytes) = section.data() {
            data.extend_from_slice(bytes);
        }
    }

    report.strings = printable_runs(&data);
    report.signals = detect(&report.exports, &report.imports, &report.strings);
    Ok(report)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn format_label(format: BinaryFormat) -> &'static str {
    match format {
        BinaryFormat::Elf => "elf",
        BinaryFormat::MachO => "mach-o",
        BinaryFormat::Pe => "pe",
        BinaryFormat::Wasm => "wasm",
        _ => "unknown",
    }
}

fn arch_label(arch: Architecture) -> &'static str {
    match arch {
        Architecture::Aarch64 | Architecture::Aarch64_Ilp32 => "aarch64",
        Architecture::Arm => "arm",
        Architecture::X86_64 | Architecture::X86_64_X32 => "x86_64",
        Architecture::I386 => "x86",
        _ => "unknown",
    }
}
