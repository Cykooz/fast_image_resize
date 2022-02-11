use super::{Coefficients, Convolution};
use crate::convolution::vertical_u8::vert_convolution_u8;
use crate::image_view::{TypedImageView, TypedImageViewMut};
use crate::pixels::U8x4;
use crate::CpuExtensions;

#[cfg(target_arch = "x86_64")]
mod avx2;
mod native;
#[cfg(target_arch = "x86_64")]
mod sse4;

impl Convolution for U8x4 {
    fn horiz_convolution(
        src_image: TypedImageView<Self>,
        dst_image: TypedImageViewMut<Self>,
        offset: u32,
        coeffs: Coefficients,
        cpu_extensions: CpuExtensions,
    ) {
        match cpu_extensions {
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Avx2 => avx2::horiz_convolution(src_image, dst_image, offset, coeffs),
            #[cfg(target_arch = "x86_64")]
            CpuExtensions::Sse4_1 => sse4::horiz_convolution(src_image, dst_image, offset, coeffs),
            _ => native::horiz_convolution(src_image, dst_image, offset, coeffs),
        }
    }

    fn vert_convolution(
        src_image: TypedImageView<Self>,
        dst_image: TypedImageViewMut<Self>,
        coeffs: Coefficients,
        cpu_extensions: CpuExtensions,
    ) {
        vert_convolution_u8(src_image, dst_image, coeffs, cpu_extensions);
    }
}
