use std::num::NonZeroU32;

use crate::image_view::{ImageRows, ImageRowsMut, TypedImageView, TypedImageViewMut};
use crate::pixels::Pixel;
use crate::{ImageBufferError, ImageView, ImageViewMut, InvalidBufferSizeError, PixelType};

#[derive(Debug)]
enum PixelsContainer<'a> {
    MutU32(&'a mut [u32]),
    MutU8(&'a mut [u8]),
    VecU32(Vec<u32>),
    VecU8(Vec<u8>),
}

/// Simple image container.
#[derive(Debug)]
pub struct Image<'a> {
    width: NonZeroU32,
    height: NonZeroU32,
    pixels: PixelsContainer<'a>,
    pixel_type: PixelType,
}

impl<'a> Image<'a> {
    /// Create empty image with given dimensions and pixel type.
    pub fn new(width: NonZeroU32, height: NonZeroU32, pixel_type: PixelType) -> Self {
        let size = (width.get() * height.get()) as usize;
        let pixels = if let PixelType::U8 = pixel_type {
            PixelsContainer::VecU8(vec![0; size])
        } else {
            PixelsContainer::VecU32(vec![0; size])
        };
        Self {
            width,
            height,
            pixels,
            pixel_type,
        }
    }

    pub fn from_vec_u32(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: Vec<u32>,
        pixel_type: PixelType,
    ) -> Result<Self, InvalidBufferSizeError> {
        let size = (width.get() * height.get()) as usize;
        if buffer.len() != size {
            return Err(InvalidBufferSizeError);
        }
        Ok(Self {
            width,
            height,
            pixels: PixelsContainer::VecU32(buffer),
            pixel_type,
        })
    }

    pub fn from_vec_u8(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: Vec<u8>,
        pixel_type: PixelType,
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize * pixel_type.size();
        if buffer.len() != size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        if !pixel_type.is_aligned(&buffer) {
            return Err(ImageBufferError::InvalidBufferAlignment);
        }
        Ok(Self {
            width,
            height,
            pixels: PixelsContainer::VecU8(buffer),
            pixel_type,
        })
    }

    pub fn from_slice_u32(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: &'a mut [u32],
        pixel_type: PixelType,
    ) -> Result<Self, InvalidBufferSizeError> {
        let size = (width.get() * height.get()) as usize;
        if buffer.len() != size {
            return Err(InvalidBufferSizeError);
        }
        Ok(Self {
            width,
            height,
            pixels: PixelsContainer::MutU32(buffer),
            pixel_type,
        })
    }

    pub fn from_slice_u8(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: &'a mut [u8],
        pixel_type: PixelType,
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize * pixel_type.size();
        if buffer.len() != size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        if !pixel_type.is_aligned(buffer) {
            return Err(ImageBufferError::InvalidBufferAlignment);
        }
        Ok(Self {
            width,
            height,
            pixels: PixelsContainer::MutU8(buffer),
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

    /// Buffer with image pixels.
    #[inline(always)]
    pub fn buffer(&self) -> &[u8] {
        match &self.pixels {
            PixelsContainer::MutU32(p) => unsafe { p.align_to::<u8>().1 },
            PixelsContainer::MutU8(p) => *p,
            PixelsContainer::VecU32(v) => unsafe { v.align_to::<u8>().1 },
            PixelsContainer::VecU8(v) => v,
        }
    }

    #[inline(always)]
    fn buffer_mut(&mut self) -> &mut [u8] {
        match &mut self.pixels {
            PixelsContainer::MutU32(p) => unsafe { p.align_to_mut::<u8>().1 },
            PixelsContainer::MutU8(p) => p,
            PixelsContainer::VecU32(ref mut v) => unsafe { v.align_to_mut::<u8>().1 },
            PixelsContainer::VecU8(ref mut v) => v.as_mut_slice(),
        }
    }

    #[inline(always)]
    pub fn view(&self) -> ImageView {
        let buffer = self.buffer();
        let rows = match self.pixel_type {
            PixelType::U8x4 => {
                let pixels = unsafe { buffer.align_to::<u32>().1 };
                ImageRows::U8x4(pixels.chunks(self.width.get() as usize).collect())
            }
            PixelType::I32 => {
                let pixels = unsafe { buffer.align_to::<i32>().1 };
                ImageRows::I32(pixels.chunks(self.width.get() as usize).collect())
            }
            PixelType::F32 => {
                let pixels = unsafe { buffer.align_to::<f32>().1 };
                ImageRows::F32(pixels.chunks(self.width.get() as usize).collect())
            }
            PixelType::U8 => ImageRows::U8(buffer.chunks(self.width.get() as usize).collect()),
        };
        ImageView::new(self.width, self.height, rows).unwrap()
    }

    #[inline(always)]
    pub fn view_mut(&mut self) -> ImageViewMut {
        let pixel_type = self.pixel_type;
        let width = self.width;
        let height = self.height;
        let buffer = self.buffer_mut();
        let rows = match pixel_type {
            PixelType::U8x4 => {
                let pixels = unsafe { buffer.align_to_mut::<u32>().1 };
                ImageRowsMut::U8x4(pixels.chunks_mut(width.get() as usize).collect())
            }
            PixelType::I32 => {
                let pixels = unsafe { buffer.align_to_mut::<i32>().1 };
                ImageRowsMut::I32(pixels.chunks_mut(width.get() as usize).collect())
            }
            PixelType::F32 => {
                let pixels = unsafe { buffer.align_to_mut::<f32>().1 };
                ImageRowsMut::F32(pixels.chunks_mut(width.get() as usize).collect())
            }
            PixelType::U8 => ImageRowsMut::U8(buffer.chunks_mut(width.get() as usize).collect()),
        };
        ImageViewMut::new(width, height, rows).unwrap()
    }
}

/// Generic image container for internal purposes.
pub(crate) struct InnerImage<'a, P>
where
    P: Pixel,
{
    width: NonZeroU32,
    height: NonZeroU32,
    rows: Vec<&'a mut [P::Type]>,
}

impl<'a, P> InnerImage<'a, P>
where
    P: Pixel,
{
    pub fn new(width: NonZeroU32, height: NonZeroU32, pixels: &'a mut [P::Type]) -> Self {
        let rows = pixels.chunks_mut(width.get() as usize).collect();
        Self {
            width,
            height,
            rows,
        }
    }

    #[inline(always)]
    pub fn src_view<'s>(&'s self) -> TypedImageView<'s, 'a, P> {
        let rows = self.rows.as_slice();
        let rows: &[&[P::Type]] = unsafe { std::mem::transmute(rows) };
        TypedImageView::new(self.width, self.height, rows)
    }

    #[inline(always)]
    pub fn dst_view<'s>(&'s mut self) -> TypedImageViewMut<'s, 'a, P> {
        TypedImageViewMut::new(self.width, self.height, self.rows.as_mut_slice())
    }
}
