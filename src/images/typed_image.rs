use crate::images::BufferContainer;
use crate::pixels::InnerPixel;
use crate::{ImageBufferError, ImageView, ImageViewMut, InvalidPixelsSize};
use std::fmt::Debug;
use std::num::NonZeroU32;

/// Generic reference to image data that provides [ImageView].
#[derive(Debug)]
pub struct TypedImageRef<'a, P> {
    width: u32,
    height: u32,
    pixels: &'a [P],
}

impl<'a, P> TypedImageRef<'a, P> {
    pub fn new(width: u32, height: u32, pixels: &'a [P]) -> Result<Self, InvalidPixelsSize> {
        let pixels_count = width as usize * height as usize;
        if pixels.len() < pixels_count {
            return Err(InvalidPixelsSize);
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

    pub fn pixels(&self) -> &[P] {
        self.pixels
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

    fn split_by_height(
        &self,
        start_row: u32,
        height: NonZeroU32,
        num_parts: NonZeroU32,
    ) -> Option<Vec<impl ImageView<Pixel = Self::Pixel>>> {
        let height = height.get();
        let num_parts = num_parts.get();
        if num_parts > height || height > self.height() || start_row > self.height() - height {
            return None;
        }
        let mut res = Vec::with_capacity(num_parts as usize);
        let step = height / num_parts;
        let mut modulo = height % num_parts;
        let mut top = start_row;
        let row_size = self.width as usize;
        let mut remains_pixels = self.pixels.split_at(top as usize * row_size).1;
        for _ in 0..num_parts {
            let mut part_height = step;
            if modulo > 0 {
                part_height += 1;
                modulo -= 1;
            }
            let parts = remains_pixels.split_at(part_height as usize * row_size);
            let image = TypedImageRef::new(self.width, part_height, parts.0).unwrap();
            res.push(image);
            remains_pixels = parts.1;
            top += part_height;
        }
        Some(res)
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
    pub fn from_pixels(width: u32, height: u32, pixels: Vec<P>) -> Result<Self, InvalidPixelsSize> {
        let pixels_count = width as usize * height as usize;
        if pixels.len() < pixels_count {
            return Err(InvalidPixelsSize);
        }
        Ok(Self {
            width,
            height,
            pixels: BufferContainer::Owned(pixels),
        })
    }

    pub fn from_pixels_slice(
        width: u32,
        height: u32,
        pixels: &'a mut [P],
    ) -> Result<Self, InvalidPixelsSize> {
        let pixels_count = width as usize * height as usize;
        if pixels.len() < pixels_count {
            return Err(InvalidPixelsSize);
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
        Self::from_pixels_slice(width, height, pixels)
            .map_err(|_| ImageBufferError::InvalidBufferSize)
    }

    pub fn pixels(&self) -> &[P] {
        self.pixels.borrow()
    }

    pub fn pixels_mut(&mut self) -> &mut [P] {
        self.pixels.borrow_mut()
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

    fn split_by_height(
        &self,
        start_row: u32,
        height: NonZeroU32,
        num_parts: NonZeroU32,
    ) -> Option<Vec<impl ImageView<Pixel = Self::Pixel>>> {
        let height = height.get();
        let num_parts = num_parts.get();
        if num_parts > height || height > self.height() || start_row > self.height() - height {
            return None;
        }
        let mut res = Vec::with_capacity(num_parts as usize);
        let step = height / num_parts;
        let mut modulo = height % num_parts;
        let mut top = start_row;
        let row_size = self.width as usize;
        let mut remains_pixels = self.pixels.borrow().split_at(top as usize * row_size).1;
        for _ in 0..num_parts {
            let mut part_height = step;
            if modulo > 0 {
                part_height += 1;
                modulo -= 1;
            }
            let parts = remains_pixels.split_at(part_height as usize * row_size);
            let image = TypedImageRef::new(self.width, part_height, parts.0).unwrap();
            res.push(image);
            remains_pixels = parts.1;
            top += part_height;
        }
        debug_assert!(top - start_row == height);
        Some(res)
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

    fn split_by_height_mut(
        &mut self,
        start_row: u32,
        height: NonZeroU32,
        num_parts: NonZeroU32,
    ) -> Option<Vec<impl ImageViewMut<Pixel = Self::Pixel>>> {
        let height = height.get();
        let num_parts = num_parts.get();
        if num_parts > height || height > self.height() || start_row > self.height() - height {
            return None;
        }
        let mut res = Vec::with_capacity(num_parts as usize);
        let step = height / num_parts;
        let mut modulo = height % num_parts;
        let mut top = start_row;
        let row_size = self.width as usize;
        let mut remains_pixels = self
            .pixels
            .borrow_mut()
            .split_at_mut(top as usize * row_size)
            .1;
        for _ in 0..num_parts {
            let mut part_height = step;
            if modulo > 0 {
                part_height += 1;
                modulo -= 1;
            }
            let parts = remains_pixels.split_at_mut(part_height as usize * row_size);
            let image = TypedImage::from_pixels_slice(self.width, part_height, parts.0).unwrap();
            res.push(image);
            remains_pixels = parts.1;
            top += part_height;
        }
        debug_assert!(top - start_row == height);
        Some(res)
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
