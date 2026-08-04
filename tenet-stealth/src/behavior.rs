#![allow(clippy::cast_precision_loss)]

const TARGETS: usize = 6;
const WAYPOINTS: usize = 3;

#[derive(Debug, Clone, PartialEq)]
pub enum Gesture {
    Move { x: f64, y: f64 },
    Scroll { delta_y: f64 },
    Dwell { ms: u64 },
}

pub fn interaction_plan(width: u32, height: u32, seed: &str) -> Vec<Gesture> {
    let w = f64::from(width.max(1));
    let h = f64::from(height.max(1));
    let mut rng = Rng::new(seed);
    let mut plan = Vec::new();
    let mut x = w * 0.5;
    let mut y = h * 0.5;
    plan.push(Gesture::Move { x, y });

    for _ in 0..TARGETS {
        let target_x = rng.range(w * 0.08, w * 0.92);
        let target_y = rng.range(h * 0.08, h * 0.72);
        for step in 1..=WAYPOINTS {
            let progress = step as f64 / WAYPOINTS as f64;
            let px = lerp(x, target_x, progress) + rng.range(-6.0, 6.0);
            let py = lerp(y, target_y, progress) + rng.range(-6.0, 6.0);
            plan.push(Gesture::Move {
                x: px.clamp(0.0, w),
                y: py.clamp(0.0, h),
            });
        }
        x = target_x;
        y = target_y;
        plan.push(Gesture::Dwell {
            ms: rng.range_u64(40, 220),
        });
        if rng.chance(0.45) {
            plan.push(Gesture::Scroll {
                delta_y: rng.range(120.0, 520.0),
            });
            plan.push(Gesture::Dwell {
                ms: rng.range_u64(60, 260),
            });
        }
    }
    plan.push(Gesture::Dwell {
        ms: rng.range_u64(120, 400),
    });
    plan
}

fn lerp(from: f64, to: f64, progress: f64) -> f64 {
    from + (to - from) * progress
}

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: &str) -> Self {
        let mut state: u64 = 0xcbf2_9ce4_8422_2325;
        for byte in seed.bytes() {
            state ^= u64::from(byte);
            state = state.wrapping_mul(0x0000_0100_0000_01b3);
        }
        Self {
            state: state.max(1),
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + self.unit() * (hi - lo)
    }

    fn range_u64(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next_u64() % (hi - lo + 1)
    }

    fn chance(&mut self, probability: f64) -> bool {
        self.unit() < probability
    }
}

#[cfg(test)]
mod tests {
    use super::{interaction_plan, Gesture};

    const TARGET_FLOOR: usize = 10;

    #[test]
    fn the_plan_moves_scrolls_and_dwells() {
        let plan = interaction_plan(1280, 800, "https://example.com/");
        assert!(plan.iter().any(|g| matches!(g, Gesture::Move { .. })));
        assert!(plan.iter().any(|g| matches!(g, Gesture::Dwell { .. })));
        assert!(plan.len() > TARGET_FLOOR);
    }

    #[test]
    fn every_move_stays_inside_the_viewport() {
        let plan = interaction_plan(1280, 800, "seed");
        for gesture in &plan {
            if let Gesture::Move { x, y } = gesture {
                assert!((0.0..=1280.0).contains(x), "x={x}");
                assert!((0.0..=800.0).contains(y), "y={y}");
            }
        }
    }

    #[test]
    fn the_same_seed_yields_the_same_plan() {
        assert_eq!(
            interaction_plan(1280, 800, "https://a.test"),
            interaction_plan(1280, 800, "https://a.test")
        );
    }

    #[test]
    fn different_seeds_yield_different_plans() {
        assert_ne!(
            interaction_plan(1280, 800, "https://a.test"),
            interaction_plan(1280, 800, "https://b.test")
        );
    }

    #[test]
    fn a_zero_viewport_does_not_panic() {
        assert!(!interaction_plan(0, 0, "seed").is_empty());
    }
}
