//! Contains types of pixels.
use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::size_of;
use std::slice;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PixelType {
    U8,
    U8x2,
    U8x3,
    U8x4,
    U16,
    U16x2,
    U16x3,
    U16x4,
    I32,
    F32,
    F32x2,
    F32x3,
    F32x4,
}

impl PixelType {
    /// Returns pixel size in bytes.
    pub fn size(&self) -> usize {
        match self {
            Self::U8 => 1,
            Self::U8x2 => 2,
            Self::U8x3 => 3,
            Self::U16 => 2,
            Self::U16x2 => 4,
            Self::U16x3 => 6,
            Self::U16x4 => 8,
            Self::F32x2 => 8,
            Self::F32x3 => 12,
            Self::F32x4 => 16,
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
            Self::F32x2 => unsafe { buffer.align_to::<F32x2>().0.is_empty() },
            Self::F32x3 => unsafe { buffer.align_to::<F32x3>().0.is_empty() },
            Self::F32x4 => unsafe { buffer.align_to::<F32x4>().0.is_empty() },
        }
    }
}

pub trait GetCount {
    fn count() -> usize;
}

/// Generic type to represent the number of components in single pixel.
pub struct Count<const N: usize>;

impl<const N: usize> GetCount for Count<N> {
    #[inline(always)]
    fn count() -> usize {
        N
    }
}

pub trait GetCountOfValues {
    fn count_of_values() -> usize;
}

/// Generic type to represent the number of available values for a single pixel component.
pub struct Values<const N: usize>;

impl<const N: usize> GetCountOfValues for Values<N> {
    fn count_of_values() -> usize {
        N
    }
}

/// Information about one component of pixel.
pub trait PixelComponent
where
    Self: Sized + Copy + Debug + PartialEq + 'static,
{
    /// Type that provides information about a count of
    /// available values of one pixel's component
    type CountOfComponentValues: GetCountOfValues;

    /// Count of available values of one pixel's component
    fn count_of_values() -> usize {
        Self::CountOfComponentValues::count_of_values()
    }
}

impl PixelComponent for u8 {
    type CountOfComponentValues = Values<0x100>;
}

impl PixelComponent for u16 {
    type CountOfComponentValues = Values<0x10000>;
}

impl PixelComponent for i32 {
    type CountOfComponentValues = Values<0>;
}

impl PixelComponent for f32 {
    type CountOfComponentValues = Values<0>;
}

// Prevent users from implementing the InnerPixel trait.
mod private {
    pub trait Sealed {}
}

/// Inner trait that provides additional information about pixel type.
///
/// Don't use this trait in your code. You must use the "child"
/// trait [PixelTrait](crate::PixelTrait) instead.
///
/// This trait is sealed and cannot be implemented for types outside this crate.
pub trait InnerPixel:
    private::Sealed + Copy + Clone + Sized + Debug + PartialEq + Default + Send + Sync + 'static
{
    /// Type of pixel components
    type Component: PixelComponent;
    /// Type that provides information about a count of pixel's components
    type CountOfComponents: GetCount;

    fn pixel_type() -> PixelType;

    /// Count of pixel's components
    fn count_of_components() -> usize {
        Self::CountOfComponents::count()
    }

    /// Count of available values of one pixel's component
    fn count_of_component_values() -> usize {
        Self::Component::count_of_values()
    }

    fn components_is_u8() -> bool {
        Self::count_of_component_values() == 256
    }

    /// Size of pixel in bytes
    ///
    /// Example:
    /// ```
    /// # use fast_image_resize::pixels::{U8x2, U8x3, U8, InnerPixel};
    /// assert_eq!(U8x3::size(), 3);
    /// assert_eq!(U8x2::size(), 2);
    /// assert_eq!(U8::size(), 1);
    /// ```
    fn size() -> usize {
        size_of::<Self>()
    }

    /// Create slice of pixel's components from slice of pixels
    fn components(buf: &[Self]) -> &[Self::Component] {
        let size = buf.len() * Self::count_of_components();
        let components_ptr = buf.as_ptr() as *const Self::Component;
        unsafe { slice::from_raw_parts(components_ptr, size) }
    }

    /// Create mutable slice of pixel's components from mutable slice of pixels
    fn components_mut(buf: &mut [Self]) -> &mut [Self::Component] {
        let size = buf.len() * Self::count_of_components();
        let components_ptr = buf.as_mut_ptr() as *mut Self::Component;
        unsafe { slice::from_raw_parts_mut(components_ptr, size) }
    }

    /// Returns empty pixel value
    fn empty() -> Self {
        Self::default()
    }
}

/// Generic type of pixel.
#[derive(Copy, Clone, PartialEq, Default)]
#[repr(C)]
pub struct Pixel<T: Default, C, const COUNT_OF_COMPONENTS: usize>(
    pub T,
    PhantomData<[C; COUNT_OF_COMPONENTS]>,
)
where
    T: Sized + Copy + Clone + PartialEq + 'static,
    C: PixelComponent;

impl<T, C, const COUNT_OF_COMPONENTS: usize> Pixel<T, C, COUNT_OF_COMPONENTS>
where
    T: Sized + Copy + Clone + PartialEq + Default + 'static,
    C: PixelComponent,
{
    #[inline(always)]
    pub const fn new(v: T) -> Self {
        Self(v, PhantomData)
    }
}

macro_rules! pixel_struct {
    ($name:ident, $type:tt, $comp_type:tt, $comp_count:literal, $pixel_type:expr, $doc:expr) => {
        #[doc = $doc]
        pub type $name = Pixel<$type, $comp_type, $comp_count>;

        impl private::Sealed for $name {}

        impl InnerPixel for $name {
            type Component = $comp_type;
            type CountOfComponents = Count<$comp_count>;

            fn pixel_type() -> PixelType {
                $pixel_type
            }
        }

        impl Debug for $name {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let components_ptr = self as *const _ as *const $comp_type;
                let components: &[$comp_type] =
                    unsafe { slice::from_raw_parts(components_ptr, $comp_count) };
                write!(f, "{}{:?}", stringify!($name), components)
            }
        }
    };
}

pixel_struct!(U8, u8, u8, 1, PixelType::U8, "One byte per pixel (e.g. L8)");
pixel_struct!(
    U8x2,
    [u8; 2],
    u8,
    2,
    PixelType::U8x2,
    "Two bytes per pixel (e.g. LA8)"
);
pixel_struct!(
    U8x3,
    [u8; 3],
    u8,
    3,
    PixelType::U8x3,
    "Three bytes per pixel (e.g. RGB8)"
);
pixel_struct!(
    U8x4,
    [u8; 4],
    u8,
    4,
    PixelType::U8x4,
    "Four bytes per pixel (RGBA8, RGBx8, CMYK8 and other)"
);
pixel_struct!(
    U16,
    u16,
    u16,
    1,
    PixelType::U16,
    "One `u16` component per pixel (e.g. L16)"
);
pixel_struct!(
    U16x2,
    [u16; 2],
    u16,
    2,
    PixelType::U16x2,
    "Two `u16` components per pixel (e.g. LA16)"
);
pixel_struct!(
    U16x3,
    [u16; 3],
    u16,
    3,
    PixelType::U16x3,
    "Three `u16` components per pixel (e.g. RGB16)"
);
pixel_struct!(
    U16x4,
    [u16; 4],
    u16,
    4,
    PixelType::U16x4,
    "Four `u16` components per pixel (e.g. RGBA16)"
);
pixel_struct!(
    I32,
    i32,
    i32,
    1,
    PixelType::I32,
    "One `i32` component per pixel"
);
pixel_struct!(
    F32,
    f32,
    f32,
    1,
    PixelType::F32,
    "One `f32` component per pixel"
);
pixel_struct!(
    F32x2,
    [f32; 2],
    f32,
    2,
    PixelType::F32x2,
    "Two `f32` component per pixel (e.g. LA32F)"
);
pixel_struct!(
    F32x3,
    [f32; 3],
    f32,
    3,
    PixelType::F32x3,
    "Three `f32` components per pixel (e.g. RGB32F)"
);
pixel_struct!(
    F32x4,
    [f32; 4],
    f32,
    4,
    PixelType::F32x4,
    "Four `f32` components per pixel (e.g. RGBA32F)"
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

// u8

impl IntoPixelComponent<u16> for u8 {
    fn into_component(self) -> u16 {
        u16::from_le_bytes([self, self])
    }
}

impl IntoPixelComponent<i32> for u8 {
    fn into_component(self) -> i32 {
        (self as i32) << 23
    }
}

impl IntoPixelComponent<f32> for u8 {
    fn into_component(self) -> f32 {
        (self as f32) / u8::MAX as f32
    }
}

// u16

impl IntoPixelComponent<u8> for u16 {
    fn into_component(self) -> u8 {
        self.to_le_bytes()[1]
    }
}

impl IntoPixelComponent<i32> for u16 {
    fn into_component(self) -> i32 {
        (self as i32) << 15
    }
}

impl IntoPixelComponent<f32> for u16 {
    fn into_component(self) -> f32 {
        (self as f32) / u16::MAX as f32
    }
}

// i32

impl IntoPixelComponent<u8> for i32 {
    fn into_component(self) -> u8 {
        (self.max(0).saturating_add(1 << 22) >> 23) as u8
    }
}

impl IntoPixelComponent<u16> for i32 {
    fn into_component(self) -> u16 {
        (self.max(0).saturating_add(1 << 14) >> 15) as u16
    }
}

impl IntoPixelComponent<f32> for i32 {
    fn into_component(self) -> f32 {
        if self < 0 {
            (self as f32) / i32::MIN as f32
        } else {
            (self as f32) / i32::MAX as f32
        }
    }
}

// f32

impl IntoPixelComponent<u8> for f32 {
    fn into_component(self) -> u8 {
        (self.clamp(0., 1.) * u8::MAX as f32).round() as u8
    }
}

impl IntoPixelComponent<u16> for f32 {
    fn into_component(self) -> u16 {
        (self.clamp(0., 1.) * u16::MAX as f32).round() as u16
    }
}

impl IntoPixelComponent<i32> for f32 {
    fn into_component(self) -> i32 {
        let max = if self < 0. {
            i32::MIN as f32
        } else {
            i32::MAX as f32
        };
        (self.clamp(-1., 1.) * max).round() as i32
    }
}
