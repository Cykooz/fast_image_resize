use crate::convolution::optimisations::Normalizer32;
use crate::pixels::InnerPixel;
use crate::{CpuExtensions, ImageView, ImageViewMut};

#[cfg(target_arch = "x86_64")]
pub(crate) mod avx2;
pub(crate) mod native;
#[cfg(target_arch = "aarch64")]
mod neon;
#[cfg(target_arch = "x86_64")]
pub(crate) mod sse4;
#[cfg(target_arch = "wasm32")]
pub mod wasm32;

pub(crate) fn vert_convolution_u16<T: InnerPixel<Component = u16>>(
    src_view: &impl ImageView<Pixel = T>,
    dst_view: &mut impl ImageViewMut<Pixel = T>,
    offset: u32,
    normalizer: &Normalizer32,
    cpu_extensions: CpuExtensions,
) {
    // Check safety conditions
    debug_assert!(src_view.width() - offset >= dst_view.width());
    debug_assert_eq!(normalizer.chunks_len(), dst_view.height() as usize);

    match cpu_extensions {
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Avx2 => avx2::vert_convolution(src_view, dst_view, offset, normalizer),
        #[cfg(target_arch = "x86_64")]
        CpuExtensions::Sse4_1 => sse4::vert_convolution(src_view, dst_view, offset, normalizer),
        #[cfg(target_arch = "aarch64")]
        CpuExtensions::Neon => neon::vert_convolution(src_view, dst_view, offset, normalizer),
        #[cfg(target_arch = "wasm32")]
        CpuExtensions::Simd128 => wasm32::vert_convolution(src_view, dst_view, offset, normalizer),
        _ => native::vert_convolution(src_view, dst_view, offset, normalizer),
    }
}
