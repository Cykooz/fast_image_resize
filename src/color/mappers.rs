use crate::PixelComponentMapper;

fn gamma_into_linear(input: f32) -> f32 {
    input.powf(2.2)
}

fn linear_into_gamma(input: f32) -> f32 {
    input.powf(1.0 / 2.2)
}

/// Create mapper to convert an image from Gamma 2.2 to linear colorspace and back.
pub fn create_gamma_22_mapper() -> PixelComponentMapper {
    PixelComponentMapper::new(gamma_into_linear, linear_into_gamma)
}

/// https://en.wikipedia.org/wiki/SRGB#From_sRGB_to_CIE_XYZ
/// http://www.ericbrasseur.org/gamma.html?i=2#formulas
fn srgb_to_linear(input: f32) -> f32 {
    if input < 0.04045 {
        input / 12.92
    } else {
        const A: f32 = 0.055;
        ((input + A) / (1. + A)).powf(2.4)
    }
}

/// https://en.wikipedia.org/wiki/SRGB#From_CIE_XYZ_to_sRGB
/// http://www.ericbrasseur.org/gamma.html?i=2#formulas
fn linear_to_srgb(input: f32) -> f32 {
    if input < 0.0031308 {
        12.92 * input
    } else {
        const A: f32 = 0.055;
        (1. + A) * input.powf(1. / 2.4) - A
    }
}

/// Create mapper to convert an image from sRGB to linear RGB colorspace and back.
pub fn create_srgb_mapper() -> PixelComponentMapper {
    PixelComponentMapper::new(srgb_to_linear, linear_to_srgb)
}
