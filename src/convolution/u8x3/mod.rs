use super::{Coefficients, Convolution};
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x3;
use crate::CpuExtensions;

mod native;
mod sse4;

impl Convolution for U8x3 {
    fn horiz_convolution(
        src_image: TypedImageView<Self>,
        dst_image: TypedImageViewMut<Self>,
        offset: u32,
        coeffs: Coefficients,
        cpu_extensions: CpuExtensions,
    ) {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 | CpuExtensions::Sse4_1 => unsafe {
                sse4::horiz_convolution(src_image, dst_image, offset, coeffs)
            },
            _ => native::horiz_convolution(src_image, dst_image, offset, coeffs),
        }
    }

    fn vert_convolution(
        src_image: TypedImageView<Self>,
        dst_image: TypedImageViewMut<Self>,
        coeffs: Coefficients,
        cpu_extensions: CpuExtensions,
    ) {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 | CpuExtensions::Sse4_1 => unsafe {
                sse4::vert_convolution(src_image, dst_image, coeffs)
            },
            _ => native::vert_convolution(src_image, dst_image, coeffs),
        }
    }
}
