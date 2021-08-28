use std::num::NonZeroU32;

use crate::{DstImageView, ImageBufferError, InvalidBufferSizeError, PixelType, SrcImageView};

#[derive(Debug)]
enum PixelsContainer<'a> {
    Mut(&'a mut [u32]),
    VecU32(Vec<u32>),
    VecU8(Vec<u8>),
}

#[derive(Debug)]
pub struct ImageData<'a> {
    width: NonZeroU32,
    height: NonZeroU32,
    pixels: PixelsContainer<'a>,
    pixel_type: PixelType,
}

impl<'a> ImageData<'a> {
    pub fn new(width: NonZeroU32, height: NonZeroU32, pixel_type: PixelType) -> Self {
        let size = (width.get() * height.get()) as usize;
        let pixels = vec![0; size];
        Self {
            width,
            height,
            pixels: PixelsContainer::VecU32(pixels),
            pixel_type,
        }
    }

    pub fn from_vec_u32(
        width: NonZeroU32,
        height: NonZeroU32,
        pixels: Vec<u32>,
        pixel_type: PixelType,
    ) -> Result<Self, InvalidBufferSizeError> {
        let size = (width.get() * height.get()) as usize;
        if pixels.len() != size {
            return Err(InvalidBufferSizeError);
        }
        Ok(Self {
            width,
            height,
            pixels: PixelsContainer::VecU32(pixels),
            pixel_type,
        })
    }

    pub fn from_vec_u8(
        width: NonZeroU32,
        height: NonZeroU32,
        mut pixels: Vec<u8>,
        pixel_type: PixelType,
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize * 4;
        if pixels.len() != size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        let (head, _, _) = unsafe { &pixels.align_to_mut::<u32>() };
        if !head.is_empty() {
            return Err(ImageBufferError::InvalidBufferAlignment);
        }
        Ok(Self {
            width,
            height,
            pixels: PixelsContainer::VecU8(pixels),
            pixel_type,
        })
    }

    pub fn from_slice_u32(
        width: NonZeroU32,
        height: NonZeroU32,
        pixels: &'a mut [u32],
        pixel_type: PixelType,
    ) -> Result<Self, InvalidBufferSizeError> {
        let size = (width.get() * height.get()) as usize;
        if pixels.len() != size {
            return Err(InvalidBufferSizeError);
        }
        Ok(Self {
            width,
            height,
            pixels: PixelsContainer::Mut(pixels),
            pixel_type,
        })
    }

    pub fn from_slice_u8(
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
        Ok(Self {
            width,
            height,
            pixels: PixelsContainer::Mut(pixels),
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
        match &self.pixels {
            PixelsContainer::Mut(p) => p,
            PixelsContainer::VecU32(v) => v,
            PixelsContainer::VecU8(v) => unsafe { v.align_to::<u32>().1 },
        }
    }

    #[inline(always)]
    pub fn get_buffer(&self) -> &[u8] {
        let pixels = self.get_pixels();
        let (_, buffer, _) = unsafe { pixels.align_to::<u8>() };
        buffer
    }

    #[inline(always)]
    pub fn src_view(&self) -> SrcImageView {
        let pixels = self.get_pixels();
        let rows = pixels.chunks(self.width.get() as usize).collect();
        SrcImageView::from_rows(self.width, self.height, rows, self.pixel_type).unwrap()
    }

    #[inline(always)]
    pub fn dst_view(&mut self) -> DstImageView {
        let rows = match &mut self.pixels {
            PixelsContainer::Mut(p) => p.chunks_mut(self.width.get() as usize).collect(),
            PixelsContainer::VecU32(v) => v.chunks_mut(self.width.get() as usize).collect(),
            PixelsContainer::VecU8(v) => {
                let p = unsafe { v.align_to_mut::<u32>().1 };
                p.chunks_mut(self.width.get() as usize).collect()
            }
        };
        DstImageView::from_rows(self.width, self.height, rows, self.pixel_type).unwrap()
    }
}
