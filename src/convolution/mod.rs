pub use filters::*;

use crate::pixels::InnerPixel;
use crate::{CpuExtensions, ImageView, ImageViewMut};

#[macro_use]
mod macros;

mod filters;
#[macro_use]
mod optimisations;
mod u8x4;
mod vertical_u8;
cfg_if::cfg_if! {
    if #[cfg(not(feature = "only_u8x4"))] {
        mod u8x1;
        mod u8x2;
        mod u8x3;
        mod u16x1;
        mod u16x2;
        mod u16x3;
        mod u16x4;
        mod i32x1;
        mod f32x1;
        mod f32x2;
        mod f32x3;
        mod f32x4;
        mod vertical_u16;
        mod vertical_f32;
    }
}

pub(crate) trait Convolution: InnerPixel {
    fn horiz_convolution(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        offset: u32,
        coeffs: Coefficients,
        cpu_extensions: CpuExtensions,
    );

    fn vert_convolution(
        src_view: &impl ImageView<Pixel = Self>,
        dst_view: &mut impl ImageViewMut<Pixel = Self>,
        offset: u32,
        coeffs: Coefficients,
        cpu_extensions: CpuExtensions,
    );
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Bound {
    pub start: u32,
    pub size: u32,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Coefficients {
    pub values: Vec<f64>,
    pub window_size: usize,
    pub bounds: Vec<Bound>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CoefficientsChunk<'a> {
    pub start: u32,
    pub values: &'a [f64],
}

impl Coefficients {
    pub fn get_chunks(&self) -> Vec<CoefficientsChunk<'_>> {
        let mut coeffs = self.values.as_slice();
        let mut res = Vec::with_capacity(self.bounds.len());
        for bound in &self.bounds {
            let (left, right) = coeffs.split_at(self.window_size);
            coeffs = right;
            let size = bound.size as usize;
            res.push(CoefficientsChunk {
                start: bound.start,
                values: &left[0..size],
            });
        }
        res
    }
}

pub(crate) fn precompute_coefficients(
    in_size: u32,
    in0: f64, // Left/top border for cropping
    in1: f64, // Right/bottom border for cropping
    out_size: u32,
    filter: fn(f64) -> f64,
    filter_support: f64,
    adaptive_kernel_size: bool,
) -> Coefficients {
    if in_size == 0 || out_size == 0 {
        return Coefficients::default();
    }
    let scale = (in1 - in0) / out_size as f64;
    if scale <= 0. {
        return Coefficients::default();
    }
    let filter_scale = if adaptive_kernel_size {
        scale.max(1.0)
    } else {
        1.0
    };

    // Determine filter radius size (length of resampling filter)
    let filter_radius = filter_support * filter_scale;
    // Maximum number of coeffs per out pixel
    let window_size = filter_radius.ceil() as usize * 2 + 1;
    // Optimization: replace division by filter_scale
    // with multiplication by recip_filter_scale
    let recip_filter_scale = 1.0 / filter_scale;

    let count_of_coeffs = window_size * out_size as usize;
    let mut coeffs: Vec<f64> = Vec::with_capacity(count_of_coeffs);
    let mut bounds: Vec<Bound> = Vec::with_capacity(out_size as usize);

    for out_x in 0..out_size {
        // Find the point in the input image corresponding to the center
        // of the current pixel in the output image.
        let in_center = in0 + (out_x as f64 + 0.5) * scale;

        // x_min and x_max are slice bounds for the input pixels relevant
        // to the output pixel we are calculating. Pixel x is relevant
        // if and only if (x >= x_min) && (x < x_max).
        // Invariant: 0 <= x_min < x_max <= width
        let x_min = (in_center - filter_radius).floor().max(0.) as u32;
        let x_max = (in_center + filter_radius).ceil().min(in_size as f64) as u32;

        let cur_index = coeffs.len();
        let mut ww: f64 = 0.0;

        // Optimisation for follow for-cycle:
        // (x + 0.5) - in_center => x - (in_center - 0.5) => x - center
        let center = in_center - 0.5;

        let mut bound_start = x_min;
        let mut bound_end = x_max;

        // Calculate the weight of each input pixel from the given x-range.
        for x in x_min..x_max {
            let w: f64 = filter((x as f64 - center) * recip_filter_scale);
            if x == bound_start && w == 0. {
                // Don't use zero coefficients at the start of bound;
                bound_start += 1;
            } else {
                coeffs.push(w);
                ww += w;
            }
        }
        for &c in coeffs.iter().rev() {
            if bound_end <= bound_start || c != 0. {
                break;
            }
            // Don't use zero coefficients at the end of bound;
            bound_end -= 1;
        }

        if ww != 0.0 {
            // Normalise values of weights.
            // The sum of weights must be equal to 1.0.
            coeffs[cur_index..].iter_mut().for_each(|w| *w /= ww);
        }
        // Remaining values should stay empty if they are used despite x_max.
        coeffs.resize(cur_index + window_size, 0.);
        bounds.push(Bound {
            start: bound_start,
            size: bound_end - bound_start,
        });
    }

    Coefficients {
        values: coeffs,
        window_size,
        bounds,
    }
}
