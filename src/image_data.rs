use std::num::NonZeroU32;

use crate::{DstImageView, ImageError, PixelType, SrcImageView};

#[derive(Debug, Clone)]
pub struct ImageData<T: AsRef<[u8]>> {
    width: NonZeroU32,
    height: NonZeroU32,
    pixels: T,
    pixel_type: PixelType,
}

impl<T: AsRef<[u8]>> ImageData<T> {
    pub fn new(
        width: NonZeroU32,
        height: NonZeroU32,
        pixels: T,
        pixel_type: PixelType,
    ) -> Result<Self, ImageError> {
        let size = (width.get() * height.get()) as usize * 4;
        if pixels.as_ref().len() != size {
            return Err(ImageError::InvalidBufferSize);
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
    pub fn get_buffer(&self) -> &[u8] {
        self.pixels.as_ref()
    }

    #[inline(always)]
    pub fn src_view(&self) -> SrcImageView {
        let pixels = unsafe { self.pixels.as_ref().align_to::<u32>().1 };
        let rows = pixels.chunks(self.width.get() as usize).collect();
        SrcImageView::new(self.width, self.height, rows, self.pixel_type).unwrap()
    }
}

impl<T: AsRef<[u8]> + AsMut<[u8]>> ImageData<T> {
    #[inline(always)]
    pub fn dst_view(&mut self) -> DstImageView {
        let pixels = unsafe { self.pixels.as_mut().align_to_mut::<u32>().1 };
        let rows = pixels.chunks_mut(self.width.get() as usize).collect();
        DstImageView::new(self.width, self.height, rows, self.pixel_type).unwrap()
    }
}

impl ImageData<Vec<u8>> {
    pub fn new_owned(width: NonZeroU32, height: NonZeroU32, pixel_type: PixelType) -> Self {
        let size = (width.get() * height.get()) as usize * 4;
        let pixels = vec![0; size];
        Self {
            width,
            height,
            pixels,
            pixel_type,
        }
    }
}
