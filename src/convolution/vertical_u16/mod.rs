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

pub(crate) fn vert_convolution_u16<T: PixelExt<Component = u16>>(
    src_image: &ImageView<T>,
    dst_image: &mut ImageViewMut<T>,
    coeffs: Coefficients,
    cpu_extensions: CpuExtensions,
) {
    match cpu_extensions {
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Avx2 => avx2::vert_convolution(src_image, dst_image, coeffs),
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Sse4_1 => sse4::vert_convolution(src_image, dst_image, coeffs),
        #[cfg(target_arch = "aarch64")]
        CpuExtensions::Neon => neon::vert_convolution(src_image, dst_image, coeffs),
        _ => native::vert_convolution(src_image, dst_image, coeffs),
    }
}
