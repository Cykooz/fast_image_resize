//! Functions for changing image gamma.
use once_cell::sync::Lazy;

use crate::pixels::{U16x2, U16x3, U16x4, U8x2, U8x3, U8x4, U16, U8};
use crate::typed_image_view::{TypedImageView, TypedImageViewMut};
use crate::{ImageView, ImageViewMut, MappingError, PixelType};

use super::MappingTable;

macro_rules! gamma_table {
    ($src_type:tt, $dst_type:tt, $gamma:expr) => {{
        const TABLE_SIZE: usize = $src_type::MAX as usize + 1;
        let mut table: [$dst_type; TABLE_SIZE] = [0; TABLE_SIZE];
        table.iter_mut().enumerate().for_each(|(i, v)| {
            let signal = i as f32 / $src_type::MAX as f32;
            let power = signal.powf($gamma);
            *v = ($dst_type::MAX as f32 * power).round() as $dst_type;
        });
        table
    }};
}

static GAMMA22_U8_INTO_LINEAR_U8: Lazy<MappingTable<u8, 256>> =
    Lazy::new(|| MappingTable(gamma_table!(u8, u8, 2.2)));
static LINEAR_U8_INTO_GAMMA22_U8: Lazy<MappingTable<u8, 256>> =
    Lazy::new(|| MappingTable(gamma_table!(u8, u8, 1.0 / 2.2)));
static GAMMA22_U8_INTO_LINEAR_U16: Lazy<MappingTable<u16, 256>> =
    Lazy::new(|| MappingTable(gamma_table!(u8, u16, 2.2)));
static LINEAR_U8_INTO_GAMMA22_U16: Lazy<MappingTable<u16, 256>> =
    Lazy::new(|| MappingTable(gamma_table!(u8, u16, 1.0 / 2.2)));
static GAMMA22_U16_INTO_LINEAR_U8: Lazy<MappingTable<u8, 65536>> =
    Lazy::new(|| MappingTable(gamma_table!(u16, u8, 2.2)));
static LINEAR_U16_INTO_GAMMA22_U8: Lazy<MappingTable<u8, 65536>> =
    Lazy::new(|| MappingTable(gamma_table!(u16, u8, 1.0 / 2.2)));
static GAMMA22_U16_INTO_LINEAR_U16: Lazy<MappingTable<u16, 65536>> =
    Lazy::new(|| MappingTable(gamma_table!(u16, u16, 2.2)));
static LINEAR_U16_INTO_GAMMA22_U16: Lazy<MappingTable<u16, 65536>> =
    Lazy::new(|| MappingTable(gamma_table!(u16, u16, 1.0 / 2.2)));

/// Convert image from gamma 2.2 into linear colorspace.
pub fn gamma22_into_linear(
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
        // U8 -> U8
        (PixelType::U8, PixelType::U8) => {
            map!(U8, U8, GAMMA22_U8_INTO_LINEAR_U8);
        }
        (PixelType::U8x2, PixelType::U8x2) => {
            map!(U8x2, U8x2, GAMMA22_U8_INTO_LINEAR_U8);
        }
        (PixelType::U8x3, PixelType::U8x3) => {
            map!(U8x3, U8x3, GAMMA22_U8_INTO_LINEAR_U8);
        }
        (PixelType::U8x4, PixelType::U8x4) => {
            map!(U8x4, U8x4, GAMMA22_U8_INTO_LINEAR_U8);
        }
        // U8 -> U16
        (PixelType::U8, PixelType::U16) => {
            map!(U8, U16, GAMMA22_U8_INTO_LINEAR_U16);
        }
        (PixelType::U8x2, PixelType::U16x2) => {
            map!(U8x2, U16x2, GAMMA22_U8_INTO_LINEAR_U16);
        }
        (PixelType::U8x3, PixelType::U16x3) => {
            map!(U8x3, U16x3, GAMMA22_U8_INTO_LINEAR_U16);
        }
        (PixelType::U8x4, PixelType::U16x4) => {
            map!(U8x4, U16x4, GAMMA22_U8_INTO_LINEAR_U16);
        }
        // U16 -> U8
        (PixelType::U16, PixelType::U8) => {
            map!(U16, U8, GAMMA22_U16_INTO_LINEAR_U8);
        }
        (PixelType::U16x2, PixelType::U8x2) => {
            map!(U16x2, U8x2, GAMMA22_U16_INTO_LINEAR_U8);
        }
        (PixelType::U16x3, PixelType::U8x3) => {
            map!(U16x3, U8x3, GAMMA22_U16_INTO_LINEAR_U8);
        }
        (PixelType::U16x4, PixelType::U8x4) => {
            map!(U16x4, U8x4, GAMMA22_U16_INTO_LINEAR_U8);
        }
        // U16 -> U16
        (PixelType::U16, PixelType::U16) => {
            map!(U16, U16, GAMMA22_U16_INTO_LINEAR_U16);
        }
        (PixelType::U16x2, PixelType::U16x2) => {
            map!(U16x2, U16x2, GAMMA22_U16_INTO_LINEAR_U16);
        }
        (PixelType::U16x3, PixelType::U16x3) => {
            map!(U16x3, U16x3, GAMMA22_U16_INTO_LINEAR_U16);
        }
        (PixelType::U16x4, PixelType::U16x4) => {
            map!(U16x4, U16x4, GAMMA22_U16_INTO_LINEAR_U16);
        }
        _ => return Err(MappingError::UnsupportedCombinationOfImageTypes),
    }

    Ok(())
}

/// Convert image from linear colorspace into gamma 2.2.
pub fn linear_into_gamma22(
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
        // U8 -> U8
        (PixelType::U8, PixelType::U8) => {
            map!(U8, U8, LINEAR_U8_INTO_GAMMA22_U8);
        }
        (PixelType::U8x2, PixelType::U8x2) => {
            map!(U8x2, U8x2, LINEAR_U8_INTO_GAMMA22_U8);
        }
        (PixelType::U8x3, PixelType::U8x3) => {
            map!(U8x3, U8x3, LINEAR_U8_INTO_GAMMA22_U8);
        }
        (PixelType::U8x4, PixelType::U8x4) => {
            map!(U8x4, U8x4, LINEAR_U8_INTO_GAMMA22_U8);
        }
        // U8 -> U16
        (PixelType::U8, PixelType::U16) => {
            map!(U8, U16, LINEAR_U8_INTO_GAMMA22_U16);
        }
        (PixelType::U8x2, PixelType::U16x2) => {
            map!(U8x2, U16x2, LINEAR_U8_INTO_GAMMA22_U16);
        }
        (PixelType::U8x3, PixelType::U16x3) => {
            map!(U8x3, U16x3, LINEAR_U8_INTO_GAMMA22_U16);
        }
        (PixelType::U8x4, PixelType::U16x4) => {
            map!(U8x4, U16x4, LINEAR_U8_INTO_GAMMA22_U16);
        }
        // U16 -> U8
        (PixelType::U16, PixelType::U8) => {
            map!(U16, U8, LINEAR_U16_INTO_GAMMA22_U8);
        }
        (PixelType::U16x2, PixelType::U8x2) => {
            map!(U16x2, U8x2, LINEAR_U16_INTO_GAMMA22_U8);
        }
        (PixelType::U16x3, PixelType::U8x3) => {
            map!(U16x3, U8x3, LINEAR_U16_INTO_GAMMA22_U8);
        }
        (PixelType::U16x4, PixelType::U8x4) => {
            map!(U16x4, U8x4, LINEAR_U16_INTO_GAMMA22_U8);
        }
        // U16 -> U16
        (PixelType::U16, PixelType::U16) => {
            map!(U16, U16, LINEAR_U16_INTO_GAMMA22_U16);
        }
        (PixelType::U16x2, PixelType::U16x2) => {
            map!(U16x2, U16x2, LINEAR_U16_INTO_GAMMA22_U16);
        }
        (PixelType::U16x3, PixelType::U16x3) => {
            map!(U16x3, U16x3, LINEAR_U16_INTO_GAMMA22_U16);
        }
        (PixelType::U16x4, PixelType::U16x4) => {
            map!(U16x4, U16x4, LINEAR_U16_INTO_GAMMA22_U16);
        }
        _ => return Err(MappingError::UnsupportedCombinationOfImageTypes),
    }

    Ok(())
}
