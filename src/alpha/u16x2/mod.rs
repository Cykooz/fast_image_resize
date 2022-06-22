use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U16x2;
use crate::CpuExtensions;

use super::AlphaMulDiv;

#[cfg(target_arch = "x86_64")]
mod avx2;
mod native;
#[cfg(target_arch = "x86_64")]
mod sse4;

impl AlphaMulDiv for U16x2 {
    fn multiply_alpha(
        src_image: TypedImageView<Self>,
        dst_image: TypedImageViewMut<Self>,
        cpu_extensions: CpuExtensions,
    ) {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => unsafe { avx2::multiply_alpha(src_image, dst_image) },
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => unsafe { sse4::multiply_alpha(src_image, dst_image) },
            _ => native::multiply_alpha(src_image, dst_image),
        }
    }

    fn multiply_alpha_inplace(image: TypedImageViewMut<Self>, cpu_extensions: CpuExtensions) {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => unsafe { avx2::multiply_alpha_inplace(image) },
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => unsafe { sse4::multiply_alpha_inplace(image) },
            _ => native::multiply_alpha_inplace(image),
        }
    }

    fn divide_alpha(
        src_image: TypedImageView<Self>,
        dst_image: TypedImageViewMut<Self>,
        cpu_extensions: CpuExtensions,
    ) {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => unsafe { avx2::divide_alpha(src_image, dst_image) },
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => unsafe { sse4::divide_alpha(src_image, dst_image) },
            _ => native::divide_alpha(src_image, dst_image),
        }
    }

    fn divide_alpha_inplace(image: TypedImageViewMut<Self>, cpu_extensions: CpuExtensions) {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => unsafe { avx2::divide_alpha_inplace(image) },
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => unsafe { sse4::divide_alpha_inplace(image) },
            _ => native::divide_alpha_inplace(image),
        }
    }
}
