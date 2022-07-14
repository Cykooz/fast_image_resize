use std::num::NonZeroU32;

use crate::errors::{CropBoxError, ImageBufferError, ImageRowsError};
use crate::image_rows::{ImageRows, ImageRowsMut};
use crate::pixels::{Pixel, PixelType};

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
            PixelType::U16x2 => {
                let pixels = align_buffer_to(buffer)?;
                ImageRows::U16x2(
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
            PixelType::U16x4 => {
                let pixels = align_buffer_to(buffer)?;
                ImageRows::U16x4(
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
    pub(crate) fn typed_rows<P: Pixel>(&self) -> Option<&[&'a [P]]> {
        self.rows.typed_rows()
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
            PixelType::U16x2 => {
                let pixels = align_buffer_to_mut(buffer)?;
                ImageRowsMut::U16x2(
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
            PixelType::U16x4 => {
                let pixels = align_buffer_to_mut(buffer)?;
                ImageRowsMut::U16x4(
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

    #[inline(always)]
    pub(crate) fn typed_rows<P: Pixel>(&mut self) -> Option<&mut [&'a mut [P]]> {
        self.rows.typed_rows()
    }
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
