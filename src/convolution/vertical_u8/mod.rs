use crate::convolution::Coefficients;
use crate::pixels::PixelExt;
use crate::CpuExtensions;
use crate::{ImageView, ImageViewMut};

#[cfg(target_arch = "x86_64")]
pub(crate) mod avx2;
pub(crate) mod native;
#[cfg(target_arch = "aarch64")]
mod neon;
#[cfg(target_arch = "x86_64")]
pub(crate) mod sse4;
#[cfg(target_arch = "wasm32")]
pub(crate) mod wasm32;

pub(crate) fn vert_convolution_u8<T: PixelExt<Component = u8>>(
    src_image: &ImageView<T>,
    dst_image: &mut ImageViewMut<T>,
    offset: u32,
    coeffs: Coefficients,
    cpu_extensions: CpuExtensions,
) {
    // Check safety conditions
    debug_assert!(src_image.width().get() - offset >= dst_image.width().get());
    debug_assert_eq!(coeffs.bounds.len(), dst_image.height().get() as usize);

    match cpu_extensions {
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Avx2 => avx2::vert_convolution(src_image, dst_image, offset, coeffs),
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Sse4_1 => sse4::vert_convolution(src_image, dst_image, offset, coeffs),
        #[cfg(target_arch = "aarch64")]
        CpuExtensions::Neon => neon::vert_convolution(src_image, dst_image, offset, coeffs),
        #[cfg(target_arch = "wasm32")]
        CpuExtensions::Wasm32 => wasm32::vert_convolution(src_image, dst_image, offset, coeffs),
        _ => native::vert_convolution(src_image, dst_image, offset, coeffs),
    }
}
