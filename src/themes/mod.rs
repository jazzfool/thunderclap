//! A collection of various themes to quickly get up and running with Reui.

use {
    image::Pixel,
    reclutch::display::{Color, ResourceReference},
};

/// Replaces a specific color in an image while preserving alpha.
/// Useful to recolor a solid icon.
pub fn recolor_icon(icon: &mut image::RgbaImage, from: Color, to: Color) {
    for pixel in icon.pixels_mut() {
        let (r, g, b, a) = pixel.channels4();
        if (r, g, b)
            == (
                (from.red * 255.0) as u8,
                (from.green * 255.0) as u8,
                (from.blue * 255.0) as u8,
            )
        {
            *pixel = image::Rgba::<u8>::from_channels(
                (to.red * 255.0) as u8,
                (to.green * 255.0) as u8,
                (to.blue * 255.0) as u8,
                a,
            );
        }
    }
}

#[cfg(feature = "default-themes")]
mod primer;

/// GitHub's "Primer" theme, based off the CSS widgets.
#[cfg(feature = "default-themes")]
pub struct Primer {
    checkmark: ResourceReference,
}
