const MIN_RUN: usize = 4;
const MAX_STRINGS: usize = 400;

pub fn printable_runs(bytes: &[u8]) -> Vec<String> {
    let mut runs = Vec::new();
    let mut current = String::new();
    for byte in bytes {
        if is_printable(*byte) {
            current.push(*byte as char);
            continue;
        }
        flush(&mut current, &mut runs);
        if runs.len() >= MAX_STRINGS {
            return runs;
        }
    }
    flush(&mut current, &mut runs);
    runs
}

fn flush(current: &mut String, runs: &mut Vec<String>) {
    if current.len() >= MIN_RUN {
        runs.push(current.clone());
    }
    current.clear();
}

fn is_printable(byte: u8) -> bool {
    matches!(byte, 0x20..=0x7e)
}

#[cfg(test)]
mod tests {
    use super::printable_runs;

    #[test]
    fn a_run_of_printable_bytes_is_kept() {
        let bytes = b"\x00\x01rkm-sec/v1\x00\x02short\x00";
        let runs = printable_runs(bytes);
        assert!(runs.contains(&"rkm-sec/v1".to_owned()));
        assert!(runs.contains(&"short".to_owned()));
    }

    #[test]
    fn a_run_below_the_minimum_is_dropped() {
        assert!(printable_runs(b"\x00ab\x00").is_empty());
    }

    #[test]
    fn a_trailing_run_is_flushed() {
        assert_eq!(printable_runs(b"\x00tail-string"), vec!["tail-string"]);
    }
}
