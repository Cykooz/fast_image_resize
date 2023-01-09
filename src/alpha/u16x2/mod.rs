use crate::pixels::U16x2;
use crate::CpuExtensions;
use crate::{ImageView, ImageViewMut};

use super::AlphaMulDiv;

#[cfg(target_arch = "x86_64")]
mod avx2;
mod native;
#[cfg(target_arch = "aarch64")]
mod neon;
#[cfg(target_arch = "x86_64")]
mod sse4;
#[cfg(target_arch = "wasm32")]
mod wasm32;

impl AlphaMulDiv for U16x2 {
    fn multiply_alpha(
        src_image: &ImageView<Self>,
        dst_image: &mut ImageViewMut<Self>,
        cpu_extensions: CpuExtensions,
    ) {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => unsafe { avx2::multiply_alpha(src_image, dst_image) },
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => unsafe { sse4::multiply_alpha(src_image, dst_image) },
            #[cfg(target_arch = "aarch64")]
            CpuExtensions::Neon => unsafe { neon::multiply_alpha(src_image, dst_image) },
            #[cfg(target_arch = "wasm32")]
            CpuExtensions::Wasm32 => unsafe { wasm32::multiply_alpha(src_image, dst_image) },
            _ => native::multiply_alpha(src_image, dst_image),
        }
    }

    fn multiply_alpha_inplace(image: &mut ImageViewMut<Self>, cpu_extensions: CpuExtensions) {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => unsafe { avx2::multiply_alpha_inplace(image) },
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => unsafe { sse4::multiply_alpha_inplace(image) },
            #[cfg(target_arch = "aarch64")]
            CpuExtensions::Neon => unsafe { neon::multiply_alpha_inplace(image) },
            #[cfg(target_arch = "wasm32")]
            CpuExtensions::Wasm32 => unsafe { wasm32::multiply_alpha_inplace(image) },
            _ => native::multiply_alpha_inplace(image),
        }
    }

    fn divide_alpha(
        src_image: &ImageView<Self>,
        dst_image: &mut ImageViewMut<Self>,
        cpu_extensions: CpuExtensions,
    ) {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => unsafe { avx2::divide_alpha(src_image, dst_image) },
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => unsafe { sse4::divide_alpha(src_image, dst_image) },
            #[cfg(target_arch = "aarch64")]
            CpuExtensions::Neon => unsafe { neon::divide_alpha(src_image, dst_image) },
            #[cfg(target_arch = "wasm32")]
            CpuExtensions::Wasm32 => unsafe { wasm32::divide_alpha(src_image, dst_image) },
            _ => native::divide_alpha(src_image, dst_image),
        }
    }

    fn divide_alpha_inplace(image: &mut ImageViewMut<Self>, cpu_extensions: CpuExtensions) {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => unsafe { avx2::divide_alpha_inplace(image) },
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => unsafe { sse4::divide_alpha_inplace(image) },
            #[cfg(target_arch = "aarch64")]
            CpuExtensions::Neon => unsafe { neon::divide_alpha_inplace(image) },
            #[cfg(target_arch = "wasm32")]
            CpuExtensions::Wasm32 => unsafe { wasm32::divide_alpha_inplace(image) },
            _ => native::divide_alpha_inplace(image),
        }
    }
}
