use crate::pixels::U8x4;
use crate::{CpuExtensions, ImageError, ImageView, ImageViewMut};

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

impl AlphaMulDiv for U8x4 {
    fn multiply_alpha(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        cpu_extensions: CpuExtensions,
    ) -> Result<(), ImageError> {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => unsafe { avx2::multiply_alpha(src_view, dst_view) },
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => unsafe { sse4::multiply_alpha(src_view, dst_view) },
            #[cfg(target_arch = "aarch64")]
            CpuExtensions::Neon => unsafe { neon::multiply_alpha(src_image, dst_image) },
            #[cfg(target_arch = "wasm32")]
            CpuExtensions::Simd128 => unsafe { wasm32::multiply_alpha(src_image, dst_image) },
            _ => native::multiply_alpha(src_view, dst_view),
        }
        Ok(())
    }

    fn multiply_alpha_inplace(
        image_view: &mut impl ImageViewMut<Pixel = Self>,
        cpu_extensions: CpuExtensions,
    ) -> Result<(), ImageError> {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => unsafe { avx2::multiply_alpha_inplace(image_view) },
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => unsafe { sse4::multiply_alpha_inplace(image_view) },
            #[cfg(target_arch = "aarch64")]
            CpuExtensions::Neon => unsafe { neon::multiply_alpha_inplace(image) },
            #[cfg(target_arch = "wasm32")]
            CpuExtensions::Simd128 => unsafe { wasm32::multiply_alpha_inplace(image) },
            _ => native::multiply_alpha_inplace(image_view),
        }
        Ok(())
    }

    fn divide_alpha(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        cpu_extensions: CpuExtensions,
    ) -> Result<(), ImageError> {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => unsafe { avx2::divide_alpha(src_view, dst_view) },
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => unsafe { sse4::divide_alpha(src_view, dst_view) },
            #[cfg(target_arch = "aarch64")]
            CpuExtensions::Neon => unsafe { neon::divide_alpha(src_view, dst_view) },
            #[cfg(target_arch = "wasm32")]
            CpuExtensions::Simd128 => unsafe { wasm32::divide_alpha(src_view, dst_view) },
            _ => native::divide_alpha(src_view, dst_view),
        }
        Ok(())
    }

    fn divide_alpha_inplace(
        image_view: &mut impl ImageViewMut<Pixel = Self>,
        cpu_extensions: CpuExtensions,
    ) -> Result<(), ImageError> {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => unsafe { avx2::divide_alpha_inplace(image_view) },
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => unsafe { sse4::divide_alpha_inplace(image_view) },
            #[cfg(target_arch = "aarch64")]
            CpuExtensions::Neon => unsafe { neon::divide_alpha_inplace(image_view) },
            #[cfg(target_arch = "wasm32")]
            CpuExtensions::Simd128 => unsafe { wasm32::divide_alpha_inplace(image_view) },
            _ => native::divide_alpha_inplace(image_view),
        }
        Ok(())
    }
}
