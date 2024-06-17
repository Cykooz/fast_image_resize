use crate::convolution::vertical_f32::vert_convolution_f32;
use crate::cpu_extensions::CpuExtensions;
use crate::pixels::F32x2;
use crate::{ImageView, ImageViewMut};

use super::{Coefficients, Convolution};

#[cfg(target_arch = "x86_64")]
mod avx2;
mod native;
// #[cfg(target_arch = "aarch64")]
// mod neon;
#[cfg(target_arch = "x86_64")]
mod sse4;
// #[cfg(target_arch = "wasm32")]
// mod wasm32;

impl Convolution for F32x2 {
    fn horiz_convolution(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        offset: u32,
        coeffs: Coefficients,
        cpu_extensions: CpuExtensions,
    ) {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => avx2::horiz_convolution(src_view, dst_view, offset, coeffs),
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => sse4::horiz_convolution(src_view, dst_view, offset, coeffs),
            #[cfg(target_arch = "aarch64")]
            CpuExtensions::Neon => neon::horiz_convolution(src_view, dst_view, offset, coeffs),
            #[cfg(target_arch = "wasm32")]
            CpuExtensions::Simd128 => wasm32::horiz_convolution(src_view, dst_view, offset, coeffs),
            _ => native::horiz_convolution(src_view, dst_view, offset, coeffs),
        }
    }

    fn vert_convolution(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        offset: u32,
        coeffs: Coefficients,
        cpu_extensions: CpuExtensions,
    ) {
        vert_convolution_f32(src_view, dst_view, offset, coeffs, cpu_extensions);
    }
}
