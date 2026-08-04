pub fn settle_delay(base_ms: u64, jitter_ms: u64, seed: &str) -> u64 {
    if jitter_ms == 0 {
        return base_ms;
    }
    let span = jitter_ms * 2 + 1;
    let offset = hash(seed) % span;
    let signed = offset as i64 - jitter_ms as i64;
    base_ms.saturating_add_signed(signed)
}

fn hash(seed: &str) -> u64 {
    let mut state: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in seed.as_bytes() {
        state ^= u64::from(*byte);
        state = state.wrapping_mul(0x0000_0100_0000_01b3);
    }
    state
}

#[cfg(test)]
mod tests {
    use super::settle_delay;

    #[test]
    fn zero_jitter_returns_the_base_delay() {
        assert_eq!(settle_delay(2500, 0, "https://example.com"), 2500);
    }

    #[test]
    fn the_delay_stays_inside_the_jitter_window() {
        for seed in ["a", "https://shopee.co.id", "https://example.com/x?y=1"] {
            let delay = settle_delay(2500, 500, seed);
            assert!((2000..=3000).contains(&delay), "{seed} produced {delay}");
        }
    }

    #[test]
    fn the_same_seed_always_produces_the_same_delay() {
        let seed = "https://shopee.co.id";
        assert_eq!(settle_delay(2500, 500, seed), settle_delay(2500, 500, seed));
    }

    #[test]
    fn different_seeds_can_produce_different_delays() {
        let first = settle_delay(2500, 500, "https://a.example");
        let second = settle_delay(2500, 500, "https://b.example");
        assert_ne!((first, second), (2500, 2500));
    }
}
