use std::num::NonZeroU32;

use crate::pixels::{PixelExt, PixelType};
use crate::{DynamicImageView, DynamicImageViewMut, ImageBufferError, ImageView, ImageViewMut};

#[derive(Debug)]
enum BufferContainer<'a> {
    MutU8(&'a mut [u8]),
    VecU8(Vec<u8>),
}

impl<'a> BufferContainer<'a> {
    fn as_vec(&self) -> Vec<u8> {
        match self {
            Self::MutU8(slice) => slice.to_vec(),
            Self::VecU8(vec) => vec.clone(),
        }
    }
}

/// Simple container of image data.
#[derive(Debug)]
pub struct Image<'a> {
    width: NonZeroU32,
    height: NonZeroU32,
    buffer: BufferContainer<'a>,
    pixel_type: PixelType,
}

impl<'a> Image<'a> {
    /// Create empty image with given dimensions and pixel type.
    pub fn new(width: NonZeroU32, height: NonZeroU32, pixel_type: PixelType) -> Self {
        let pixels_count = (width.get() * height.get()) as usize;
        let buffer = BufferContainer::VecU8(vec![0; pixels_count * pixel_type.size()]);
        Self {
            width,
            height,
            buffer,
            pixel_type,
        }
    }

    pub fn from_vec_u8(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: Vec<u8>,
        pixel_type: PixelType,
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize * pixel_type.size();
        if buffer.len() < size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        if !pixel_type.is_aligned(&buffer) {
            return Err(ImageBufferError::InvalidBufferAlignment);
        }
        Ok(Self {
            width,
            height,
            buffer: BufferContainer::VecU8(buffer),
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
        if buffer.len() < size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        if !pixel_type.is_aligned(buffer) {
            return Err(ImageBufferError::InvalidBufferAlignment);
        }
        Ok(Self {
            width,
            height,
            buffer: BufferContainer::MutU8(buffer),
            pixel_type,
        })
    }

    /// Creates a copy of the image.
    pub fn copy(&self) -> Image<'static> {
        Image {
            width: self.width,
            height: self.height,
            buffer: BufferContainer::VecU8(self.buffer.as_vec()),
            pixel_type: self.pixel_type,
        }
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
        match &self.buffer {
            BufferContainer::MutU8(p) => p,
            BufferContainer::VecU8(v) => v,
        }
    }

    /// Mutable buffer with image pixels.
    #[inline(always)]
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        match &mut self.buffer {
            BufferContainer::MutU8(p) => p,
            BufferContainer::VecU8(ref mut v) => v.as_mut_slice(),
        }
    }

    #[inline(always)]
    pub fn into_vec(self) -> Vec<u8> {
        match self.buffer {
            BufferContainer::MutU8(p) => p.into(),
            BufferContainer::VecU8(v) => v,
        }
    }

    #[inline(always)]
    pub fn view(&self) -> DynamicImageView {
        macro_rules! get_dynamic_image {
            ($img_type: expr) => {
                ($img_type(ImageView::from_buffer(self.width, self.height, self.buffer()).unwrap()))
            };
        }

        match self.pixel_type {
            PixelType::U8 => get_dynamic_image!(DynamicImageView::U8),
            PixelType::U8x2 => get_dynamic_image!(DynamicImageView::U8x2),
            PixelType::U8x3 => get_dynamic_image!(DynamicImageView::U8x3),
            PixelType::U8x4 => get_dynamic_image!(DynamicImageView::U8x4),
            PixelType::U16 => get_dynamic_image!(DynamicImageView::U16),
            PixelType::U16x2 => get_dynamic_image!(DynamicImageView::U16x2),
            PixelType::U16x3 => get_dynamic_image!(DynamicImageView::U16x3),
            PixelType::U16x4 => get_dynamic_image!(DynamicImageView::U16x4),
            PixelType::I32 => get_dynamic_image!(DynamicImageView::I32),
            PixelType::F32 => get_dynamic_image!(DynamicImageView::F32),
        }
    }

    #[inline(always)]
    pub fn view_mut(&mut self) -> DynamicImageViewMut {
        macro_rules! get_dynamic_image {
            ($img_type: expr) => {
                ($img_type(
                    ImageViewMut::from_buffer(self.width, self.height, self.buffer_mut()).unwrap(),
                ))
            };
        }

        match self.pixel_type {
            PixelType::U8 => get_dynamic_image!(DynamicImageViewMut::U8),
            PixelType::U8x2 => get_dynamic_image!(DynamicImageViewMut::U8x2),
            PixelType::U8x3 => get_dynamic_image!(DynamicImageViewMut::U8x3),
            PixelType::U8x4 => get_dynamic_image!(DynamicImageViewMut::U8x4),
            PixelType::U16 => get_dynamic_image!(DynamicImageViewMut::U16),
            PixelType::U16x2 => get_dynamic_image!(DynamicImageViewMut::U16x2),
            PixelType::U16x3 => get_dynamic_image!(DynamicImageViewMut::U16x3),
            PixelType::U16x4 => get_dynamic_image!(DynamicImageViewMut::U16x4),
            PixelType::I32 => get_dynamic_image!(DynamicImageViewMut::I32),
            PixelType::F32 => get_dynamic_image!(DynamicImageViewMut::F32),
        }
    }
}

/// Generic image container for internal purposes.
pub(crate) struct InnerImage<'a, P>
where
    P: PixelExt,
{
    width: NonZeroU32,
    height: NonZeroU32,
    pixels: &'a mut [P],
}

impl<'a, P> InnerImage<'a, P>
where
    P: PixelExt,
{
    pub fn new(width: NonZeroU32, height: NonZeroU32, pixels: &'a mut [P]) -> Self {
        Self {
            width,
            height,
            pixels,
        }
    }

    #[inline(always)]
    pub fn src_view(&self) -> ImageView<P> {
        ImageView::from_pixels(self.width, self.height, self.pixels).unwrap()
    }

    #[inline(always)]
    pub fn dst_view(&mut self) -> ImageViewMut<P> {
        ImageViewMut::from_pixels(self.width, self.height, self.pixels).unwrap()
    }
}
