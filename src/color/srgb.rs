//! Functions for converting image between sRGB and linear colorspace.
use once_cell::sync::Lazy;

use crate::pixels::{U16x3, U16x4, U8x3, U8x4};
use crate::typed_image_view::{TypedImageView, TypedImageViewMut};
use crate::{ImageView, ImageViewMut, MappingError, PixelType};

use super::MappingTable;

/// https://en.wikipedia.org/wiki/SRGB#From_sRGB_to_CIE_XYZ
/// http://www.ericbrasseur.org/gamma.html?i=2#formulas
macro_rules! srgb_into_rgb_table {
    ($src_type:tt, $dst_type:tt) => {{
        const TABLE_SIZE: usize = $src_type::MAX as usize + 1;
        let mut table: [$dst_type; TABLE_SIZE] = [0; TABLE_SIZE];
        table.iter_mut().enumerate().for_each(|(i, v)| {
            let signal = i as f32 / $src_type::MAX as f32;
            let power = if signal < 0.04045 {
                signal / 12.92
            } else {
                const A: f32 = 0.055;
                ((signal + A) / (1. + A)).powf(2.4)
            };
            *v = ($dst_type::MAX as f32 * power).round() as $dst_type;
        });
        table
    }};
}

/// https://en.wikipedia.org/wiki/SRGB#From_CIE_XYZ_to_sRGB
/// http://www.ericbrasseur.org/gamma.html?i=2#formulas
macro_rules! rgb_into_srgb_table {
    ($src_type:tt, $dst_type:tt) => {{
        const TABLE_SIZE: usize = $src_type::MAX as usize + 1;
        let mut table: [$dst_type; TABLE_SIZE] = [0; TABLE_SIZE];
        table.iter_mut().enumerate().for_each(|(i, v)| {
            let signal = i as f32 / $src_type::MAX as f32;
            let power = if signal < 0.0031308 {
                12.92 * signal
            } else {
                const A: f32 = 0.055;
                (1. + A) * signal.powf(1. / 2.4) - A
            };
            *v = ($dst_type::MAX as f32 * power).round() as $dst_type;
        });
        table
    }};
}

static SRGB8_INTO_RGB8: Lazy<MappingTable<u8, 256>> =
    Lazy::new(|| MappingTable(srgb_into_rgb_table!(u8, u8)));
static SRGB8_INTO_RGB16: Lazy<MappingTable<u16, 256>> =
    Lazy::new(|| MappingTable(srgb_into_rgb_table!(u8, u16)));
static SRGB16_INTO_RGB8: Lazy<MappingTable<u8, 65536>> =
    Lazy::new(|| MappingTable(srgb_into_rgb_table!(u16, u8)));
static SRGB16_INTO_RGB16: Lazy<MappingTable<u16, 65536>> =
    Lazy::new(|| MappingTable(srgb_into_rgb_table!(u16, u16)));
static RGB8_INTO_SRGB8: Lazy<MappingTable<u8, 256>> =
    Lazy::new(|| MappingTable(rgb_into_srgb_table!(u8, u8)));
static RGB8_INTO_SRGB16: Lazy<MappingTable<u16, 256>> =
    Lazy::new(|| MappingTable(rgb_into_srgb_table!(u8, u16)));
static RGB16_INTO_SRGB8: Lazy<MappingTable<u8, 65536>> =
    Lazy::new(|| MappingTable(rgb_into_srgb_table!(u16, u8)));
static RGB16_INTO_SRGB16: Lazy<MappingTable<u16, 65536>> =
    Lazy::new(|| MappingTable(rgb_into_srgb_table!(u16, u16)));

/// Convert image from sRGB into linear RGB colorspace.
pub fn srgb_into_rgb(
    src_image: &ImageView,
    dst_image: &mut ImageViewMut,
) -> Result<(), MappingError> {
    if src_image.width() != dst_image.width() || src_image.height() != dst_image.height() {
        return Err(MappingError::DifferentDimensions);
    }

    macro_rules! map {
        ($src_pixel:ty, $dst_pixel:ty, $mapping_table:ident) => {
            if let Some(src) = TypedImageView::<$src_pixel>::from_image_view(src_image) {
                if let Some(dst) = TypedImageViewMut::<$dst_pixel>::from_image_view(dst_image) {
                    $mapping_table.map_typed_image(src, dst);
                }
            }
        };
    }

    let src_pixel_type = src_image.pixel_type();
    let dst_pixel_type = dst_image.pixel_type();
    match (src_pixel_type, dst_pixel_type) {
        (PixelType::U8x3, PixelType::U8x3) => {
            map!(U8x3, U8x3, SRGB8_INTO_RGB8);
        }
        (PixelType::U8x3, PixelType::U16x3) => {
            map!(U8x3, U16x3, SRGB8_INTO_RGB16);
        }
        (PixelType::U16x3, PixelType::U8x3) => {
            map!(U16x3, U8x3, SRGB16_INTO_RGB8);
        }
        (PixelType::U16x3, PixelType::U16x3) => {
            map!(U16x3, U16x3, SRGB16_INTO_RGB16);
        }
        (PixelType::U8x4, PixelType::U8x4) => {
            map!(U8x4, U8x4, SRGB8_INTO_RGB8);
        }
        (PixelType::U8x4, PixelType::U16x4) => {
            map!(U8x4, U16x4, SRGB8_INTO_RGB16);
        }
        (PixelType::U16x4, PixelType::U8x4) => {
            map!(U16x4, U8x4, SRGB16_INTO_RGB8);
        }
        (PixelType::U16x4, PixelType::U16x4) => {
            map!(U16x4, U16x4, SRGB16_INTO_RGB16);
        }
        _ => return Err(MappingError::UnsupportedCombinationOfImageTypes),
    }

    Ok(())
}

/// Convert image from linear RGB into sRGB colorspace.
pub fn rgb_into_srgb(
    src_image: &ImageView,
    dst_image: &mut ImageViewMut,
) -> Result<(), MappingError> {
    if src_image.width() != dst_image.width() || src_image.height() != dst_image.height() {
        return Err(MappingError::DifferentDimensions);
    }

    macro_rules! map {
        ($src_pixel:ty, $dst_pixel:ty, $mapping_table:ident) => {
            if let Some(src) = TypedImageView::<$src_pixel>::from_image_view(src_image) {
                if let Some(dst) = TypedImageViewMut::<$dst_pixel>::from_image_view(dst_image) {
                    $mapping_table.map_typed_image(src, dst);
                }
            }
        };
    }

    let src_pixel_type = src_image.pixel_type();
    let dst_pixel_type = dst_image.pixel_type();
    match (src_pixel_type, dst_pixel_type) {
        (PixelType::U8x3, PixelType::U8x3) => {
            map!(U8x3, U8x3, RGB8_INTO_SRGB8);
        }
        (PixelType::U8x3, PixelType::U16x3) => {
            map!(U8x3, U16x3, RGB8_INTO_SRGB16);
        }
        (PixelType::U16x3, PixelType::U8x3) => {
            map!(U16x3, U8x3, RGB16_INTO_SRGB8);
        }
        (PixelType::U16x3, PixelType::U16x3) => {
            map!(U16x3, U16x3, RGB16_INTO_SRGB16);
        }
        (PixelType::U8x4, PixelType::U8x4) => {
            map!(U8x4, U8x4, RGB8_INTO_SRGB8);
        }
        (PixelType::U8x4, PixelType::U16x4) => {
            map!(U8x4, U16x4, RGB8_INTO_SRGB16);
        }
        (PixelType::U16x4, PixelType::U8x4) => {
            map!(U16x4, U8x4, RGB16_INTO_SRGB8);
        }
        (PixelType::U16x4, PixelType::U16x4) => {
            map!(U16x4, U16x4, RGB16_INTO_SRGB16);
        }
        _ => return Err(MappingError::UnsupportedCombinationOfImageTypes),
    }

    Ok(())
}
