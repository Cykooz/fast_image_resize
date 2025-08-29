use crate::images::{BufferContainer, TypedImage, TypedImageRef};
use crate::pixels::InnerPixel;
use crate::{
    ImageBufferError, ImageView, ImageViewMut, IntoImageView, IntoImageViewMut, PixelTrait,
    PixelType,
};

/// Simple reference to image data that provides [IntoImageView].
#[derive(Debug, Copy, Clone)]
pub struct ImageRef<'a> {
    width: u32,
    height: u32,
    buffer: &'a [u8],
    pixel_type: PixelType,
}

impl<'a> ImageRef<'a> {
    /// Create an image from slice with pixels-data.
    pub fn new(
        width: u32,
        height: u32,
        buffer: &'a [u8],
        pixel_type: PixelType,
    ) -> Result<Self, ImageBufferError> {
        let size = width as usize * height as usize * pixel_type.size();
        if buffer.len() < size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        if !pixel_type.is_aligned(buffer) {
            return Err(ImageBufferError::InvalidBufferAlignment);
        }
        Ok(Self {
            width,
            height,
            buffer,
            pixel_type,
        })
    }

    pub fn from_pixels<P: PixelTrait>(
        width: u32,
        height: u32,
        pixels: &'a [P],
    ) -> Result<Self, ImageBufferError> {
        let (head, buffer, _) = unsafe { pixels.align_to::<u8>() };
        if !head.is_empty() {
            return Err(ImageBufferError::InvalidBufferAlignment);
        }
        Self::new(width, height, buffer, P::pixel_type())
    }

    #[inline]
    pub fn pixel_type(&self) -> PixelType {
        self.pixel_type
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Buffer with image pixels data.
    #[inline]
    pub fn buffer(&self) -> &[u8] {
        self.buffer
    }

    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        self.buffer.into()
    }

    /// Get the typed version of the image.
    pub fn typed_image<P: InnerPixel>(&self) -> Option<TypedImageRef<'_, P>> {
        if P::pixel_type() != self.pixel_type {
            return None;
        }
        let typed_image = TypedImageRef::from_buffer(self.width, self.height, self.buffer).unwrap();
        Some(typed_image)
    }
}

impl<'a> IntoImageView for ImageRef<'a> {
    fn pixel_type(&self) -> Option<PixelType> {
        Some(self.pixel_type)
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn image_view<P: PixelTrait>(&self) -> Option<impl ImageView<Pixel = P>> {
        self.typed_image()
    }
}

/// Simple dynamic container of image data that provides [IntoImageView] and [IntoImageViewMut].
#[derive(Debug)]
pub struct Image<'a> {
    width: u32,
    height: u32,
    buffer: BufferContainer<'a, u8>,
    pixel_type: PixelType,
}

impl Image<'static> {
    /// Create an empty image with given dimensions and pixel type.
    pub fn new(width: u32, height: u32, pixel_type: PixelType) -> Self {
        let pixels_count = width as usize * height as usize;
        let buffer = BufferContainer::Owned(vec![0; pixels_count * pixel_type.size()]);
        Self {
            width,
            height,
            buffer,
            pixel_type,
        }
    }

    /// Create an image from vector with pixels data.
    pub fn from_vec_u8(
        width: u32,
        height: u32,
        buffer: Vec<u8>,
        pixel_type: PixelType,
    ) -> Result<Self, ImageBufferError> {
        let size = width as usize * height as usize * pixel_type.size();
        if buffer.len() < size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        if !pixel_type.is_aligned(&buffer) {
            return Err(ImageBufferError::InvalidBufferAlignment);
        }
        Ok(Self {
            width,
            height,
            buffer: BufferContainer::Owned(buffer),
            pixel_type,
        })
    }
}

impl<'a> Image<'a> {
    /// Create an image from slice with pixels data.
    pub fn from_slice_u8(
        width: u32,
        height: u32,
        buffer: &'a mut [u8],
        pixel_type: PixelType,
    ) -> Result<Self, ImageBufferError> {
        let size = width as usize * height as usize * pixel_type.size();
        if buffer.len() < size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        if !pixel_type.is_aligned(buffer) {
            return Err(ImageBufferError::InvalidBufferAlignment);
        }
        Ok(Self {
            width,
            height,
            buffer: BufferContainer::Borrowed(buffer),
            pixel_type,
        })
    }

    #[inline]
    pub fn pixel_type(&self) -> PixelType {
        self.pixel_type
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Buffer with image pixels data.
    #[inline]
    pub fn buffer(&self) -> &[u8] {
        match &self.buffer {
            BufferContainer::Borrowed(p) => p,
            BufferContainer::Owned(v) => v,
        }
    }

    /// Mutable buffer with image pixels data.
    #[inline]
    pub fn buffer_mut(&mut self) -> &mut [u8] {
        match &mut self.buffer {
            BufferContainer::Borrowed(p) => p,
            BufferContainer::Owned(ref mut v) => v.as_mut_slice(),
        }
    }

    #[inline]
    pub fn into_vec(self) -> Vec<u8> {
        match self.buffer {
            BufferContainer::Borrowed(p) => p.into(),
            BufferContainer::Owned(v) => v,
        }
    }

    /// Creates a copy of the image.
    pub fn copy(&self) -> Image<'static> {
        Image {
            width: self.width,
            height: self.height,
            buffer: BufferContainer::Owned(self.buffer.as_vec()),
            pixel_type: self.pixel_type,
        }
    }

    /// Get the typed version of the image.
    pub fn typed_image<P: InnerPixel>(&self) -> Option<TypedImageRef<'_, P>> {
        if P::pixel_type() != self.pixel_type {
            return None;
        }
        let typed_image =
            TypedImageRef::from_buffer(self.width, self.height, self.buffer()).unwrap();
        Some(typed_image)
    }

    /// Get the typed mutable version of the image.
    pub fn typed_image_mut<P: InnerPixel>(&mut self) -> Option<TypedImage<'_, P>> {
        if P::pixel_type() != self.pixel_type {
            return None;
        }
        let typed_image =
            TypedImage::from_buffer(self.width, self.height, self.buffer_mut()).unwrap();
        Some(typed_image)
    }
}

impl<'a> IntoImageView for Image<'a> {
    fn pixel_type(&self) -> Option<PixelType> {
        Some(self.pixel_type)
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn image_view<P: PixelTrait>(&self) -> Option<impl ImageView<Pixel = P>> {
        self.typed_image()
    }
}

impl<'a> IntoImageViewMut for Image<'a> {
    fn image_view_mut<P: PixelTrait>(&mut self) -> Option<impl ImageViewMut<Pixel = P>> {
        self.typed_image_mut()
    }
}
