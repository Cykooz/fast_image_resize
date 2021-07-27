use std::mem::transmute;
use std::num::NonZeroU32;
use std::slice;

use crate::errors::{CropBoxError, ImageBufferError, ImageRowsError, InvalidBufferSizeError};

pub type TwoRows<'a> = (&'a [u32], &'a [u32]);
pub type FourRows<'a> = (&'a [u32], &'a [u32], &'a [u32], &'a [u32]);
pub type RowMut<'a, 'b> = &'b mut &'a mut [u32];
pub type FourRowsMut<'a, 'b> = (
    &'b mut &'a mut [u32],
    &'b mut &'a mut [u32],
    &'b mut &'a mut [u32],
    &'b mut &'a mut [u32],
);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelType {
    U8x4,
    I32,
    F32,
}

#[derive(Debug, Clone, Copy)]
pub struct CropBox {
    pub left: u32,
    pub top: u32,
    pub width: NonZeroU32,
    pub height: NonZeroU32,
}

/// An immutable view of image data used by resizer as source image.
#[derive(Debug, Clone)]
pub struct SrcImageView<'a> {
    width: NonZeroU32,
    height: NonZeroU32,
    crop_box: CropBox,
    rows: Vec<&'a [u32]>,
    pixel_type: PixelType,
}

/// An mutable view of image data used by resizer as destination image.
#[derive(Debug)]
pub struct DstImageView<'a> {
    width: NonZeroU32,
    height: NonZeroU32,
    rows: Vec<&'a mut [u32]>,
    pixel_type: PixelType,
}

impl<'a> SrcImageView<'a> {
    pub fn from_rows(
        width: NonZeroU32,
        height: NonZeroU32,
        rows: Vec<&'a [u32]>,
        pixel_type: PixelType,
    ) -> Result<Self, ImageRowsError> {
        if rows.len() != height.get() as usize {
            return Err(ImageRowsError::InvalidRowsCount);
        }
        let row_size = width.get() as usize;
        if rows.iter().any(|row| row.len() != row_size) {
            return Err(ImageRowsError::InvalidRowSize);
        }
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
            pixel_type,
        })
    }

    pub fn from_buffer(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: &'a [u8],
        pixel_type: PixelType,
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize * 4;
        if buffer.len() != size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        let (head, pixels, _) = unsafe { buffer.align_to::<u32>() };
        if !head.is_empty() {
            return Err(ImageBufferError::InvalidBufferAlignment);
        }

        let rows = pixels.chunks(width.get() as usize).collect();
        Ok(Self::from_rows(width, height, rows, pixel_type).unwrap())
    }

    pub fn from_pixels(
        width: NonZeroU32,
        height: NonZeroU32,
        pixels: &'a [u32],
        pixel_type: PixelType,
    ) -> Result<Self, InvalidBufferSizeError> {
        let size = (width.get() * height.get()) as usize;
        if pixels.len() != size {
            return Err(InvalidBufferSizeError);
        }
        let rows = pixels.chunks(width.get() as usize).collect();
        Ok(Self::from_rows(width, height, rows, pixel_type).unwrap())
    }

    #[inline(always)]
    pub fn pixel_type(&self) -> PixelType {
        self.pixel_type
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

    #[inline(always)]
    pub fn get_buffer(&self) -> Vec<u8> {
        let row_size = self.width.get() as usize;
        self.rows
            .iter()
            .map(|row| unsafe { row[0..row_size].align_to::<u8>().1 })
            .flatten()
            .copied()
            .collect()
    }

    #[inline]
    pub(crate) fn get_pixel_u32(&self, x: u32, y: u32) -> u32 {
        self.rows[y as usize][x as usize]
    }

    #[inline(always)]
    pub(crate) fn get_pixel_i32(&self, x: u32, y: u32) -> i32 {
        unsafe { transmute(self.get_pixel_u32(x, y)) }
    }

    #[inline(always)]
    pub(crate) fn get_pixel_f32(&self, x: u32, y: u32) -> f32 {
        f32::from_bits(self.get_pixel_u32(x, y))
    }

    #[inline(always)]
    pub(crate) fn iter_4_rows(
        &'a self,
        start_y: u32,
        max_y: u32,
    ) -> impl Iterator<Item = FourRows<'a>> {
        let start_y = start_y as usize;
        let max_y = max_y.min(self.height.get()) as usize;
        let rows = self.rows.get(start_y..max_y).unwrap_or_else(|| &[]);
        rows.chunks_exact(4).map(|rows| match *rows {
            [r0, r1, r2, r3] => (r0, r1, r2, r3),
            _ => unreachable!(),
        })
    }

    #[inline(always)]
    pub(crate) fn iter_2_rows(
        &'a self,
        start_y: u32,
        max_y: u32,
    ) -> impl Iterator<Item = TwoRows<'a>> {
        let start_y = start_y as usize;
        let max_y = max_y.min(self.height.get()) as usize;
        let rows = self.rows.get(start_y..max_y).unwrap_or_else(|| &[]);
        rows.chunks_exact(2).map(|rows| match *rows {
            [r0, r1] => (r0, r1),
            _ => unreachable!(),
        })
    }

    #[inline(always)]
    pub(crate) fn iter_rows(&'a self, start_y: u32, max_y: u32) -> impl Iterator<Item = &'a [u32]> {
        let start_y = start_y as usize;
        let max_y = max_y.min(self.height.get()) as usize;
        let rows = self.rows.get(start_y..max_y).unwrap_or_else(|| &[]);
        rows.iter().copied()
    }

    #[inline(always)]
    pub(crate) fn iter_horiz(&self, x: u32, y: u32) -> &[u32] {
        if let Some(&row) = self.rows.get(y as usize) {
            let start_pos = x as usize;
            if let Some(res) = row.get(start_pos..) {
                return res;
            }
        }
        &[]
    }

    #[inline]
    pub(crate) fn iter_horiz_i32(&self, x: u32, y: u32) -> &[i32] {
        let row = self.iter_horiz(x, y);
        let ptr = row.as_ptr();
        unsafe { slice::from_raw_parts(ptr as *const i32, row.len()) }
    }

    #[inline]
    pub(crate) fn iter_horiz_f32(&self, x: u32, y: u32) -> &[f32] {
        let row = self.iter_horiz(x, y);
        let ptr = row.as_ptr();
        unsafe { slice::from_raw_parts(ptr as *const f32, row.len()) }
    }

    #[inline(always)]
    pub(crate) fn get_row(&self, y: u32) -> Option<&[u32]> {
        self.rows.get(y as usize).copied()
    }

    #[inline(always)]
    pub(crate) fn iter_rows_with_step(
        &self,
        mut y: f64,
        step: f64,
        max_count: usize,
    ) -> impl Iterator<Item = &[u32]> {
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

impl<'a> DstImageView<'a> {
    #[inline(always)]
    pub fn from_rows(
        width: NonZeroU32,
        height: NonZeroU32,
        rows: Vec<&'a mut [u32]>,
        pixel_type: PixelType,
    ) -> Result<Self, ImageRowsError> {
        if rows.len() != height.get() as usize {
            return Err(ImageRowsError::InvalidRowsCount);
        }
        let row_size = width.get() as usize;
        if rows.iter().any(|row| row.len() != row_size) {
            return Err(ImageRowsError::InvalidRowSize);
        }
        Ok(Self {
            width,
            height,
            rows,
            pixel_type,
        })
    }

    pub fn from_buffer(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: &'a mut [u8],
        pixel_type: PixelType,
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize * 4;
        if buffer.len() != size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        let (head, pixels, _) = unsafe { buffer.align_to_mut::<u32>() };
        if !head.is_empty() {
            return Err(ImageBufferError::InvalidBufferAlignment);
        }

        let rows = pixels.chunks_mut(width.get() as usize).collect();
        Ok(Self::from_rows(width, height, rows, pixel_type).unwrap())
    }

    #[inline(always)]
    pub fn pixel_type(&self) -> PixelType {
        self.pixel_type
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
    pub(crate) fn iter_rows_mut(&mut self) -> slice::IterMut<&'a mut [u32]> {
        self.rows.iter_mut()
    }

    #[inline(always)]
    pub(crate) fn iter_4_rows_mut(&mut self) -> impl Iterator<Item = FourRowsMut<'a, '_>> {
        self.rows.chunks_exact_mut(4).map(|rows| match rows {
            [a, b, c, d] => (a, b, c, d),
            _ => unreachable!(),
        })
    }

    #[inline(always)]
    pub(crate) fn get_row_mut(&mut self, y: u32) -> Option<RowMut<'a, '_>> {
        self.rows.get_mut(y as usize)
    }
}
