//! Colour grade of a picture clip: warmer or cooler (temperature) and green or magenta (tint).

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Grade {
    /// -1 cool (blue) .. 1 warm (orange).
    pub temperature: f32,
    /// -1 green .. 1 magenta.
    pub tint: f32,
}

impl Grade {
    pub fn is_neutral(&self) -> bool {
        self.temperature == 0.0 && self.tint == 0.0
    }

    pub fn sanitized(mut self) -> Self {
        let fix = |v: f32| {
            if v.is_finite() {
                v.clamp(-1.0, 1.0)
            } else {
                0.0
            }
        };
        self.temperature = fix(self.temperature);
        self.tint = fix(self.tint);
        self
    }

    /// Per-channel gains for the shader. They keep the brightness: a warmer picture is not
    /// also a brighter one.
    pub fn gains(&self) -> [f32; 3] {
        let g = self.sanitized();
        let (t, k) = (g.temperature, g.tint);
        let rgb = [
            1.0 + 0.30 * t + 0.10 * k,
            1.0 + 0.02 * t - 0.25 * k,
            1.0 - 0.30 * t + 0.10 * k,
        ];
        let luma = 0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2];
        let luma = if luma > 1e-3 { luma } else { 1.0 };
        [rgb[0] / luma, rgb[1] / luma, rgb[2] / luma]
    }

    /// Apply only what changed between `old` and `new` (several clips edited together).
    pub fn apply_diff(&mut self, old: &Grade, new: &Grade) {
        if old.temperature != new.temperature {
            self.temperature = new.temperature;
        }
        if old.tint != new.tint {
            self.tint = new.tint;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neutral_changes_nothing() {
        assert_eq!(Grade::default().gains(), [1.0, 1.0, 1.0]);
        assert!(Grade::default().is_neutral());
    }

    #[test]
    fn warm_pushes_red_up_and_blue_down() {
        let [r, _, b] = Grade {
            temperature: 1.0,
            tint: 0.0,
        }
        .gains();
        assert!(r > 1.0 && b < 1.0);
        let [r, _, b] = Grade {
            temperature: -1.0,
            tint: 0.0,
        }
        .gains();
        assert!(r < 1.0 && b > 1.0);
    }

    #[test]
    fn magenta_lowers_green_and_brightness_stays() {
        let g = Grade {
            temperature: 0.4,
            tint: 1.0,
        };
        let [r, gr, b] = g.gains();
        assert!(gr < r && gr < b);
        let luma = 0.2126 * r + 0.7152 * gr + 0.0722 * b;
        assert!((luma - 1.0).abs() < 1e-4);
    }

    #[test]
    fn nonsense_is_harmless() {
        let g = Grade {
            temperature: f32::NAN,
            tint: 50.0,
        }
        .gains();
        assert!(g.iter().all(|v| v.is_finite() && *v > 0.0));
    }
}
