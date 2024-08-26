use super::{Coefficients, Convolution};
use crate::pixels::I32;
use crate::{CpuExtensions, ImageView, ImageViewMut};

mod native;

type P = I32;

impl Convolution for P {
    fn horiz_convolution(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        offset: u32,
        coeffs: Coefficients,
        _cpu_extensions: CpuExtensions,
    ) {
        debug_assert!(src_view.height() - offset >= dst_view.height());
        let coeffs_ref = &coeffs;

        try_process_in_threads_h! {
            horiz_convolution(
                src_view,
                dst_view,
                offset,
                coeffs_ref,
            );
        }
    }

    fn vert_convolution(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        offset: u32,
        coeffs: Coefficients,
        _cpu_extensions: CpuExtensions,
    ) {
        debug_assert!(src_view.width() - offset >= dst_view.width());

        let coeffs_ref = &coeffs;

        try_process_in_threads_v! {
            vert_convolution(
                src_view,
                dst_view,
                offset,
                coeffs_ref,
            );
        }
    }
}

#[inline(always)]
fn horiz_convolution(
    src_view: &impl ImageView<Pixel = P>,
    dst_view: &mut impl ImageViewMut<Pixel = P>,
    offset: u32,
    coefficients: &Coefficients,
) {
    native::horiz_convolution(src_view, dst_view, offset, coefficients);
}

#[inline(always)]
fn vert_convolution(
    src_view: &impl ImageView<Pixel = P>,
    dst_view: &mut impl ImageViewMut<Pixel = P>,
    offset: u32,
    coefficients: &Coefficients,
) {
    native::vert_convolution(src_view, dst_view, offset, coefficients);
}
