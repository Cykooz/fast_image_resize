use std::mem::size_of;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PixelType {
    U8x4,
    I32,
    F32,
    U8,
}

impl PixelType {
    pub(crate) fn size(&self) -> usize {
        match self {
            Self::U8 => 1,
            _ => 4,
        }
    }

    pub(crate) fn is_aligned(&self, buffer: &[u8]) -> bool {
        match self {
            Self::U8x4 => unsafe { buffer.align_to::<u32>().0.is_empty() },
            Self::I32 => unsafe { buffer.align_to::<i32>().0.is_empty() },
            Self::F32 => unsafe { buffer.align_to::<f32>().0.is_empty() },
            Self::U8 => true,
        }
    }
}

pub(crate) trait Pixel {
    type Type: Copy;

    fn size() -> usize {
        size_of::<Self::Type>()
    }

    fn pixel_type() -> PixelType;
}

macro_rules! pixel_struct {
    ($name:ident, $type:tt, $pixel_type:expr) => {
        pub struct $name;

        impl Pixel for $name {
            type Type = $type;

            fn pixel_type() -> PixelType {
                $pixel_type
            }
        }
    };
}

pixel_struct!(U8x4, u32, PixelType::U8x4);
pixel_struct!(I32, i32, PixelType::I32);
pixel_struct!(F32, f32, PixelType::F32);
pixel_struct!(U8, u8, PixelType::U8);
