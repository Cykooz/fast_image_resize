//! Contains types of pixels.
use std::fmt::Debug;
use std::mem::size_of;
use std::slice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PixelType {
    U8x2,
    U8x3,
    U8x4,
    U16,
    U16x2,
    U16x3,
    U16x4,
    I32,
    F32,
    U8,
}

impl PixelType {
    pub(crate) fn size(&self) -> usize {
        match self {
            Self::U8 => 1,
            Self::U8x2 => 2,
            Self::U8x3 => 3,
            Self::U16 => 2,
            Self::U16x2 => 4,
            Self::U16x3 => 6,
            Self::U16x4 => 8,
            _ => 4,
        }
    }

    /// Returns `true` if given buffer is aligned by the alignment of pixel.
    pub(crate) fn is_aligned(&self, buffer: &[u8]) -> bool {
        match self {
            Self::U8 => true,
            Self::U8x2 => unsafe { buffer.align_to::<U8x2>().0.is_empty() },
            Self::U8x3 => unsafe { buffer.align_to::<U8x3>().0.is_empty() },
            Self::U8x4 => unsafe { buffer.align_to::<U8x4>().0.is_empty() },
            Self::U16 => unsafe { buffer.align_to::<U16>().0.is_empty() },
            Self::U16x2 => unsafe { buffer.align_to::<U16x2>().0.is_empty() },
            Self::U16x3 => unsafe { buffer.align_to::<U16x3>().0.is_empty() },
            Self::U16x4 => unsafe { buffer.align_to::<U16x4>().0.is_empty() },
            Self::I32 => unsafe { buffer.align_to::<I32>().0.is_empty() },
            Self::F32 => unsafe { buffer.align_to::<F32>().0.is_empty() },
        }
    }
}

pub trait GetCount {
    fn count() -> usize;
}

/// Generic type to represent the number of components in single pixel.
pub struct Count<const N: usize>;

impl<const N: usize> GetCount for Count<N> {
    fn count() -> usize {
        N
    }
}

pub trait CountOfValues {
    fn count_of_values() -> usize;
}

/// Generic type to represent the number of available values for a single pixel component.
pub struct Values<const N: usize>;

impl<const N: usize> CountOfValues for Values<N> {
    fn count_of_values() -> usize {
        N
    }
}

pub trait PixelComponent
where
    Self: Sized + Copy + Debug + 'static,
{
}

impl PixelComponent for u8 {}
impl PixelComponent for u16 {}
impl PixelComponent for i32 {}
impl PixelComponent for f32 {}

/// Additional information about pixel type.
pub trait Pixel
where
    Self: Copy + Clone + Sized + Debug,
{
    /// Type of pixel components
    type Component: PixelComponent;
    /// Type that provides information about a count of pixel's components
    type CountOfComponents: GetCount;
    /// Type that provides information about a count of available values of one
    /// pixel's component
    type CountOfComponentValues: CountOfValues;

    fn pixel_type() -> PixelType;

    /// Count of pixel's components
    fn count_of_components() -> usize {
        Self::CountOfComponents::count()
    }

    /// Count of available values of one pixel's component
    fn count_of_component_values() -> usize {
        Self::CountOfComponentValues::count_of_values()
    }

    /// Size of pixel in bytes
    ///
    /// Example:
    /// ```
    /// # use fast_image_resize::pixels::{U8x2, U8x3, U8, Pixel};
    /// assert_eq!(U8x3::size(), 3);
    /// assert_eq!(U8x2::size(), 2);
    /// assert_eq!(U8::size(), 1);
    /// ```
    fn size() -> usize {
        size_of::<Self>()
    }

    /// Create slice of components of pixels from slice of pixels
    fn components(buf: &[Self]) -> &[Self::Component] {
        let size = buf.len() * Self::count_of_components();
        let components_ptr = buf.as_ptr() as *const Self::Component;
        unsafe { slice::from_raw_parts(components_ptr, size) }
    }

    /// Create mutable slice of components of pixels from mutable slice of pixels
    fn components_mut(buf: &mut [Self]) -> &mut [Self::Component] {
        let size = buf.len() * Self::count_of_components();
        let components_ptr = buf.as_mut_ptr() as *mut Self::Component;
        unsafe { slice::from_raw_parts_mut(components_ptr, size) }
    }
}

macro_rules! pixel_struct {
    ($name:ident, $type:tt, $comp_type:tt, $comp_count:literal, $comp_values:literal, $pixel_type:expr, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy, PartialEq)]
        #[repr(C)]
        pub struct $name(pub $type);

        impl Pixel for $name {
            type Component = $comp_type;
            type CountOfComponents = Count<$comp_count>;
            type CountOfComponentValues = Values<$comp_values>;

            fn pixel_type() -> PixelType {
                $pixel_type
            }
        }
    };
}

pixel_struct!(
    U8,
    u8,
    u8,
    1,
    256,
    PixelType::U8,
    "One byte per pixel (e.g. L8)"
);
pixel_struct!(
    U8x2,
    u16,
    u8,
    2,
    256,
    PixelType::U8x2,
    "Two bytes per pixel (e.g. LA8)"
);
pixel_struct!(
    U8x3,
    [u8; 3],
    u8,
    3,
    256,
    PixelType::U8x3,
    "Three bytes per pixel (e.g. RGB8)"
);
pixel_struct!(
    U8x4,
    u32,
    u8,
    4,
    256,
    PixelType::U8x4,
    "Four bytes per pixel (RGBA8, RGBx8, CMYK8 and other)"
);
pixel_struct!(
    U16,
    u16,
    u16,
    1,
    65536,
    PixelType::U16,
    "One `u16` component per pixel (e.g. L16)"
);
pixel_struct!(
    U16x2,
    [u16; 2],
    u16,
    2,
    65536,
    PixelType::U16x2,
    "Two `u16` components per pixel (e.g. LA16)"
);
pixel_struct!(
    U16x3,
    [u16; 3],
    u16,
    3,
    65536,
    PixelType::U16x3,
    "Three `u16` components per pixel (e.g. RGB16)"
);
pixel_struct!(
    U16x4,
    [u16; 4],
    u16,
    4,
    65536,
    PixelType::U16x4,
    "Four `u16` components per pixel (e.g. RGBA16)"
);
pixel_struct!(
    I32,
    i32,
    i32,
    1,
    0,
    PixelType::I32,
    "One `i32` component per pixel"
);
pixel_struct!(
    F32,
    f32,
    f32,
    1,
    0,
    PixelType::F32,
    "One `f32` component per pixel"
);

pub trait IntoPixelComponent<Out: PixelComponent>
where
    Self: PixelComponent,
{
    fn into_component(self) -> Out;
}

impl<C: PixelComponent> IntoPixelComponent<C> for C {
    fn into_component(self) -> C {
        self
    }
}

impl IntoPixelComponent<u8> for u16 {
    fn into_component(self) -> u8 {
        self.to_le_bytes()[1]
    }
}

impl IntoPixelComponent<u16> for u8 {
    fn into_component(self) -> u16 {
        u16::from_le_bytes([self, self])
    }
}
