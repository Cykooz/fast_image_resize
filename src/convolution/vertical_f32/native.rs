use crate::convolution::Coefficients;
use crate::pixels::InnerPixel;
use crate::utils::foreach_with_pre_reading;
use crate::{ImageView, ImageViewMut};

#[inline(always)]
pub(crate) fn vert_convolution<T>(
    src_view: &impl ImageView<Pixel = T>,
    dst_view: &mut impl ImageViewMut<Pixel = T>,
    offset: u32,
    coeffs: &Coefficients,
) where
    T: InnerPixel<Component = f32>,
{
    let coefficients_chunks = coeffs.get_chunks();
    let src_x_initial = offset as usize * T::count_of_components();

    let dst_rows = dst_view.iter_rows_mut(0);
    let coeffs_chunks_iter = coefficients_chunks.into_iter();
    for (coeffs_chunk, dst_row) in coeffs_chunks_iter.zip(dst_rows) {
        let first_y_src = coeffs_chunk.start;
        let ks = coeffs_chunk.values;
        let mut dst_components = T::components_mut(dst_row);
        let mut x_src = src_x_initial;

        #[cfg(target_arch = "aarch64")]
        {
            (dst_components, x_src) =
                convolution_by_chunks::<_, 16>(src_view, dst_components, x_src, first_y_src, ks);
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            if !dst_components.is_empty() {
                (dst_components, x_src) =
                    convolution_by_chunks::<_, 8>(src_view, dst_components, x_src, first_y_src, ks);
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            if !dst_components.is_empty() {
                (dst_components, x_src) =
                    crate::convolution::vertical_f32::native::convolution_by_chunks::<_, 4>(
                        src_view,
                        dst_components,
                        x_src,
                        first_y_src,
                        ks,
                    );
            }
        }

        if !dst_components.is_empty() {
            convolution_by_f32(src_view, dst_components, x_src, first_y_src, ks);
        }
    }
}

#[inline(always)]
pub(crate) fn convolution_by_f32<T: InnerPixel<Component = f32>>(
    src_view: &impl ImageView<Pixel = T>,
    dst_components: &mut [f32],
    mut x_src: usize,
    first_y_src: u32,
    ks: &[f64],
) -> usize {
    for dst_component in dst_components.iter_mut() {
        let mut ss = 0.;
        let src_rows = src_view.iter_rows(first_y_src);
        for (&k, src_row) in ks.iter().zip(src_rows) {
            // SAFETY: Alignment of src_row is greater or equal than alignment f32
            //         because a component of pixel type T is f32.
            let src_ptr = src_row.as_ptr() as *const f32;
            let src_component = unsafe { *src_ptr.add(x_src) };
            ss += src_component as f64 * k;
        }
        *dst_component = ss as f32;
        x_src += 1
    }
    x_src
}

#[inline(always)]
fn convolution_by_chunks<'a, T, const CHUNK_SIZE: usize>(
    src_view: &impl ImageView<Pixel = T>,
    dst_components: &'a mut [f32],
    mut x_src: usize,
    first_y_src: u32,
    ks: &[f64],
) -> (&'a mut [f32], usize)
where
    T: InnerPixel<Component = f32>,
{
    let mut dst_chunks = dst_components.chunks_exact_mut(CHUNK_SIZE);

    for dst_chunk in &mut dst_chunks {
        let mut ss = [0.; CHUNK_SIZE];
        let src_rows = src_view.iter_rows(first_y_src);

        foreach_with_pre_reading(
            ks.iter().zip(src_rows),
            |(&k, src_row)| {
                let src_ptr = src_row.as_ptr() as *const f32;
                let src_chunk = unsafe {
                    let ptr = src_ptr.add(x_src) as *const [f32; CHUNK_SIZE];
                    ptr.read_unaligned()
                };
                (src_chunk, k)
            },
            |(src_chunk, k)| {
                for (s, c) in ss.iter_mut().zip(src_chunk) {
                    *s += c as f64 * k;
                }
            },
        );

        for (i, s) in ss.iter().copied().enumerate() {
            dst_chunk[i] = s as f32;
        }
        x_src += CHUNK_SIZE;
    }

    (dst_chunks.into_remainder(), x_src)
}
