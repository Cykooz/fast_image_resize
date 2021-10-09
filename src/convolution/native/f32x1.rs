use std::slice;

use crate::convolution::{Coefficients, Convolution};
use crate::{DstImageView, SrcImageView};

pub struct NativeF32;

impl Convolution for NativeF32 {
    fn horiz_convolution(
        &self,
        src_image: &SrcImageView,
        dst_image: &mut DstImageView,
        offset: u32,
        coeffs: Coefficients,
    ) {
        let (values, window_size, bounds) = (coeffs.values, coeffs.window_size, coeffs.bounds);

        for y_dst in 0..dst_image.height().get() {
            let y_src = y_dst + offset;
            if let Some(out_row) = dst_image.get_row_mut(y_dst) {
                let out_row_f32 = unsafe {
                    let len = out_row.len();
                    let ptr = out_row.as_mut_ptr();
                    slice::from_raw_parts_mut(ptr as *mut f32, len)
                };
                for (x_dst, (&bound, out_pixel)) in bounds.iter().zip(out_row_f32).enumerate() {
                    let first_x_src = bound.start;
                    let start_index = window_size * x_dst;
                    let end_index = start_index + bound.size as usize;

                    let ks = &values[start_index..end_index];

                    let mut ss = 0.;
                    let pixels = src_image.iter_horiz_f32(first_x_src, y_src);
                    for (&k, &pixel) in ks.iter().zip(pixels) {
                        ss += pixel as f64 * k;
                    }
                    *out_pixel = ss as f32;
                }
            }
        }
    }

    fn vert_convolution(
        &self,
        src_image: &SrcImageView,
        dst_image: &mut DstImageView,
        coeffs: Coefficients,
    ) {
        let (values, window_size, bounds) = (coeffs.values, coeffs.window_size, coeffs.bounds);

        for (y_dst, &bound) in bounds.iter().enumerate() {
            let first_y_src = bound.start;
            let start_index = window_size * y_dst;
            let end_index = start_index + bound.size as usize;
            let ks = &values[start_index..end_index];

            if let Some(out_row) = dst_image.get_row_mut(y_dst as u32) {
                let out_row_f32 = unsafe {
                    let len = out_row.len();
                    let ptr = out_row.as_mut_ptr();
                    slice::from_raw_parts_mut(ptr as *mut f32, len)
                };
                for (x_src, out_pixel) in out_row_f32.iter_mut().enumerate() {
                    let mut ss = 0.;
                    for (dy, &k) in ks.iter().enumerate() {
                        let pixel = src_image.get_pixel_f32(x_src as u32, first_y_src + dy as u32);
                        ss += pixel as f64 * k;
                    }
                    *out_pixel = ss as f32;
                }
            }
        }
    }
}
