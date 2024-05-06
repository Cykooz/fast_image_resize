use std::fmt::Debug;

use crate::images::BufferContainer;
use crate::pixels::InnerPixel;
use crate::{ImageBufferError, ImageView, ImageViewMut, InvalidPixelsSliceSize};

/// Generic reference to image data that provides [ImageView].
#[derive(Debug)]
pub struct TypedImageRef<'a, P> {
    width: u32,
    height: u32,
    pixels: &'a [P],
}

impl<'a, P> TypedImageRef<'a, P> {
    pub fn new(width: u32, height: u32, pixels: &'a [P]) -> Result<Self, InvalidPixelsSliceSize> {
        let pixels_count = width as usize * height as usize;
        if pixels.len() < pixels_count {
            return Err(InvalidPixelsSliceSize);
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn from_buffer(
        width: u32,
        height: u32,
        buffer: &'a [u8],
    ) -> Result<Self, ImageBufferError> {
        let pixels = align_buffer_to(buffer)?;
        Self::new(width, height, pixels).map_err(|_| ImageBufferError::InvalidBufferSize)
    }
}

unsafe impl<'a, P: InnerPixel> ImageView for TypedImageRef<'a, P> {
    type Pixel = P;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn iter_rows(&self, start_row: u32) -> impl Iterator<Item = &[Self::Pixel]> {
        let width = self.width as usize;
        if width == 0 {
            [].chunks_exact(1)
        } else {
            let start = start_row as usize * width;
            self.pixels
                .get(start..)
                .unwrap_or_default()
                .chunks_exact(width)
        }
    }

    fn iter_rows_with_step(
        &self,
        start_y: f64,
        step: f64,
        max_rows: u32,
    ) -> impl Iterator<Item = &[Self::Pixel]> {
        let row_size = self.width as usize;
        let steps = (self.height() as f64 - start_y) / step;
        let steps = (steps.max(0.).ceil() as u32).min(max_rows);
        let mut y = start_y;
        let mut next_row_y = start_y as usize;
        let mut cur_row = None;
        (0..steps).filter_map(move |_| {
            let cur_row_y = y as usize;
            if next_row_y <= cur_row_y {
                let start = cur_row_y * row_size;
                let end = start + row_size;
                cur_row = self.pixels.get(start..end);
                next_row_y = cur_row_y + 1;
            }
            y += step;
            cur_row
        })
    }
}

/// Generic image container that provides [ImageView] and [ImageViewMut].
#[derive(Debug)]
pub struct TypedImage<'a, P: Default + Copy + Debug> {
    width: u32,
    height: u32,
    pixels: BufferContainer<'a, P>,
}

impl<P: Default + Copy + Debug> TypedImage<'static, P> {
    pub fn new(width: u32, height: u32) -> Self {
        let pixels_count = width as usize * height as usize;
        Self {
            width,
            height,
            pixels: BufferContainer::Owned(vec![P::default(); pixels_count]),
        }
    }
}

impl<'a, P: InnerPixel> TypedImage<'a, P> {
    pub fn from_pixels(
        width: u32,
        height: u32,
        pixels: &'a mut [P],
    ) -> Result<Self, InvalidPixelsSliceSize> {
        let pixels_count = width as usize * height as usize;
        if pixels.len() < pixels_count {
            return Err(InvalidPixelsSliceSize);
        }
        Ok(Self {
            width,
            height,
            pixels: BufferContainer::Borrowed(pixels),
        })
    }

    pub fn from_buffer(
        width: u32,
        height: u32,
        buffer: &'a mut [u8],
    ) -> Result<Self, ImageBufferError> {
        let size = width as usize * height as usize * P::size();
        if buffer.len() < size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        let pixels = align_buffer_to_mut(buffer)?;
        Self::from_pixels(width, height, pixels).map_err(|_| ImageBufferError::InvalidBufferSize)
    }
}

unsafe impl<'a, P: InnerPixel> ImageView for TypedImage<'a, P> {
    type Pixel = P;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn iter_rows(&self, start_row: u32) -> impl Iterator<Item = &[Self::Pixel]> {
        let width = self.width as usize;
        if width == 0 {
            [].chunks_exact(1)
        } else {
            let start = start_row as usize * width;
            self.pixels
                .borrow()
                .get(start..)
                .unwrap_or_default()
                .chunks_exact(width)
        }
    }
}

unsafe impl<'a, P: InnerPixel> ImageViewMut for TypedImage<'a, P> {
    fn iter_rows_mut(&mut self, start_row: u32) -> impl Iterator<Item = &mut [Self::Pixel]> {
        let width = self.width as usize;
        if width == 0 {
            [].chunks_exact_mut(1)
        } else {
            let start = start_row as usize * width;
            self.pixels
                .borrow_mut()
                .get_mut(start..)
                .unwrap_or_default()
                .chunks_exact_mut(width)
        }
    }
}

pub(crate) fn align_buffer_to<T>(buffer: &[u8]) -> Result<&[T], ImageBufferError> {
    let (head, pixels, _) = unsafe { buffer.align_to::<T>() };
    if !head.is_empty() {
        return Err(ImageBufferError::InvalidBufferAlignment);
    }
    Ok(pixels)
}

pub(crate) fn align_buffer_to_mut<T>(buffer: &mut [u8]) -> Result<&mut [T], ImageBufferError> {
    let (head, pixels, _) = unsafe { buffer.align_to_mut::<T>() };
    if !head.is_empty() {
        return Err(ImageBufferError::InvalidBufferAlignment);
    }
    Ok(pixels)
}
