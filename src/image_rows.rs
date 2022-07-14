use std::mem::transmute;
use std::num::NonZeroU32;

use crate::pixels::{Pixel, U16x2, U16x3, U16x4, U8x2, U8x3, U8x4, F32, I32, U16, U8};
use crate::{ImageRowsError, PixelType};

/// An immutable rows of image.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ImageRows<'a> {
    U8(Vec<&'a [U8]>),
    U8x2(Vec<&'a [U8x2]>),
    U8x3(Vec<&'a [U8x3]>),
    U8x4(Vec<&'a [U8x4]>),
    U16(Vec<&'a [U16]>),
    U16x2(Vec<&'a [U16x2]>),
    U16x3(Vec<&'a [U16x3]>),
    U16x4(Vec<&'a [U16x4]>),
    I32(Vec<&'a [I32]>),
    F32(Vec<&'a [F32]>),
}

impl<'a> ImageRows<'a> {
    pub(crate) fn check_size(
        &self,
        width: NonZeroU32,
        height: NonZeroU32,
    ) -> Result<(), ImageRowsError> {
        match self {
            ImageRows::U8(rows) => check_rows_count_and_size(width, height, rows),
            ImageRows::U8x2(rows) => check_rows_count_and_size(width, height, rows),
            ImageRows::U8x3(rows) => check_rows_count_and_size(width, height, rows),
            ImageRows::U8x4(rows) => check_rows_count_and_size(width, height, rows),
            ImageRows::U16(rows) => check_rows_count_and_size(width, height, rows),
            ImageRows::U16x2(rows) => check_rows_count_and_size(width, height, rows),
            ImageRows::U16x3(rows) => check_rows_count_and_size(width, height, rows),
            ImageRows::U16x4(rows) => check_rows_count_and_size(width, height, rows),
            ImageRows::I32(rows) => check_rows_count_and_size(width, height, rows),
            ImageRows::F32(rows) => check_rows_count_and_size(width, height, rows),
        }
    }

    pub fn pixel_type(&self) -> PixelType {
        match self {
            Self::U8(_) => PixelType::U8,
            Self::U8x2(_) => PixelType::U8x2,
            Self::U8x3(_) => PixelType::U8x3,
            Self::U8x4(_) => PixelType::U8x4,
            Self::U16(_) => PixelType::U16,
            Self::U16x2(_) => PixelType::U16x2,
            Self::U16x3(_) => PixelType::U16x3,
            Self::U16x4(_) => PixelType::U16x4,
            Self::I32(_) => PixelType::I32,
            Self::F32(_) => PixelType::F32,
        }
    }

    pub fn typed_rows<P: Pixel>(&self) -> Option<&[&'a [P]]> {
        match (P::pixel_type(), self) {
            (PixelType::U8, Self::U8(rows)) => Some(unsafe { transmute(rows.as_slice()) }),
            (PixelType::U8x2, Self::U8x2(rows)) => Some(unsafe { transmute(rows.as_slice()) }),
            (PixelType::U8x3, Self::U8x3(rows)) => Some(unsafe { transmute(rows.as_slice()) }),
            (PixelType::U8x4, Self::U8x4(rows)) => Some(unsafe { transmute(rows.as_slice()) }),
            (PixelType::U16, Self::U16(rows)) => Some(unsafe { transmute(rows.as_slice()) }),
            (PixelType::U16x2, Self::U16x2(rows)) => Some(unsafe { transmute(rows.as_slice()) }),
            (PixelType::U16x3, Self::U16x3(rows)) => Some(unsafe { transmute(rows.as_slice()) }),
            (PixelType::U16x4, Self::U16x4(rows)) => Some(unsafe { transmute(rows.as_slice()) }),
            (PixelType::I32, Self::I32(rows)) => Some(unsafe { transmute(rows.as_slice()) }),
            (PixelType::F32, Self::F32(rows)) => Some(unsafe { transmute(rows.as_slice()) }),
            _ => None,
        }
    }
}

/// A mutable rows of image.
#[derive(Debug)]
#[non_exhaustive]
pub enum ImageRowsMut<'a> {
    U8(Vec<&'a mut [U8]>),
    U8x2(Vec<&'a mut [U8x2]>),
    U8x3(Vec<&'a mut [U8x3]>),
    U8x4(Vec<&'a mut [U8x4]>),
    U16(Vec<&'a mut [U16]>),
    U16x2(Vec<&'a mut [U16x2]>),
    U16x3(Vec<&'a mut [U16x3]>),
    U16x4(Vec<&'a mut [U16x4]>),
    I32(Vec<&'a mut [I32]>),
    F32(Vec<&'a mut [F32]>),
}

impl<'a> ImageRowsMut<'a> {
    pub(crate) fn check_size(
        &self,
        width: NonZeroU32,
        height: NonZeroU32,
    ) -> Result<(), ImageRowsError> {
        match self {
            Self::U8x2(rows) => check_rows_count_and_size(width, height, rows),
            Self::U8x3(rows) => check_rows_count_and_size(width, height, rows),
            Self::U8x4(rows) => check_rows_count_and_size(width, height, rows),
            Self::U16(rows) => check_rows_count_and_size(width, height, rows),
            Self::U16x2(rows) => check_rows_count_and_size(width, height, rows),
            Self::U16x3(rows) => check_rows_count_and_size(width, height, rows),
            Self::U16x4(rows) => check_rows_count_and_size(width, height, rows),
            Self::I32(rows) => check_rows_count_and_size(width, height, rows),
            Self::F32(rows) => check_rows_count_and_size(width, height, rows),
            Self::U8(rows) => check_rows_count_and_size(width, height, rows),
        }
    }

    pub fn pixel_type(&self) -> PixelType {
        match self {
            Self::U8x2(_) => PixelType::U8x2,
            Self::U8x3(_) => PixelType::U8x3,
            Self::U8x4(_) => PixelType::U8x4,
            Self::U16(_) => PixelType::U16,
            Self::U16x2(_) => PixelType::U16x2,
            Self::U16x3(_) => PixelType::U16x3,
            Self::U16x4(_) => PixelType::U16x4,
            Self::I32(_) => PixelType::I32,
            Self::F32(_) => PixelType::F32,
            Self::U8(_) => PixelType::U8,
        }
    }

    pub fn typed_rows<P: Pixel>(&mut self) -> Option<&mut [&'a mut [P]]> {
        match (P::pixel_type(), self) {
            (PixelType::U8, Self::U8(rows)) => Some(unsafe { transmute(rows.as_mut_slice()) }),
            (PixelType::U8x2, Self::U8x2(rows)) => Some(unsafe { transmute(rows.as_mut_slice()) }),
            (PixelType::U8x3, Self::U8x3(rows)) => Some(unsafe { transmute(rows.as_mut_slice()) }),
            (PixelType::U8x4, Self::U8x4(rows)) => Some(unsafe { transmute(rows.as_mut_slice()) }),
            (PixelType::U16, Self::U16(rows)) => Some(unsafe { transmute(rows.as_mut_slice()) }),
            (PixelType::U16x2, Self::U16x2(rows)) => {
                Some(unsafe { transmute(rows.as_mut_slice()) })
            }
            (PixelType::U16x3, Self::U16x3(rows)) => {
                Some(unsafe { transmute(rows.as_mut_slice()) })
            }
            (PixelType::U16x4, Self::U16x4(rows)) => {
                Some(unsafe { transmute(rows.as_mut_slice()) })
            }
            (PixelType::I32, Self::I32(rows)) => Some(unsafe { transmute(rows.as_mut_slice()) }),
            (PixelType::F32, Self::F32(rows)) => Some(unsafe { transmute(rows.as_mut_slice()) }),
            _ => None,
        }
    }
}

fn check_rows_count_and_size<T>(
    width: NonZeroU32,
    height: NonZeroU32,
    rows: &[impl AsRef<[T]>],
) -> Result<(), ImageRowsError> {
    if rows.len() != height.get() as usize {
        return Err(ImageRowsError::InvalidRowsCount);
    }
    let row_size = width.get() as usize;
    if rows.iter().any(|row| row.as_ref().len() != row_size) {
        return Err(ImageRowsError::InvalidRowSize);
    }
    Ok(())
}

macro_rules! image_rows_from {
    ($pixel_type:tt, $enum_type:expr) => {
        impl<'a> From<Vec<&'a [$pixel_type]>> for ImageRows<'a> {
            fn from(rows: Vec<&'a [$pixel_type]>) -> Self {
                $enum_type(rows)
            }
        }
    };
}

image_rows_from!(U8, ImageRows::U8);
image_rows_from!(U8x2, ImageRows::U8x2);
image_rows_from!(U8x3, ImageRows::U8x3);
image_rows_from!(U8x4, ImageRows::U8x4);
image_rows_from!(U16, ImageRows::U16);
image_rows_from!(U16x2, ImageRows::U16x2);
image_rows_from!(U16x3, ImageRows::U16x3);
image_rows_from!(U16x4, ImageRows::U16x4);
image_rows_from!(I32, ImageRows::I32);
image_rows_from!(F32, ImageRows::F32);
