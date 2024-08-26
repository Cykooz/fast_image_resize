use crate::convolution::optimisations::{CoefficientsI16Chunk, Normalizer16};
use crate::image_view::{ImageView, ImageViewMut};
use crate::pixels::InnerPixel;
use crate::utils::foreach_with_pre_reading;

#[inline(always)]
pub(crate) fn vert_convolution<T>(
    src_image: &impl ImageView<Pixel = T>,
    dst_image: &mut impl ImageViewMut<Pixel = T>,
    offset: u32,
    normalizer: &Normalizer16,
) where
    T: InnerPixel<Component = u8>,
{
    let coefficients_chunks = normalizer.chunks();
    let precision = normalizer.precision();
    let initial = 1 << (precision - 1);
    let src_x_initial = offset as usize * T::count_of_components();

    let dst_rows = dst_image.iter_rows_mut(0);
    let coeffs_chunks_iter = coefficients_chunks.iter();
    let coefs_and_dst_row = coeffs_chunks_iter.zip(dst_rows);

    for (coeffs_chunk, dst_row) in coefs_and_dst_row {
        scale_row(
            src_image,
            dst_row,
            initial,
            src_x_initial,
            coeffs_chunk,
            normalizer,
        );
    }
}

fn scale_row<T>(
    src_image: &impl ImageView<Pixel = T>,
    dst_row: &mut [T],
    initial: i32,
    src_x_initial: usize,
    coeffs_chunk: &CoefficientsI16Chunk,
    normalizer: &Normalizer16,
) where
    T: InnerPixel<Component = u8>,
{
    let first_y_src = coeffs_chunk.start;
    let ks = coeffs_chunk.values();
    let mut x_src = src_x_initial;
    let dst_components = T::components_mut(dst_row);

    let (_, dst_chunks, tail) = unsafe { dst_components.align_to_mut::<[u8; 16]>() };
    x_src = convolution_by_chunks(
        src_image,
        normalizer,
        initial,
        dst_chunks,
        x_src,
        first_y_src,
        ks,
    );
    if tail.is_empty() {
        return;
    }

    let (_, dst_chunks, tail) = unsafe { tail.align_to_mut::<[u8; 8]>() };
    x_src = convolution_by_chunks(
        src_image,
        normalizer,
        initial,
        dst_chunks,
        x_src,
        first_y_src,
        ks,
    );
    if tail.is_empty() {
        return;
    }

    let (_, dst_chunks, tail) = unsafe { tail.align_to_mut::<[u8; 4]>() };
    x_src = convolution_by_chunks(
        src_image,
        normalizer,
        initial,
        dst_chunks,
        x_src,
        first_y_src,
        ks,
    );

    if !tail.is_empty() {
        convolution_by_u8(src_image, normalizer, initial, tail, x_src, first_y_src, ks);
    }
}

#[inline(always)]
pub(crate) fn convolution_by_u8<T>(
    src_image: &impl ImageView<Pixel = T>,
    normalizer: &Normalizer16,
    initial: i32,
    dst_components: &mut [u8],
    mut x_src: usize,
    first_y_src: u32,
    ks: &[i16],
) -> usize
where
    T: InnerPixel<Component = u8>,
{
    for dst_component in dst_components {
        let mut ss = initial;
        let src_rows = src_image.iter_rows(first_y_src);
        for (&k, src_row) in ks.iter().zip(src_rows) {
            let src_ptr = src_row.as_ptr() as *const u8;
            let src_component = unsafe { *src_ptr.add(x_src) };
            ss += src_component as i32 * (k as i32);
        }
        *dst_component = unsafe { normalizer.clip(ss) };
        x_src += 1
    }
    x_src
}

#[inline(always)]
fn convolution_by_chunks<T, const CHUNK_SIZE: usize>(
    src_image: &impl ImageView<Pixel = T>,
    normalizer: &Normalizer16,
    initial: i32,
    dst_chunks: &mut [[u8; CHUNK_SIZE]],
    mut x_src: usize,
    first_y_src: u32,
    ks: &[i16],
) -> usize
where
    T: InnerPixel<Component = u8>,
{
    for dst_chunk in dst_chunks {
        let mut ss = [initial; CHUNK_SIZE];
        let src_rows = src_image.iter_rows(first_y_src);

        foreach_with_pre_reading(
            ks.iter().zip(src_rows),
            |(&k, src_row)| {
                let src_ptr = src_row.as_ptr() as *const u8;
                let src_chunk = unsafe {
                    let ptr = src_ptr.add(x_src) as *const [u8; CHUNK_SIZE];
                    ptr.read_unaligned()
                };
                (src_chunk, k)
            },
            |(src_chunk, k)| {
                for (s, c) in ss.iter_mut().zip(src_chunk) {
                    *s += c as i32 * (k as i32);
                }
            },
        );

        for (i, s) in ss.iter().copied().enumerate() {
            dst_chunk[i] = unsafe { normalizer.clip(s) };
        }
        x_src += CHUNK_SIZE;
    }
    x_src
}
