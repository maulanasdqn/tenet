const MIN_RUN: usize = 5;
const MAX_STRINGS: usize = 600;

pub fn printable_runs(bytes: &[u8]) -> Vec<String> {
    let mut runs = Vec::new();
    let mut current = String::new();
    for byte in bytes {
        if matches!(byte, 0x20..=0x7e) {
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

#[cfg(test)]
mod tests {
    use super::printable_runs;

    #[test]
    fn a_printable_run_is_kept_and_a_short_one_dropped() {
        let runs = printable_runs(b"\x00rkm-sec-native/1.0\x00ab\x00");
        assert!(runs.contains(&"rkm-sec-native/1.0".to_owned()));
        assert!(!runs.contains(&"ab".to_owned()));
    }
}
