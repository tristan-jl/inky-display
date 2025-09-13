mod epd;
mod inky;
mod palette;
mod space;

use image::Rgb;
use image::RgbImage;

pub use epd::EPDType;
pub use inky::Inky;
pub use inky::InkyColour;
pub use inky::LedState;
pub use palette::Palette;
pub use space::ColourSpace;

#[derive(Debug)]
#[repr(C)]
struct PascalString {
    len: u8,
    pub chars: [u8; u8::MAX as usize],
}

impl PascalString {
    fn with_len(len: u8) -> Self {
        Self {
            len,
            chars: [0; u8::MAX as usize],
        }
    }
}

impl std::fmt::Display for PascalString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let x = str::from_utf8(&self.chars).unwrap();
        writeln!(f, "{x}")
    }
}

#[allow(clippy::cast_sign_loss)]
#[allow(clippy::cast_possible_truncation)]
fn f32_to_u8(input: f32) -> u8 {
    input.round().clamp(0.0, 255.0) as u8
}

/// Quantises an image to the nearest colours in the given colour space and given palette.
pub fn quantise_image(buf: &mut RgbImage, palette: &Palette, space: ColourSpace) {
    buf.pixels_mut().for_each(|pixel| {
        *pixel = Rgb(palette.closest_colour(space, &pixel.0));
    });
}

/// Quantises an image using the given palette and colour space and applies Floyd–Steinberg dithering.
pub fn quantise_and_dither_image(buf: &mut RgbImage, palette: &Palette, space: ColourSpace) {
    let (max_width, max_height) = buf.dimensions();

    for x in 0..max_width {
        for y in 0..max_height {
            fn helper(input: f32, quant_err: f32, factor: f32) -> u8 {
                f32_to_u8(input + quant_err * factor)
            }

            let old_pixel = buf[(x, y)].0;
            buf[(x, y)] = Rgb(palette.closest_colour(space, &old_pixel));

            if x > 0 && x < max_width - 1 && y < max_height - 1 {
                let mut right_pixel = buf[(x + 1, y)];
                let mut bottom_left_pixel = buf[(x - 1, y + 1)];
                let mut bottom_pixel = buf[(x, y + 1)];
                let mut bottom_right_pixel = buf[(x + 1, y + 1)];

                for ((i, &old), &new) in old_pixel.iter().enumerate().zip(old_pixel.iter()) {
                    let quant_err = f32::from(old) - f32::from(new);
                    right_pixel.0[i] = helper(f32::from(right_pixel.0[i]), quant_err, 7.0 / 16.0);
                    bottom_left_pixel.0[i] =
                        helper(f32::from(bottom_left_pixel.0[i]), quant_err, 3.0 / 16.0);
                    bottom_pixel.0[i] = helper(f32::from(bottom_pixel.0[i]), quant_err, 5.0 / 16.0);
                    bottom_right_pixel.0[i] =
                        helper(f32::from(bottom_right_pixel.0[i]), quant_err, 1.0 / 16.0);
                }
                buf[(x + 1, y)] = right_pixel;
                buf[(x - 1, y + 1)] = bottom_left_pixel;
                buf[(x, y + 1)] = bottom_pixel;
                buf[(x + 1, y + 1)] = bottom_right_pixel;
            }
        }
    }
}
