pub(crate) trait EuclideanDistance {
    fn distance_sq(&self, c1: [u8; 3], c2: [u8; 3]) -> f32;
}

/// Type for describing difference colour spaces.
///
/// Implements different distance metrics.
#[derive(Copy, Clone, Debug)]
pub enum ColourSpace {
    /// Simple RGB colour space
    RGB,
    /// Colour space designed to be perceptually uniform.
    /// <https://en.wikipedia.org/wiki/CIELAB_color_space>
    CIELAB,
}

impl EuclideanDistance for ColourSpace {
    fn distance_sq(&self, c1: [u8; 3], c2: [u8; 3]) -> f32 {
        match self {
            ColourSpace::RGB => c1
                .iter()
                .zip(c2)
                .map(|(c1_i, c2_i)| (f32::from(*c1_i) - f32::from(c2_i)).powi(2))
                .sum(),
            ColourSpace::CIELAB => {
                let cielab1 = xyz_to_cielab(rgb_to_xyz(c1));
                let cielab2 = xyz_to_cielab(rgb_to_xyz(c2));
                cielab1
                    .iter()
                    .zip(cielab2)
                    .map(|(&c1_i, c2_i)| (c1_i - c2_i).powi(2))
                    .sum()
            }
        }
    }
}

// from http://www.easyrgb.com/en/math.php#text2
fn rgb_to_xyz(input: [u8; 3]) -> [f32; 3] {
    fn f(v: f32) -> f32 {
        if v > 0.04045 {
            ((v + 0.055) / 1.055).powf(2.4)
        } else {
            v / 12.92
        }
    }
    let r = f32::from(input[0]) / 255.0;
    let g = f32::from(input[1]) / 255.0;
    let b = f32::from(input[2]) / 255.0;

    let r = f(r) * 100.0;
    let g = f(g) * 100.0;
    let b = f(b) * 100.0;

    [
        r * 0.4124 + g * 0.3576 + b * 0.1805,
        r * 0.2126 + g * 0.7152 + b * 0.0722,
        r * 0.0193 + g * 0.1192 + b * 0.9505,
    ]
}

// From https://en.wikipedia.org/wiki/CIELAB_color_space#Converting_between_CIELAB_and_CIE_XYZ_coordinates
const REF_X: f32 = 95.0489;
const REF_Y: f32 = 100.0;
const REF_Z: f32 = 108.884;

fn xyz_to_cielab(input: [f32; 3]) -> [f32; 3] {
    fn f(v: f32) -> f32 {
        if v > 0.008_856 {
            v.powf(1.0 / 3.0)
        } else {
            (7.787 * v) + (16.0 / 116.0)
        }
    }

    let x = f(input[0] / REF_X);
    let y = f(input[1] / REF_Y);
    let z = f(input[2] / REF_Z);

    [(116.0 * y) - 16.0, 500.0 * (x - y), 200.0 * (y - z)]
}

mod tests {
    #[allow(unused_imports)] // clippy can't read macros?
    use super::*;

    macro_rules! rgb_to_xyz_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, exp) = $value;
                let res = rgb_to_xyz(input);
                assert!((res[0] - exp[0]).abs() < 0.1, "{res:?} {exp:?}");
                assert!((res[1] - exp[1]).abs() < 0.1, "{res:?} {exp:?}");
                assert!((res[2] - exp[2]).abs() < 0.1, "{res:?} {exp:?}");
            }
        )*
        }
    }

    macro_rules! xyz_to_cielab_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (input, exp) = $value;
                let res = xyz_to_cielab(input);
                assert!((res[0] - exp[0]).abs() < 0.1, "{res:?} {exp:?}");
                assert!((res[1] - exp[1]).abs() < 0.1, "{res:?} {exp:?}");
                assert!((res[2] - exp[2]).abs() < 0.1, "{res:?} {exp:?}");
            }
        )*
        }
    }

    rgb_to_xyz_tests! {
        rgb_to_xyz_1: ([0,0,0],[0.0,0.0,0.0]),
        rgb_to_xyz_2: ([255, 255, 255], [95.047, 100.000, 108.883]),
        rgb_to_xyz_3: ([12, 143, 208], [21.355, 24.274, 63.222]),
    }

    xyz_to_cielab_tests! {
        xyz_to_cielab_1: ([0.0,0.0,0.0],[0.0,0.0,0.0]),
        xyz_to_cielab_2: ([95.047, 100.000, 108.883], [100.0, 0.0, 0.0]),
        xyz_to_cielab_3: ([21.355, 24.274, 63.222], [56.361, -7.939, -42.092]),
    }
}
