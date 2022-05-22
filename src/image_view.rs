use std::num::NonZeroU32;
use std::slice;

use crate::errors::{CropBoxError, ImageBufferError, ImageRowsError};
use crate::pixels::{Pixel, PixelType, U16x3, U8x2, U8x3, U8x4, F32, I32, U16, U8};

pub(crate) type RowMut<'a, 'b, T> = &'a mut &'b mut [T];
pub(crate) type TwoRows<'a, T> = (&'a [T], &'a [T]);
pub(crate) type FourRows<'a, T> = (&'a [T], &'a [T], &'a [T], &'a [T]);
pub(crate) type FourRowsMut<'a, 'b, T> = (
    &'a mut &'b mut [T],
    &'a mut &'b mut [T],
    &'a mut &'b mut [T],
    &'a mut &'b mut [T],
);

/// Parameters of crop box that may be used with [`ImageView`]
#[derive(Debug, Clone, Copy)]
pub struct CropBox {
    pub left: u32,
    pub top: u32,
    pub width: NonZeroU32,
    pub height: NonZeroU32,
}

/// An immutable rows of image.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ImageRows<'a> {
    U8(Vec<&'a [U8]>),
    U8x2(Vec<&'a [U8x2]>),
    U8x3(Vec<&'a [U8x3]>),
    U8x4(Vec<&'a [U8x4]>),
    U16(Vec<&'a [U16]>),
    U16x3(Vec<&'a [U16x3]>),
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
            ImageRows::U16x3(rows) => check_rows_count_and_size(width, height, rows),
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
            Self::U16x3(_) => PixelType::U16x3,
            Self::I32(_) => PixelType::I32,
            Self::F32(_) => PixelType::F32,
        }
    }
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
image_rows_from!(U16x3, ImageRows::U16x3);
image_rows_from!(I32, ImageRows::I32);
image_rows_from!(F32, ImageRows::F32);

/// A mutable rows of image.
#[derive(Debug)]
#[non_exhaustive]
pub enum ImageRowsMut<'a> {
    U8x2(Vec<&'a mut [U8x2]>),
    U8x3(Vec<&'a mut [U8x3]>),
    U8x4(Vec<&'a mut [U8x4]>),
    U16(Vec<&'a mut [U16]>),
    U16x3(Vec<&'a mut [U16x3]>),
    I32(Vec<&'a mut [I32]>),
    F32(Vec<&'a mut [F32]>),
    U8(Vec<&'a mut [U8]>),
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
            Self::U16x3(rows) => check_rows_count_and_size(width, height, rows),
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
            Self::U16x3(_) => PixelType::U16x3,
            Self::I32(_) => PixelType::I32,
            Self::F32(_) => PixelType::F32,
            Self::U8(_) => PixelType::U8,
        }
    }
}

/// An immutable view of image data used by resizer as source image.
#[derive(Debug, Clone)]
pub struct ImageView<'a> {
    width: NonZeroU32,
    height: NonZeroU32,
    crop_box: CropBox,
    rows: ImageRows<'a>,
}

impl<'a> ImageView<'a> {
    pub fn new(
        width: NonZeroU32,
        height: NonZeroU32,
        rows: ImageRows<'a>,
    ) -> Result<Self, ImageRowsError> {
        rows.check_size(width, height)?;
        Ok(Self {
            width,
            height,
            crop_box: CropBox {
                left: 0,
                top: 0,
                width,
                height,
            },
            rows,
        })
    }

    pub fn from_buffer(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: &'a [u8],
        pixel_type: PixelType,
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize * pixel_type.size();
        if buffer.len() < size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        let rows_count = height.get() as usize;
        let rows = match pixel_type {
            PixelType::U8x2 => {
                let pixels = align_buffer_to(buffer)?;
                ImageRows::U8x2(
                    pixels
                        .chunks_exact(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
            PixelType::U8x3 => {
                let pixels = align_buffer_to(buffer)?;
                ImageRows::U8x3(
                    pixels
                        .chunks_exact(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
            PixelType::U8x4 => {
                let pixels = align_buffer_to(buffer)?;
                ImageRows::U8x4(
                    pixels
                        .chunks_exact(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
            PixelType::U16 => {
                let pixels = align_buffer_to(buffer)?;
                ImageRows::U16(
                    pixels
                        .chunks_exact(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
            PixelType::U16x3 => {
                let pixels = align_buffer_to(buffer)?;
                ImageRows::U16x3(
                    pixels
                        .chunks_exact(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
            PixelType::I32 => {
                let pixels = align_buffer_to(buffer)?;
                ImageRows::I32(
                    pixels
                        .chunks_exact(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
            PixelType::F32 => {
                let pixels = align_buffer_to(buffer)?;
                ImageRows::F32(
                    pixels
                        .chunks_exact(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
            PixelType::U8 => {
                let pixels = align_buffer_to(buffer)?;
                ImageRows::U8(
                    pixels
                        .chunks_exact(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
        };
        Ok(Self {
            width,
            height,
            crop_box: CropBox {
                left: 0,
                top: 0,
                width,
                height,
            },
            rows,
        })
    }

    #[inline(always)]
    pub fn pixel_type(&self) -> PixelType {
        self.rows.pixel_type()
    }

    #[inline(always)]
    pub fn width(&self) -> NonZeroU32 {
        self.width
    }

    #[inline(always)]
    pub fn height(&self) -> NonZeroU32 {
        self.height
    }

    #[inline(always)]
    pub fn crop_box(&self) -> CropBox {
        self.crop_box
    }

    pub fn set_crop_box(&mut self, crop_box: CropBox) -> Result<(), CropBoxError> {
        if crop_box.left >= self.width.get() || crop_box.top >= self.height.get() {
            return Err(CropBoxError::PositionIsOutOfImageBoundaries);
        }
        let right = crop_box.left + crop_box.width.get();
        let bottom = crop_box.top + crop_box.height.get();
        if right > self.width.get() || bottom > self.height.get() {
            return Err(CropBoxError::SizeIsOutOfImageBoundaries);
        }
        self.crop_box = crop_box;
        Ok(())
    }

    /// Set a crop box to resize the source image into the
    /// aspect ratio of destination image without distortions.
    ///
    /// `centering` used to control the cropping position. Use (0.5, 0.5) for
    /// center cropping (e.g. if cropping the width, take 50% off
    /// of the left side, and therefore 50% off the right side).
    /// (0.0, 0.0) will crop from the top left corner (i.e. if
    /// cropping the width, take all the crop off of the right
    /// side, and if cropping the height, take all of it off the
    /// bottom). (1.0, 0.0) will crop from the bottom left
    /// corner, etc. (i.e. if cropping the width, take all the
    /// crop off the left side, and if cropping the height take
    /// none from the top, and therefore all off the bottom).
    pub fn set_crop_box_to_fit_dst_size(
        &mut self,
        dst_width: NonZeroU32,
        dst_height: NonZeroU32,
        centering: Option<(f32, f32)>,
    ) {
        // This function based on code of ImageOps.fit() from Pillow package.
        // https://github.com/python-pillow/Pillow/blob/master/src/PIL/ImageOps.py
        let centering = if let Some((x, y)) = centering {
            (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0))
        } else {
            (0.5, 0.5)
        };

        // calculate aspect ratios
        let width = self.width.get() as f32;
        let height = self.height.get() as f32;
        let image_ratio = width / height;
        let required_ration = dst_width.get() as f32 / dst_height.get() as f32;

        let crop_width;
        let crop_height;
        // figure out if the sides or top/bottom will be cropped off
        if (image_ratio - required_ration).abs() < f32::EPSILON {
            // The image is already the needed ratio
            crop_width = width;
            crop_height = height;
        } else if image_ratio >= required_ration {
            // The image is wider than what's needed, crop the sides
            crop_width = required_ration * height;
            crop_height = height;
        } else {
            // The image is taller than what's needed, crop the top and bottom
            crop_width = width;
            crop_height = width / required_ration;
        }

        let crop_left = (width - crop_width) * centering.0;
        let crop_top = (height - crop_height) * centering.1;

        self.set_crop_box(CropBox {
            left: crop_left.round() as u32,
            top: crop_top.round() as u32,
            width: NonZeroU32::new(crop_width.round() as u32).unwrap(),
            height: NonZeroU32::new(crop_height.round() as u32).unwrap(),
        })
        .unwrap();
    }

    pub(crate) fn u8x2_image(&self) -> Option<TypedImageView<U8x2>> {
        if let ImageRows::U8x2(ref rows) = self.rows {
            Some(TypedImageView {
                width: self.width,
                height: self.height,
                crop_box: self.crop_box,
                rows,
            })
        } else {
            None
        }
    }

    pub(crate) fn u8x3_image(&self) -> Option<TypedImageView<U8x3>> {
        if let ImageRows::U8x3(ref rows) = self.rows {
            Some(TypedImageView {
                width: self.width,
                height: self.height,
                crop_box: self.crop_box,
                rows,
            })
        } else {
            None
        }
    }

    pub(crate) fn u8x4_image(&self) -> Option<TypedImageView<U8x4>> {
        if let ImageRows::U8x4(ref rows) = self.rows {
            Some(TypedImageView {
                width: self.width,
                height: self.height,
                crop_box: self.crop_box,
                rows,
            })
        } else {
            None
        }
    }

    pub(crate) fn u16_image(&self) -> Option<TypedImageView<U16>> {
        if let ImageRows::U16(ref rows) = self.rows {
            Some(TypedImageView {
                width: self.width,
                height: self.height,
                crop_box: self.crop_box,
                rows,
            })
        } else {
            None
        }
    }

    pub(crate) fn u16x3_image(&self) -> Option<TypedImageView<U16x3>> {
        if let ImageRows::U16x3(ref rows) = self.rows {
            Some(TypedImageView {
                width: self.width,
                height: self.height,
                crop_box: self.crop_box,
                rows,
            })
        } else {
            None
        }
    }

    pub(crate) fn i32_image(&self) -> Option<TypedImageView<I32>> {
        if let ImageRows::I32(ref rows) = self.rows {
            Some(TypedImageView {
                width: self.width,
                height: self.height,
                crop_box: self.crop_box,
                rows,
            })
        } else {
            None
        }
    }

    pub(crate) fn f32_image(&self) -> Option<TypedImageView<F32>> {
        if let ImageRows::F32(ref rows) = self.rows {
            Some(TypedImageView {
                width: self.width,
                height: self.height,
                crop_box: self.crop_box,
                rows,
            })
        } else {
            None
        }
    }

    pub(crate) fn u8_image(&self) -> Option<TypedImageView<U8>> {
        if let ImageRows::U8(ref rows) = self.rows {
            Some(TypedImageView {
                width: self.width,
                height: self.height,
                crop_box: self.crop_box,
                rows,
            })
        } else {
            None
        }
    }
}

/// Generic immutable image view.
pub(crate) struct TypedImageView<'a, 'b, P>
where
    P: Pixel,
{
    width: NonZeroU32,
    height: NonZeroU32,
    crop_box: CropBox,
    rows: &'a [&'b [P]],
}

impl<'a, 'b, P> TypedImageView<'a, 'b, P>
where
    P: Pixel,
{
    pub fn new(width: NonZeroU32, height: NonZeroU32, rows: &'a [&'b [P]]) -> Self {
        Self {
            width,
            height,
            crop_box: CropBox {
                left: 0,
                top: 0,
                width,
                height,
            },
            rows,
        }
    }

    #[inline(always)]
    pub fn width(&self) -> NonZeroU32 {
        self.width
    }

    #[inline(always)]
    pub fn height(&self) -> NonZeroU32 {
        self.height
    }

    #[inline(always)]
    pub fn crop_box(&self) -> CropBox {
        self.crop_box
    }

    #[inline(always)]
    pub(crate) fn iter_4_rows<'s>(
        &'s self,
        start_y: u32,
        max_y: u32,
    ) -> impl Iterator<Item = FourRows<'b, P>> + 's {
        let start_y = start_y as usize;
        let max_y = max_y.min(self.height.get()) as usize;
        let rows = self.rows.get(start_y..max_y).unwrap_or(&[]);
        rows.chunks_exact(4).map(|rows| match *rows {
            [r0, r1, r2, r3] => (r0, r1, r2, r3),
            _ => unreachable!(),
        })
    }

    #[inline(always)]
    pub(crate) fn iter_2_rows<'s>(
        &'s self,
        start_y: u32,
        max_y: u32,
    ) -> impl Iterator<Item = TwoRows<'b, P>> + 's {
        let start_y = start_y as usize;
        let max_y = max_y.min(self.height.get()) as usize;
        let rows = self.rows.get(start_y..max_y).unwrap_or(&[]);
        rows.chunks_exact(2).map(|rows| match *rows {
            [r0, r1] => (r0, r1),
            _ => unreachable!(),
        })
    }

    #[inline(always)]
    pub(crate) fn iter_rows<'s>(&'s self, start_y: u32) -> impl Iterator<Item = &'b [P]> + 's {
        let start_y = start_y as usize;
        let rows = self.rows.get(start_y..).unwrap_or(&[]);
        rows.iter().copied()
    }

    #[inline(always)]
    pub(crate) fn get_row(&self, y: u32) -> Option<&'b [P]> {
        self.rows.get(y as usize).copied()
    }

    #[inline(always)]
    pub(crate) fn iter_rows_with_step<'s>(
        &'s self,
        mut y: f64,
        step: f64,
        max_count: usize,
    ) -> impl Iterator<Item = &'b [P]> + 's {
        let steps = (self.height.get() as f64 - y) / step;
        let steps = (steps.max(0.).ceil() as usize).min(max_count);
        (0..steps).map(move |_| {
            // Safety of value of y guaranteed by calculation of steps count
            let row = unsafe { *self.rows.get_unchecked(y as usize) };
            y += step;
            row
        })
    }
}

/// A mutable view of image data used by resizer as destination image.
#[derive(Debug)]
pub struct ImageViewMut<'a> {
    width: NonZeroU32,
    height: NonZeroU32,
    rows: ImageRowsMut<'a>,
}

impl<'a> ImageViewMut<'a> {
    pub fn new(
        width: NonZeroU32,
        height: NonZeroU32,
        rows: ImageRowsMut<'a>,
    ) -> Result<Self, ImageRowsError> {
        rows.check_size(width, height)?;
        Ok(Self {
            width,
            height,
            rows,
        })
    }

    pub fn from_buffer(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: &'a mut [u8],
        pixel_type: PixelType,
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize * pixel_type.size();
        if buffer.len() < size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        let rows_count = height.get() as usize;
        let rows = match pixel_type {
            PixelType::U8x2 => {
                let pixels = align_buffer_to_mut(buffer)?;
                ImageRowsMut::U8x2(
                    pixels
                        .chunks_exact_mut(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
            PixelType::U8x3 => {
                let pixels = align_buffer_to_mut(buffer)?;
                ImageRowsMut::U8x3(
                    pixels
                        .chunks_exact_mut(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
            PixelType::U8x4 => {
                let pixels = align_buffer_to_mut(buffer)?;
                ImageRowsMut::U8x4(
                    pixels
                        .chunks_exact_mut(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
            PixelType::U16 => {
                let pixels = align_buffer_to_mut(buffer)?;
                ImageRowsMut::U16(
                    pixels
                        .chunks_exact_mut(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
            PixelType::U16x3 => {
                let pixels = align_buffer_to_mut(buffer)?;
                ImageRowsMut::U16x3(
                    pixels
                        .chunks_exact_mut(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
            PixelType::I32 => {
                let pixels = align_buffer_to_mut(buffer)?;
                ImageRowsMut::I32(
                    pixels
                        .chunks_exact_mut(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
            PixelType::F32 => {
                let pixels = align_buffer_to_mut(buffer)?;
                ImageRowsMut::F32(
                    pixels
                        .chunks_exact_mut(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
            PixelType::U8 => {
                let pixels = align_buffer_to_mut(buffer)?;
                ImageRowsMut::U8(
                    pixels
                        .chunks_exact_mut(width.get() as usize)
                        .take(rows_count)
                        .collect(),
                )
            }
        };
        Ok(Self {
            width,
            height,
            rows,
        })
    }

    #[inline(always)]
    pub fn pixel_type(&self) -> PixelType {
        self.rows.pixel_type()
    }

    #[inline(always)]
    pub fn width(&self) -> NonZeroU32 {
        self.width
    }

    #[inline(always)]
    pub fn height(&self) -> NonZeroU32 {
        self.height
    }

    pub(crate) fn u8x2_image<'s>(&'s mut self) -> Option<TypedImageViewMut<'s, 'a, U8x2>> {
        if let ImageRowsMut::U8x2(rows) = &mut self.rows {
            Some(TypedImageViewMut {
                width: self.width,
                height: self.height,
                rows,
            })
        } else {
            None
        }
    }

    pub(crate) fn u8x3_image<'s>(&'s mut self) -> Option<TypedImageViewMut<'s, 'a, U8x3>> {
        if let ImageRowsMut::U8x3(rows) = &mut self.rows {
            Some(TypedImageViewMut {
                width: self.width,
                height: self.height,
                rows,
            })
        } else {
            None
        }
    }

    pub(crate) fn u8x4_image<'s>(&'s mut self) -> Option<TypedImageViewMut<'s, 'a, U8x4>> {
        if let ImageRowsMut::U8x4(rows) = &mut self.rows {
            Some(TypedImageViewMut {
                width: self.width,
                height: self.height,
                rows,
            })
        } else {
            None
        }
    }

    pub(crate) fn u16_image<'s>(&'s mut self) -> Option<TypedImageViewMut<'s, 'a, U16>> {
        if let ImageRowsMut::U16(rows) = &mut self.rows {
            Some(TypedImageViewMut {
                width: self.width,
                height: self.height,
                rows,
            })
        } else {
            None
        }
    }

    pub(crate) fn u16x3_image<'s>(&'s mut self) -> Option<TypedImageViewMut<'s, 'a, U16x3>> {
        if let ImageRowsMut::U16x3(rows) = &mut self.rows {
            Some(TypedImageViewMut {
                width: self.width,
                height: self.height,
                rows,
            })
        } else {
            None
        }
    }

    pub(crate) fn i32_image<'s>(&'s mut self) -> Option<TypedImageViewMut<'s, 'a, I32>> {
        if let ImageRowsMut::I32(rows) = &mut self.rows {
            Some(TypedImageViewMut {
                width: self.width,
                height: self.height,
                rows,
            })
        } else {
            None
        }
    }

    pub(crate) fn f32_image<'s>(&'s mut self) -> Option<TypedImageViewMut<'s, 'a, F32>> {
        if let ImageRowsMut::F32(rows) = &mut self.rows {
            Some(TypedImageViewMut {
                width: self.width,
                height: self.height,
                rows,
            })
        } else {
            None
        }
    }

    pub(crate) fn u8_image<'s>(&'s mut self) -> Option<TypedImageViewMut<'s, 'a, U8>> {
        if let ImageRowsMut::U8(rows) = &mut self.rows {
            Some(TypedImageViewMut {
                width: self.width,
                height: self.height,
                rows,
            })
        } else {
            None
        }
    }
}

/// Generic mutable image view.
pub(crate) struct TypedImageViewMut<'a, 'b, P>
where
    P: Pixel,
{
    width: NonZeroU32,
    height: NonZeroU32,
    rows: &'a mut [&'b mut [P]],
}

impl<'a, 'b, P> TypedImageViewMut<'a, 'b, P>
where
    P: Pixel,
{
    pub fn new(width: NonZeroU32, height: NonZeroU32, rows: &'a mut [&'b mut [P]]) -> Self {
        Self {
            width,
            height,
            rows,
        }
    }

    #[inline(always)]
    pub fn width(&self) -> NonZeroU32 {
        self.width
    }

    #[inline(always)]
    pub fn height(&self) -> NonZeroU32 {
        self.height
    }

    #[inline(always)]
    pub fn iter_rows_mut(&mut self) -> slice::IterMut<&'b mut [P]> {
        self.rows.iter_mut()
    }

    #[inline(always)]
    pub fn iter_4_rows_mut<'s>(&'s mut self) -> impl Iterator<Item = FourRowsMut<'s, 'b, P>> {
        self.rows.chunks_exact_mut(4).map(|rows| match rows {
            [a, b, c, d] => (a, b, c, d),
            _ => unreachable!(),
        })
    }

    #[inline(always)]
    pub fn get_row_mut<'s>(&'s mut self, y: u32) -> Option<RowMut<'s, 'b, P>> {
        self.rows.get_mut(y as usize)
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

fn align_buffer_to<T>(buffer: &[u8]) -> Result<&[T], ImageBufferError> {
    let (head, pixels, _) = unsafe { buffer.align_to::<T>() };
    if !head.is_empty() {
        return Err(ImageBufferError::InvalidBufferAlignment);
    }
    Ok(pixels)
}

fn align_buffer_to_mut<T>(buffer: &mut [u8]) -> Result<&mut [T], ImageBufferError> {
    let (head, pixels, _) = unsafe { buffer.align_to_mut::<T>() };
    if !head.is_empty() {
        return Err(ImageBufferError::InvalidBufferAlignment);
    }
    Ok(pixels)
}
