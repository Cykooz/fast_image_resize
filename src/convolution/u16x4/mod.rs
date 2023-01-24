use crate::convolution::vertical_u16::vert_convolution_u16;
use crate::pixels::U16x4;
use crate::CpuExtensions;
use crate::{ImageView, ImageViewMut};

use super::{Coefficients, Convolution};

#[cfg(target_arch = "x86_64")]
mod avx2;
mod native;
#[cfg(target_arch = "aarch64")]
mod neon;
#[cfg(target_arch = "x86_64")]
mod sse4;
#[cfg(target_arch = "wasm32")]
mod wasm32;

impl Convolution for U16x4 {
    fn horiz_convolution(
        src_image: &ImageView<Self>,
        dst_image: &mut ImageViewMut<Self>,
        offset: u32,
        coeffs: Coefficients,
        cpu_extensions: CpuExtensions,
    ) {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => avx2::horiz_convolution(src_image, dst_image, offset, coeffs),
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => sse4::horiz_convolution(src_image, dst_image, offset, coeffs),
            #[cfg(target_arch = "aarch64")]
            CpuExtensions::Neon => neon::horiz_convolution(src_image, dst_image, offset, coeffs),
            #[cfg(target_arch = "wasm32")]
            CpuExtensions::Wasm32 => {
                wasm32::horiz_convolution(src_image, dst_image, offset, coeffs)
            }
            _ => native::horiz_convolution(src_image, dst_image, offset, coeffs),
        }
    }

    fn vert_convolution(
        src_image: &ImageView<Self>,
        dst_image: &mut ImageViewMut<Self>,
        offset: u32,
        coeffs: Coefficients,
        cpu_extensions: CpuExtensions,
    ) {
        vert_convolution_u16(src_image, dst_image, offset, coeffs, cpu_extensions);
    }
}
