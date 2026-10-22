use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// 705 600 000 ticks per second: divisible by every common frame rate
/// (24, 25, 30, 50, 60, 24000/1001, ...) and by 44.1 / 48 kHz sample rates, so
/// edit math never drifts.
pub const TICKS_PER_SECOND: i64 = 705_600_000;

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct Tick(pub i64);

impl Tick {
    pub const ZERO: Tick = Tick(0);

    pub fn from_secs(secs: f64) -> Tick {
        if secs.is_finite() {
            Tick((secs * TICKS_PER_SECOND as f64).round() as i64)
        } else {
            Tick::ZERO
        }
    }

    pub fn secs(self) -> f64 {
        self.0 as f64 / TICKS_PER_SECOND as f64
    }

    pub fn from_frames(frames: i64, fps: Fps) -> Tick {
        Tick(frames.saturating_mul(fps.ticks_per_frame()))
    }

    /// Frame index containing this tick (floor).
    pub fn frames(self, fps: Fps) -> i64 {
        self.0.div_euclid(fps.ticks_per_frame().max(1))
    }

    pub fn snap_to_frame(self, fps: Fps) -> Tick {
        let tpf = fps.ticks_per_frame().max(1);
        Tick(((self.0 as f64 / tpf as f64).round() as i64).saturating_mul(tpf))
    }

    pub fn max(self, other: Tick) -> Tick {
        Tick(self.0.max(other.0))
    }

    pub fn min(self, other: Tick) -> Tick {
        Tick(self.0.min(other.0))
    }

    pub fn clamp(self, lo: Tick, hi: Tick) -> Tick {
        Tick(self.0.clamp(lo.0, hi.0.max(lo.0)))
    }

    /// `HH:MM:SS:FF`
    pub fn timecode(self, fps: Fps) -> String {
        let total = self.frames(fps).max(0);
        let fr = fps.rounded().max(1) as i64;
        let (s, f) = (total / fr, total % fr);
        format!(
            "{:02}:{:02}:{:02}:{:02}",
            s / 3600,
            (s / 60) % 60,
            s % 60,
            f
        )
    }
}

impl Add for Tick {
    type Output = Tick;
    fn add(self, rhs: Tick) -> Tick {
        Tick(self.0.saturating_add(rhs.0))
    }
}
impl Sub for Tick {
    type Output = Tick;
    fn sub(self, rhs: Tick) -> Tick {
        Tick(self.0.saturating_sub(rhs.0))
    }
}
impl AddAssign for Tick {
    fn add_assign(&mut self, rhs: Tick) {
        *self = *self + rhs;
    }
}
impl SubAssign for Tick {
    fn sub_assign(&mut self, rhs: Tick) {
        *self = *self - rhs;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Fps {
    pub num: u32,
    pub den: u32,
}

impl Fps {
    pub const FPS_24: Fps = Fps { num: 24, den: 1 };
    pub const FPS_30: Fps = Fps { num: 30, den: 1 };
    pub const FPS_60: Fps = Fps { num: 60, den: 1 };

    pub fn ticks_per_frame(self) -> i64 {
        TICKS_PER_SECOND * self.den.max(1) as i64 / self.num.max(1) as i64
    }

    pub fn as_f64(self) -> f64 {
        self.num as f64 / self.den.max(1) as f64
    }

    pub fn rounded(self) -> u32 {
        self.as_f64().round() as u32
    }
}

impl Default for Fps {
    fn default() -> Self {
        Fps::FPS_30
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_rates_divide_evenly() {
        for fps in [
            Fps::FPS_24,
            Fps::FPS_30,
            Fps::FPS_60,
            Fps {
                num: 24000,
                den: 1001,
            },
            Fps {
                num: 30000,
                den: 1001,
            },
        ] {
            assert_eq!(
                TICKS_PER_SECOND * fps.den as i64 % fps.num as i64,
                0,
                "{fps:?}"
            );
        }
    }

    #[test]
    fn timecode_formats() {
        let t = Tick::from_secs(61.5);
        assert_eq!(t.timecode(Fps::FPS_30), "00:01:01:15");
    }

    #[test]
    fn non_finite_seconds_do_not_poison_time() {
        assert_eq!(Tick::from_secs(f64::NAN), Tick::ZERO);
        assert_eq!(Tick::from_secs(f64::INFINITY), Tick::ZERO);
    }
}
