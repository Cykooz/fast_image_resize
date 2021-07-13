use std::num::NonZeroU32;

use crate::{DstImageView, ImageBufferError, PixelType, SrcImageView};

#[derive(Debug, Clone)]
pub struct ImageData<T: AsRef<[u32]>> {
    width: NonZeroU32,
    height: NonZeroU32,
    pixels: T,
    pixel_type: PixelType,
}

impl<T: AsRef<[u32]>> ImageData<T> {
    pub fn new(
        width: NonZeroU32,
        height: NonZeroU32,
        pixels: T,
        pixel_type: PixelType,
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize;
        if pixels.as_ref().len() != size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        Ok(Self {
            width,
            height,
            pixels,
            pixel_type,
        })
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
    pub fn get_pixels(&self) -> &[u32] {
        self.pixels.as_ref()
    }

    #[inline(always)]
    pub fn get_buffer(&self) -> &[u8] {
        let (_, buffer, _) = unsafe { self.pixels.as_ref().align_to::<u8>() };
        buffer
    }

    #[inline(always)]
    pub fn src_view(&self) -> SrcImageView {
        let pixels = self.pixels.as_ref();
        let rows = pixels.chunks(self.width.get() as usize).collect();
        SrcImageView::from_rows(self.width, self.height, rows, self.pixel_type).unwrap()
    }
}

impl<T: AsRef<[u32]> + AsMut<[u32]>> ImageData<T> {
    #[inline(always)]
    pub fn dst_view(&mut self) -> DstImageView {
        let pixels = self.pixels.as_mut();
        let rows = pixels.chunks_mut(self.width.get() as usize).collect();
        DstImageView::from_rows(self.width, self.height, rows, self.pixel_type).unwrap()
    }
}

impl ImageData<Vec<u32>> {
    pub fn new_owned(width: NonZeroU32, height: NonZeroU32, pixel_type: PixelType) -> Self {
        let size = (width.get() * height.get()) as usize;
        let pixels = vec![0; size];
        Self {
            width,
            height,
            pixels,
            pixel_type,
        }
    }

    pub fn from_buffer(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: &[u8],
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
        Ok(Self {
            width,
            height,
            pixels: pixels.to_owned(),
            pixel_type,
        })
    }
}
