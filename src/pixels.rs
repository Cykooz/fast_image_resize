//! Contains types of pixels.
use std::mem::size_of;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelType {
    U8x3,
    U8x4,
    I32,
    F32,
    U8,
}

impl PixelType {
    pub(crate) fn size(&self) -> usize {
        match self {
            Self::U8x3 => 3,
            Self::U8 => 1,
            _ => 4,
        }
    }

    /// Returns `true` is given buffer is aligned by the alignment of pixel.
    pub(crate) fn is_aligned(&self, buffer: &[u8]) -> bool {
        match self {
            Self::U8x3 => unsafe { buffer.align_to::<U8x3>().0.is_empty() },
            Self::U8x4 => unsafe { buffer.align_to::<U8x4>().0.is_empty() },
            Self::I32 => unsafe { buffer.align_to::<I32>().0.is_empty() },
            Self::F32 => unsafe { buffer.align_to::<F32>().0.is_empty() },
            Self::U8 => true,
        }
    }
}

/// Additional information about pixel type.
pub trait Pixel
where
    Self: Copy + Sized,
{
    fn pixel_type() -> PixelType;

    /// Size of pixel in bytes
    ///
    /// Example:
    /// ```
    /// # use fast_image_resize::pixels::{U8x3, U8, Pixel};
    /// assert_eq!(U8x3::size(), 3);
    /// assert_eq!(U8::size(), 1);
    /// ```
    fn size() -> usize {
        size_of::<Self>()
    }
}

macro_rules! pixel_struct {
    ($name:ident, $type:tt, $pixel_type:expr, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq)]
        #[repr(C)]
        pub struct $name(pub $type);

        impl Pixel for $name {
            fn pixel_type() -> PixelType {
                $pixel_type
            }
        }
    };
}

pixel_struct!(U8, u8, PixelType::U8, "One byte per pixel");
pixel_struct!(
    U8x3,
    [u8; 3],
    PixelType::U8x3,
    "Three bytes per pixel (e.g. RGB)"
);
pixel_struct!(
    U8x4,
    u32,
    PixelType::U8x4,
    "Four bytes per pixel (RGBA, RGBx, CMYK and other)"
);
pixel_struct!(I32, i32, PixelType::I32, "One `i32` component per pixel");
pixel_struct!(F32, f32, PixelType::F32, "One `f32` component per pixel");
