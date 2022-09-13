use std::f64::consts::PI;

pub type FilterFn<'a> = &'a dyn Fn(f64) -> f64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FilterType {
    /// Each pixel of source image contributes to one pixel of the
    /// destination image with identical weights. For upscaling is equivalent
    /// of `Nearest` resize algorithm.    
    Box,
    /// Bilinear filter calculate the output pixel value using linear
    /// interpolation on all pixels that may contribute to the output value.
    Bilinear,
    /// Hamming filter has the same performance as `Bilinear` filter while
    /// providing the image downscaling quality comparable to bicubic
    /// (`CatmulRom` or `Mitchell`). Produces a sharper image than `Bilinear`,
    /// doesn't have dislocations on local level like with `Box`.
    /// The filter don’t show good quality for the image upscaling.
    Hamming,
    /// Catmull-Rom bicubic filter calculate the output pixel value using
    /// cubic interpolation on all pixels that may contribute to the output
    /// value.
    CatmullRom,
    /// Mitchell–Netravali bicubic filter calculate the output pixel value
    /// using cubic interpolation on all pixels that may contribute to the
    /// output value.
    Mitchell,
    /// Lanczos3 filter calculate the output pixel value using a high-quality
    /// Lanczos filter (a truncated sinc) on all pixels that may contribute
    /// to the output value.
    Lanczos3,
}

impl Default for FilterType {
    fn default() -> Self {
        FilterType::Lanczos3
    }
}

/// Returns reference to filter function and value of `filter_support`.
#[inline]
pub fn get_filter_func(filter_type: FilterType) -> (FilterFn<'static>, f64) {
    match filter_type {
        FilterType::Box => (&box_filter, 0.5),
        FilterType::Bilinear => (&bilinear_filter, 1.0),
        FilterType::Hamming => (&hamming_filter, 1.0),
        FilterType::CatmullRom => (&catmul_filter, 2.0),
        FilterType::Mitchell => (&mitchell_filter, 2.0),
        FilterType::Lanczos3 => (&lanczos_filter, 3.0),
    }
}

#[inline]
fn box_filter(x: f64) -> f64 {
    if x > -0.5 && x <= 0.5 {
        1.0
    } else {
        0.0
    }
}

#[inline]
fn bilinear_filter(mut x: f64) -> f64 {
    x = x.abs();
    if x < 1.0 {
        1.0 - x
    } else {
        0.0
    }
}

#[inline]
fn hamming_filter(mut x: f64) -> f64 {
    x = x.abs();
    if x == 0.0 {
        1.0
    } else if x >= 1.0 {
        0.0
    } else {
        x *= PI;
        (0.54 + 0.46 * x.cos()) * x.sin() / x
    }
}

/// Catmull-Rom (bicubic) filter
/// https://en.wikipedia.org/wiki/Bicubic_interpolation#Bicubic_convolution_algorithm
#[inline]
fn catmul_filter(mut x: f64) -> f64 {
    const A: f64 = -0.5;
    x = x.abs();
    if x < 1.0 {
        ((A + 2.) * x - (A + 3.)) * x * x + 1.
    } else if x < 2.0 {
        (((x - 5.) * x + 8.) * x - 4.) * A
    } else {
        0.0
    }
}

/// Mitchell–Netravali filter (B = C = 1/3)
/// https://en.wikipedia.org/wiki/Mitchell%E2%80%93Netravali_filters
#[inline]
fn mitchell_filter(mut x: f64) -> f64 {
    x = x.abs();
    if x < 1.0 {
        (7. * x / 6. - 2.) * x * x + 16. / 18.
    } else if x < 2.0 {
        ((2. - 7. * x / 18.) * x - 10. / 3.) * x + 16. / 9.
    } else {
        0.0
    }
}

#[inline]
fn sinc_filter(mut x: f64) -> f64 {
    if x == 0.0 {
        1.0
    } else {
        x *= PI;
        x.sin() / x
    }
}

#[inline]
fn lanczos_filter(x: f64) -> f64 {
    // truncated sinc
    if (-3.0..3.0).contains(&x) {
        sinc_filter(x) * sinc_filter(x / 3.)
    } else {
        0.0
    }
}
