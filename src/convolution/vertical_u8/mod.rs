use crate::convolution::Coefficients;
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::Pixel;
use crate::CpuExtensions;

#[cfg(target_arch = "x86_64")]
pub(crate) mod avx2;
pub(crate) mod native;
#[cfg(target_arch = "x86_64")]
pub(crate) mod sse4;

pub(crate) fn vert_convolution_u8<T: Pixel<Component = u8>>(
    src_image: TypedImageView<T>,
    dst_image: TypedImageViewMut<T>,
    coeffs: Coefficients,
    cpu_extensions: CpuExtensions,
) {
    match cpu_extensions {
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Avx2 => avx2::vert_convolution(src_image, dst_image, coeffs),
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Sse4_1 => sse4::vert_convolution(src_image, dst_image, coeffs),
        _ => native::vert_convolution(src_image, dst_image, coeffs),
    }
}
