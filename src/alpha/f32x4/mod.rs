use crate::cpu_extensions::CpuExtensions;
use crate::pixels::F32x4;
use crate::{ImageError, ImageView, ImageViewMut};

use super::AlphaMulDiv;

#[cfg(target_arch = "x86_64")]
mod avx2;
mod native;
#[cfg(target_arch = "x86_64")]
mod sse4;

type P = F32x4;

impl AlphaMulDiv for P {
    fn multiply_alpha(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        cpu_extensions: CpuExtensions,
    ) -> Result<(), ImageError> {
        process_two_images! {
            multiple(src_view, dst_view, cpu_extensions);
        }
        Ok(())
    }

    fn multiply_alpha_inplace(
        image_view: &mut impl ImageViewMut<Pixel = Self>,
        cpu_extensions: CpuExtensions,
    ) -> Result<(), ImageError> {
        process_one_images! {
            multiply_inplace(image_view, cpu_extensions);
        }
        Ok(())
    }

    fn divide_alpha(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        cpu_extensions: CpuExtensions,
    ) -> Result<(), ImageError> {
        process_two_images! {
            divide(src_view, dst_view, cpu_extensions);
        }
        Ok(())
    }

    fn divide_alpha_inplace(
        image_view: &mut impl ImageViewMut<Pixel = Self>,
        cpu_extensions: CpuExtensions,
    ) -> Result<(), ImageError> {
        process_one_images! {
            divide_inplace(image_view, cpu_extensions);
        }
        Ok(())
    }
}

fn multiple(
    src_view: &impl ImageView<Pixel = P>,
    dst_view: &mut impl ImageViewMut<Pixel = P>,
    cpu_extensions: CpuExtensions,
) {
    match cpu_extensions {
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Avx2 => unsafe { avx2::multiply_alpha(src_view, dst_view) },
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Sse4_1 => unsafe { sse4::multiply_alpha(src_view, dst_view) },
        // #[cfg(target_arch = "aarch64")]
        // CpuExtensions::Neon => unsafe { neon::multiply_alpha(src_view, dst_view) },
        // #[cfg(target_arch = "wasm32")]
        // CpuExtensions::Simd128 => unsafe { wasm32::multiply_alpha(src_view, dst_view) },
        _ => native::multiply_alpha(src_view, dst_view),
    }
}

fn multiply_inplace(image_view: &mut impl ImageViewMut<Pixel = P>, cpu_extensions: CpuExtensions) {
    match cpu_extensions {
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Avx2 => unsafe { avx2::multiply_alpha_inplace(image_view) },
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Sse4_1 => unsafe { sse4::multiply_alpha_inplace(image_view) },
        // #[cfg(target_arch = "aarch64")]
        // CpuExtensions::Neon => unsafe { neon::multiply_alpha_inplace(image_view) },
        // #[cfg(target_arch = "wasm32")]
        // CpuExtensions::Simd128 => unsafe { wasm32::multiply_alpha_inplace(image_view) },
        _ => native::multiply_alpha_inplace(image_view),
    }
}

fn divide(
    src_view: &impl ImageView<Pixel = P>,
    dst_view: &mut impl ImageViewMut<Pixel = P>,
    cpu_extensions: CpuExtensions,
) {
    match cpu_extensions {
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Avx2 => unsafe { avx2::divide_alpha(src_view, dst_view) },
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Sse4_1 => unsafe { sse4::divide_alpha(src_view, dst_view) },
        // #[cfg(target_arch = "aarch64")]
        // CpuExtensions::Neon => unsafe { neon::divide_alpha(src_view, dst_view) },
        // #[cfg(target_arch = "wasm32")]
        // CpuExtensions::Simd128 => unsafe { wasm32::divide_alpha(src_view, dst_view) },
        _ => native::divide_alpha(src_view, dst_view),
    }
}

fn divide_inplace(image_view: &mut impl ImageViewMut<Pixel = P>, cpu_extensions: CpuExtensions) {
    match cpu_extensions {
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Avx2 => unsafe { avx2::divide_alpha_inplace(image_view) },
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Sse4_1 => unsafe { sse4::divide_alpha_inplace(image_view) },
        // #[cfg(target_arch = "aarch64")]
        // CpuExtensions::Neon => unsafe { neon::divide_alpha_inplace(image_view) },
        // #[cfg(target_arch = "wasm32")]
        // CpuExtensions::Simd128 => unsafe { wasm32::divide_alpha_inplace(image_view) },
        _ => native::divide_alpha_inplace(image_view),
    }
}
