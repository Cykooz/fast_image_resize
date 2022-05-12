//! Contains types of pixels.
use std::fmt::Debug;
use std::mem::size_of;
use std::slice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PixelType {
    U8x2,
    U8x3,
    U8x4,
    U16x3,
    I32,
    F32,
    U8,
}

impl PixelType {
    pub(crate) fn size(&self) -> usize {
        match self {
            Self::U8 => 1,
            Self::U8x2 => 2,
            Self::U8x3 => 3,
            Self::U16x3 => 6,
            _ => 4,
        }
    }

    /// Returns `true` if given buffer is aligned by the alignment of pixel.
    pub(crate) fn is_aligned(&self, buffer: &[u8]) -> bool {
        match self {
            Self::U8 => true,
            Self::U8x2 => unsafe { buffer.align_to::<U8x2>().0.is_empty() },
            Self::U8x3 => unsafe { buffer.align_to::<U8x3>().0.is_empty() },
            Self::U8x4 => unsafe { buffer.align_to::<U8x4>().0.is_empty() },
            Self::U16x3 => unsafe { buffer.align_to::<U16x3>().0.is_empty() },
            Self::I32 => unsafe { buffer.align_to::<I32>().0.is_empty() },
            Self::F32 => unsafe { buffer.align_to::<F32>().0.is_empty() },
        }
    }
}

/// Additional information about pixel type.
pub trait Pixel
where
    Self: Copy + Sized + Debug,
{
    /// Type of pixel components
    type Component;

    fn pixel_type() -> PixelType;

    /// Count of pixel's components
    fn components_count() -> usize;

    /// Size of pixel in bytes
    ///
    /// Example:
    /// ```
    /// # use fast_image_resize::pixels::{U8x2, U8x3, U8, Pixel};
    /// assert_eq!(U8x3::size(), 3);
    /// assert_eq!(U8x2::size(), 2);
    /// assert_eq!(U8::size(), 1);
    /// ```
    fn size() -> usize {
        size_of::<Self>()
    }

    /// Create slice of components of pixels from slice of pixels
    fn components(buf: &[Self]) -> &[Self::Component] {
        let size = buf.len() * Self::components_count();
        let components_ptr = buf.as_ptr() as *const Self::Component;
        unsafe { slice::from_raw_parts(components_ptr, size) }
    }

    /// Create mutable slice of components of pixels from mutable slice of pixels
    fn components_mut(buf: &mut [Self]) -> &mut [Self::Component] {
        let size = buf.len() * Self::components_count();
        let components_ptr = buf.as_mut_ptr() as *mut Self::Component;
        unsafe { slice::from_raw_parts_mut(components_ptr, size) }
    }
}

macro_rules! pixel_struct {
    ($name:ident, $type:tt, $comp_type:tt, $comp_count:expr, $pixel_type:expr, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq)]
        #[repr(C)]
        pub struct $name(pub $type);

        impl Pixel for $name {
            type Component = $comp_type;

            fn pixel_type() -> PixelType {
                $pixel_type
            }

            fn components_count() -> usize {
                $comp_count
            }
        }
    };
}

pixel_struct!(U8, u8, u8, 1, PixelType::U8, "One byte per pixel");
pixel_struct!(
    U8x2,
    u16,
    u8,
    2,
    PixelType::U8x2,
    "Two bytes per pixel (e.g. LA)"
);
pixel_struct!(
    U8x3,
    [u8; 3],
    u8,
    3,
    PixelType::U8x3,
    "Three bytes per pixel (e.g. RGB)"
);
pixel_struct!(
    U8x4,
    u32,
    u8,
    4,
    PixelType::U8x4,
    "Four bytes per pixel (RGBA, RGBx, CMYK and other)"
);
pixel_struct!(
    U16x3,
    [u16; 3],
    u16,
    3,
    PixelType::U16x3,
    "Three `u16` components per pixel (e.g. RGB)"
);
pixel_struct!(
    I32,
    i32,
    i32,
    1,
    PixelType::I32,
    "One `i32` component per pixel"
);
pixel_struct!(
    F32,
    f32,
    f32,
    1,
    PixelType::F32,
    "One `f32` component per pixel"
);
