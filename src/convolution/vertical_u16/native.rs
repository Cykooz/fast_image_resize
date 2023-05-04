use crate::convolution::{optimisations, Coefficients};
use crate::pixels::PixelExt;
use crate::{ImageView, ImageViewMut};

#[inline(always)]
pub(crate) fn vert_convolution<T: PixelExt<Component = u16>>(
    src_image: &ImageView<T>,
    dst_image: &mut ImageViewMut<T>,
    offset: u32,
    coeffs: Coefficients,
) {
    let normalizer = optimisations::Normalizer32::new(coeffs);
    let coefficients_chunks = normalizer.normalized_chunks();
    let precision = normalizer.precision();
    let initial: i64 = 1 << (precision - 1);
    let x_src = offset as usize * T::count_of_components();

    let dst_rows = dst_image.iter_rows_mut();
    let coeffs_chunks_iter = coefficients_chunks.into_iter();
    for (coeffs_chunk, dst_row) in coeffs_chunks_iter.zip(dst_rows) {
        let first_y_src = coeffs_chunk.start;
        let ks = coeffs_chunk.values;
        let dst_components = T::components_mut(dst_row);

        convolution_by_u16(
            src_image,
            &normalizer,
            initial,
            dst_components,
            x_src,
            first_y_src,
            ks,
        );
    }
}

#[inline(always)]
pub(crate) fn convolution_by_u16<T: PixelExt<Component = u16>>(
    src_image: &ImageView<T>,
    normalizer: &optimisations::Normalizer32,
    initial: i64,
    dst_components: &mut [u16],
    mut x_src: usize,
    first_y_src: u32,
    ks: &[i32],
) -> usize {
    for dst_component in dst_components.iter_mut() {
        let mut ss = initial;
        let src_rows = src_image.iter_rows(first_y_src);
        for (&k, src_row) in ks.iter().zip(src_rows) {
            // SAFETY: Alignment of src_row is greater or equal than alignment u16
            //         because one component of pixel type T is u16.
            let src_ptr = src_row.as_ptr() as *const u16;
            let src_component = unsafe { *src_ptr.add(x_src) };
            ss += src_component as i64 * (k as i64);
        }
        *dst_component = normalizer.clip(ss);
        x_src += 1
    }
    x_src
}
