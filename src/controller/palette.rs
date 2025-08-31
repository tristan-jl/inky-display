use super::f32_to_u8;
use super::space::{ColourSpace, EuclideanDistance};
use anyhow::Result;
use anyhow::anyhow;

/// Palette
/// Represents a colour palette as a collection of RGB colours.
#[derive(Debug, Clone)]
pub struct Palette(Vec<[u8; 3]>);

impl From<&[[u8; 3]]> for Palette {
    fn from(value: &[[u8; 3]]) -> Self {
        Self(value.to_vec())
    }
}
impl From<Vec<[u8; 3]>> for Palette {
    fn from(value: Vec<[u8; 3]>) -> Self {
        value.as_slice().into()
    }
}

impl Palette {
    /// Creates a new Palette by blending 2 palettes together based on the given saturation.
    ///
    /// # Errors
    ///
    /// Returns an error if saturation is not 0 <= s <= 1.
    pub fn from_blend(
        desat_palette: &[[u8; 3]],
        sat_palette: &[[u8; 3]],
        saturation: f32,
    ) -> Result<Self> {
        if !(0.0..=1.0).contains(&saturation) {
            return Err(anyhow!(
                "Saturation should be between 0 and 1, got: {}",
                saturation
            ));
        }

        let mut res = desat_palette.to_vec();
        for (r, (d, s)) in res
            .iter_mut()
            .zip(desat_palette.iter().zip(sat_palette.iter()))
        {
            *r = [
                f32_to_u8(f32::from(d[0]) * (1.0 - saturation) + f32::from(s[0]) * saturation),
                f32_to_u8(f32::from(d[1]) * (1.0 - saturation) + f32::from(s[1]) * saturation),
                f32_to_u8(f32::from(d[2]) * (1.0 - saturation) + f32::from(s[2]) * saturation),
            ];
        }
        Ok(res.into())
    }

    /// Returns the palette colours.
    #[must_use]
    pub fn get_colours(&self) -> &[[u8; 3]] {
        &self.0
    }

    /// Finds the closest colour in the palette to a given pixel using the specified colour space.
    #[must_use]
    pub fn closest_colour(&self, space: ColourSpace, pixel: &[u8; 3]) -> [u8; 3] {
        let closest_colour_idx = closest_colour_h(&self.0, space, *pixel);

        self.0[closest_colour_idx.0]
    }

    /// Returns the index of the palette colour of the pixel provided.
    ///
    /// If the pixel isn't a palette colour, returns 0.
    #[must_use]
    pub fn to_idx(&self, pixel: &[u8; 3]) -> u8 {
        for (i, c) in self.0.iter().enumerate() {
            if *c == *pixel {
                return i as u8;
            }
        }
        0
    }
}

fn closest_colour_h(cl: &[[u8; 3]], space: ColourSpace, pixel: [u8; 3]) -> (usize, f32) {
    let mut closest_colour_idx = 0;
    let mut closest_dist: f32 = f32::MAX;

    for (i, palette_colour) in cl.iter().enumerate() {
        let dist: f32 = space.distance_sq(*palette_colour, pixel);
        if dist < closest_dist {
            closest_colour_idx = i;
            closest_dist = dist;
        }
    }

    (closest_colour_idx, closest_dist)
}
