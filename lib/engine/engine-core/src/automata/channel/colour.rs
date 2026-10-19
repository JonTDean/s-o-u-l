//! sRGB colour currency and perceptually even ramps shared by CPU painters and the voxel shader.
use serde::{Deserialize, Serialize};

/// Display colour as sRGB bytes: the lab's colour currency across crates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Srgb(pub [u8; 3]);

/// Amber signal colour and default specimen colour.
pub const SIGNAL: Srgb = Srgb([0xff, 0xb5, 0x47]);
/// Window and stage background.
pub const BACKGROUND: Srgb = Srgb([0x0d, 0x0f, 0x10]);
/// Palette offered by the inspector: signal, ink-hi, ok, info, fault.
pub const SWATCHES: [Srgb; 5] = [
    SIGNAL,
    Srgb([0xec, 0xe6, 0xd4]),
    Srgb([0x7b, 0xd8, 0x8f]),
    Srgb([0x6f, 0xb7, 0xff]),
    Srgb([0xff, 0x6b, 0x5b]),
];

impl Default for Srgb {
    fn default() -> Self {
        SIGNAL
    }
}

impl Srgb {
    /// Linear-light RGB in 0..=1, as shaders and Oklab expect.
    pub fn linear(self) -> [f32; 3] {
        self.0.map(|u| {
            let c = f32::from(u) / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        })
    }

    /// Oklab perceptual lightness in 0..=1.
    pub fn lightness(self) -> f32 {
        oklab(self)[0]
    }
}

/// Four perceptually even stops: near-background, dim, the colour, a light tint.
pub fn ramp(colour: Srgb) -> [Srgb; 4] {
    let (bg, c, white) = (oklab(BACKGROUND), oklab(colour), oklab(Srgb([255; 3])));
    [mix(bg, c, 0.12), mix(bg, c, 0.55), c, mix(c, white, 0.45)].map(from_oklab)
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [0, 1, 2].map(|i| a[i] + (b[i] - a[i]) * t)
}

fn oklab(colour: Srgb) -> [f32; 3] {
    let [r, g, b] = colour.linear();
    let l = (0.412_221_46 * r + 0.536_332_55 * g + 0.051_445_995 * b).cbrt();
    let m = (0.211_903_5 * r + 0.680_699_5 * g + 0.107_396_96 * b).cbrt();
    let s = (0.088_302_46 * r + 0.281_718_85 * g + 0.629_978_7 * b).cbrt();
    [
        0.210_454_26 * l + 0.793_617_8 * m - 0.004_072_047 * s,
        1.977_998_5 * l - 2.428_592_2 * m + 0.450_593_7 * s,
        0.025_904_037 * l + 0.782_771_77 * m - 0.808_675_77 * s,
    ]
}

fn from_oklab([l, a, b]: [f32; 3]) -> Srgb {
    let l_ = (l + 0.396_337_78 * a + 0.215_803_76 * b).powi(3);
    let m_ = (l - 0.105_561_346 * a - 0.063_854_17 * b).powi(3);
    let s_ = (l - 0.089_484_18 * a - 1.291_485_5 * b).powi(3);
    let gamma = |x: f32| {
        let c = if x <= 0.003_130_8 {
            12.92 * x
        } else {
            1.055 * x.powf(1.0 / 2.4) - 0.055
        };
        (c.clamp(0.0, 1.0) * 255.0).round() as u8
    };
    Srgb([
        gamma(4.076_741_7 * l_ - 3.307_711_6 * m_ + 0.230_969_94 * s_),
        gamma(-1.268_438 * l_ + 2.609_757_4 * m_ - 0.341_319_38 * s_),
        gamma(-0.004_196_086_3 * l_ - 0.703_418_6 * m_ + 1.707_614_7 * s_),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramp_keeps_the_chosen_colour_as_its_third_stop() {
        for colour in SWATCHES {
            let stop = ramp(colour)[2].0;
            for (a, b) in stop.iter().zip(colour.0) {
                assert!(a.abs_diff(b) <= 1, "{colour:?} round-trips through Oklab");
            }
        }
    }

    #[test]
    fn ramp_lightness_is_strictly_increasing() {
        for colour in SWATCHES {
            let l: Vec<f32> = ramp(colour).iter().map(|s| s.lightness()).collect();
            assert!(l.windows(2).all(|w| w[0] < w[1]), "{colour:?}: {l:?}");
            assert!(l[0] < 0.3, "first stop sits near the background");
        }
    }

    #[test]
    fn linear_conversion_hits_known_points() {
        assert_eq!(Srgb([0, 0, 0]).linear(), [0.0, 0.0, 0.0]);
        let white = Srgb([255, 255, 255]).linear();
        assert!(white.iter().all(|c| (c - 1.0).abs() < 1e-6));
    }
}
