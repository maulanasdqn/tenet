pub mod embed;
pub mod parse;
pub mod report;
pub mod signals;
pub mod strings;

pub use embed::{extract_base64_wasm, wasm_api_calls};
pub use parse::analyze;
pub use report::{Export, Import, WasmReport};
pub use signals::Signal;

pub fn is_wasm(bytes: &[u8]) -> bool {
    bytes.starts_with(b"\0asm")
}
