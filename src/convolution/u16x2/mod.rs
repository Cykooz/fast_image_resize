use super::{Coefficients, Convolution};
use crate::convolution::optimisations::Normalizer32;
use crate::convolution::vertical_u16::vert_convolution_u16;
use crate::pixels::U16x2;
use crate::{CpuExtensions, ImageView, ImageViewMut};

#[cfg(target_arch = "x86_64")]
mod avx2;
mod native;
#[cfg(target_arch = "aarch64")]
mod neon;
#[cfg(target_arch = "x86_64")]
mod sse4;
#[cfg(target_arch = "wasm32")]
mod wasm32;

type P = U16x2;

impl Convolution for P {
    fn horiz_convolution(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        offset: u32,
        coeffs: Coefficients,
        cpu_extensions: CpuExtensions,
    ) {
        debug_assert!(src_view.height() - offset >= dst_view.height());

        let normalizer = Normalizer32::new(coeffs);
        let normalizer_ref = &normalizer;

        try_process_in_threads_h! {
            horiz_convolution(
                src_view,
                dst_view,
                offset,
                normalizer_ref,
                cpu_extensions,
            );
        }
    }

    fn vert_convolution(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        offset: u32,
        coeffs: Coefficients,
        cpu_extensions: CpuExtensions,
    ) {
        debug_assert!(src_view.width() - offset >= dst_view.width());

        let normalizer = Normalizer32::new(coeffs);
        let normalizer_ref = &normalizer;

        try_process_in_threads_v! {
            vert_convolution_u16(
                src_view,
                dst_view,
                offset,
                normalizer_ref,
                cpu_extensions,
            );
        }
    }
}

fn horiz_convolution(
    src_view: &impl ImageView<Pixel = P>,
    dst_view: &mut impl ImageViewMut<Pixel = P>,
    offset: u32,
    normalizer: &Normalizer32,
    cpu_extensions: CpuExtensions,
) {
    match cpu_extensions {
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Avx2 => avx2::horiz_convolution(src_view, dst_view, offset, normalizer),
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Sse4_1 => sse4::horiz_convolution(src_view, dst_view, offset, normalizer),
        #[cfg(target_arch = "aarch64")]
        CpuExtensions::Neon => neon::horiz_convolution(src_view, dst_view, offset, normalizer),
        #[cfg(target_arch = "wasm32")]
        CpuExtensions::Simd128 => wasm32::horiz_convolution(src_view, dst_view, offset, normalizer),
        _ => native::horiz_convolution(src_view, dst_view, offset, normalizer),
    }
}
