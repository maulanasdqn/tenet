use std::io::{Cursor, Read};
use std::path::Path;

use zip::ZipArchive;

const MAX_LIBS: usize = 16;
const MAX_LIB_BYTES: u64 = 64 * 1024 * 1024;

pub struct NativeLib {
    pub name: String,
    pub bytes: Vec<u8>,
}

pub fn is_apk(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06")
}

pub fn native_libs(bytes: &[u8]) -> Vec<NativeLib> {
    let Ok(mut archive) = ZipArchive::new(Cursor::new(bytes)) else {
        return Vec::new();
    };
    let mut libs = Vec::new();
    for index in 0..archive.len() {
        if libs.len() >= MAX_LIBS {
            break;
        }
        let Ok(mut entry) = archive.by_index(index) else {
            continue;
        };
        let name = entry.name().to_owned();
        if !is_native_lib(&name) || entry.size() > MAX_LIB_BYTES {
            continue;
        }
        let mut buffer = Vec::new();
        if entry.read_to_end(&mut buffer).is_ok() {
            libs.push(NativeLib {
                name,
                bytes: buffer,
            });
        }
    }
    libs
}

fn is_native_lib(name: &str) -> bool {
    name.starts_with("lib/")
        && Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("so"))
}

#[cfg(test)]
mod tests {
    use super::{is_apk, is_native_lib};

    #[test]
    fn a_zip_header_is_recognised_as_an_apk() {
        assert!(is_apk(b"PK\x03\x04rest"));
        assert!(!is_apk(b"\x7fELF"));
    }

    #[test]
    fn only_android_lib_paths_are_native_libs() {
        assert!(is_native_lib("lib/arm64-v8a/librkmsec.so"));
        assert!(!is_native_lib("assets/config.json"));
        assert!(!is_native_lib("classes.dex"));
    }
}
