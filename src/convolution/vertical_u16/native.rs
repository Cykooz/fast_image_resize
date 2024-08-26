use crate::convolution::optimisations::Normalizer32;
use crate::pixels::InnerPixel;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

#[inline(always)]
pub(crate) fn vert_convolution<T>(
    src_view: &impl ImageView<Pixel = T>,
    dst_view: &mut impl ImageViewMut<Pixel = T>,
    offset: u32,
    normalizer: &Normalizer32,
) where
    T: InnerPixel<Component = u16>,
{
    let coefficients_chunks = normalizer.chunks();
    let precision = normalizer.precision();
    let initial: i64 = 1 << (precision - 1);
    let src_x_initial = offset as usize * T::count_of_components();

    let dst_rows = dst_view.iter_rows_mut(0);
    let coeffs_chunks_iter = coefficients_chunks.iter();
    for (coeffs_chunk, dst_row) in coeffs_chunks_iter.zip(dst_rows) {
        let first_y_src = coeffs_chunk.start;
        let ks = coeffs_chunk.values();
        let dst_components = T::components_mut(dst_row);
        let mut x_src = src_x_initial;

        let (_, dst_chunks, tail) = unsafe { dst_components.align_to_mut::<[u16; 16]>() };
        x_src = convolution_by_chunks(
            src_view,
            normalizer,
            initial,
            dst_chunks,
            x_src,
            first_y_src,
            ks,
        );

        if !tail.is_empty() {
            convolution_by_u16(src_view, normalizer, initial, tail, x_src, first_y_src, ks);
        }
    }
}

#[inline(always)]
pub(crate) fn convolution_by_u16<T: InnerPixel<Component = u16>>(
    src_view: &impl ImageView<Pixel = T>,
    normalizer: &Normalizer32,
    initial: i64,
    dst_components: &mut [u16],
    mut x_src: usize,
    first_y_src: u32,
    ks: &[i32],
) -> usize {
    for dst_component in dst_components.iter_mut() {
        let mut ss = initial;
        let src_rows = src_view.iter_rows(first_y_src);
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

#[inline(always)]
fn convolution_by_chunks<T, const CHUNK_SIZE: usize>(
    src_view: &impl ImageView<Pixel = T>,
    normalizer: &Normalizer32,
    initial: i64,
    dst_chunks: &mut [[u16; CHUNK_SIZE]],
    mut x_src: usize,
    first_y_src: u32,
    ks: &[i32],
) -> usize
where
    T: InnerPixel<Component = u16>,
{
    for dst_chunk in dst_chunks {
        let mut ss = [initial; CHUNK_SIZE];
        let src_rows = src_view.iter_rows(first_y_src);

        foreach_with_pre_reading(
            ks.iter().zip(src_rows),
            |(&k, src_row)| {
                let src_ptr = src_row.as_ptr() as *const u16;
                let src_chunk = unsafe {
                    let ptr = src_ptr.add(x_src) as *const [u16; CHUNK_SIZE];
                    ptr.read_unaligned()
                };
                (src_chunk, k)
            },
            |(src_chunk, k)| {
                for (s, c) in ss.iter_mut().zip(src_chunk) {
                    *s += c as i64 * (k as i64);
                }
            },
        );

        for (i, s) in ss.iter().copied().enumerate() {
            dst_chunk[i] = normalizer.clip(s);
        }
        x_src += CHUNK_SIZE;
    }
    x_src
}
