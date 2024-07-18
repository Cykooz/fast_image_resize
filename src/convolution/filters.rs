use std::f64::consts::PI;
use std::fmt::{Debug, Formatter};
use thiserror::Error;

type FilterFn = fn(f64) -> f64;

/// Description of custom filter for image convolution.
#[derive(Clone, Copy)]
pub struct Filter {
    /// Name of filter
    name: &'static str,
    /// Filter function
    func: FilterFn,
    /// Minimal "radius" of kernel in pixels
    support: f64,
}

impl PartialEq for Filter {
    fn eq(&self, other: &Self) -> bool {
        self.support == other.support && self.name == other.name
    }
}

impl Eq for Filter {}

impl Debug for Filter {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Filter")
            .field("name", &self.name)
            .field("support", &self.support)
            .finish()
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateFilterError {
    /// Value of 'support' argument must be finite and greater than 0.0
    #[error("Value of 'support' argument must be finite and greater than 0.0")]
    InvalidSupport,
}

impl Filter {
    /// # Arguments
    ///
    /// * `name` - Name of filter
    /// * `func` - Filter function
    /// * `support` - Minimal "radius" of kernel in pixels
    pub fn new(
        name: &'static str,
        func: FilterFn,
        support: f64,
    ) -> Result<Self, CreateFilterError> {
        if support.is_finite() && support > 0.0 {
            Ok(Self {
                name,
                func,
                support,
            })
        } else {
            Err(CreateFilterError::InvalidSupport)
        }
    }

    /// Name of filter
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Minimal "radius" of kernel in pixels
    pub fn support(&self) -> f64 {
        self.support
    }
}

/// Type of filter used for image convolution.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum FilterType {
    /// Each pixel of source image contributes to one pixel of the
    /// destination image with identical weights. For upscaling is equivalent
    /// of `Nearest` resize algorithm.
    ///
    /// Minimal kernel size 1x1 px.    
    Box,
    /// Bilinear filter calculates the output pixel value using linear
    /// interpolation on all pixels that may contribute to the output value.
    ///
    /// Minimal kernel size 2x2 px.
    Bilinear,
    /// Hamming filter has the same performance as `Bilinear` filter while
    /// providing the image downscaling quality comparable to bicubic
    /// (`CatmulRom` or `Mitchell`). Produces a sharper image than `Bilinear`,
    /// doesn't have dislocations on local level like with `Box`.
    /// The filter doesn't show good quality for the image upscaling.
    ///
    /// Minimal kernel size 2x2 px.
    Hamming,
    /// Catmull-Rom bicubic filter calculates the output pixel value using
    /// cubic interpolation on all pixels that may contribute to the output
    /// value.
    ///
    /// Minimal kernel size 4x4 px.
    CatmullRom,
    /// Mitchell–Netravali bicubic filter calculate the output pixel value
    /// using cubic interpolation on all pixels that may contribute to the
    /// output value.
    ///
    /// Minimal kernel size 4x4 px.
    Mitchell,
    /// Gaussian filter with a standard deviation of 0.5.
    ///
    /// Minimal kernel size 6x6 px.
    Gaussian,
    /// Lanczos3 filter calculate the output pixel value using a high-quality
    /// Lanczos filter (a truncated sinc) on all pixels that may contribute
    /// to the output value.
    ///
    /// Minimal kernel size 6x6 px.
    #[default]
    Lanczos3,
    /// Custom filter function.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use fast_image_resize::{Filter, FilterType};
    ///
    /// fn sinc_filter(mut x: f64) -> f64 {
    ///     if x == 0.0 {
    ///         1.0
    ///     } else {
    ///         x *= std::f64::consts::PI;
    ///         x.sin() / x
    ///     }
    /// }
    ///
    /// fn lanczos4_filter(x: f64) -> f64 {
    ///     if (-4.0..4.0).contains(&x) {
    ///         sinc_filter(x) * sinc_filter(x / 4.)
    ///     } else {
    ///         0.0
    ///     }
    /// }
    ///
    /// let lanczos4 = FilterType::Custom(
    ///     Filter::new("Lanczos4", lanczos4_filter, 4.0).unwrap()
    /// );
    ///
    /// assert_eq!(
    ///     format!("{:?}", lanczos4),
    ///     "Custom(Filter { name: \"Lanczos4\", support: 4.0 })"
    /// );
    /// ```
    Custom(Filter),
}

/// Returns reference to filter function and value of `filter_support`.
#[inline]
pub(crate) fn get_filter_func(filter_type: FilterType) -> (FilterFn, f64) {
    match filter_type {
        FilterType::Box => (box_filter, 0.5),
        FilterType::Bilinear => (bilinear_filter, 1.0),
        FilterType::Hamming => (hamming_filter, 1.0),
        FilterType::CatmullRom => (catmul_filter, 2.0),
        FilterType::Mitchell => (mitchell_filter, 2.0),
        FilterType::Gaussian => (gaussian_filter, 3.0),
        FilterType::Lanczos3 => (lanczos_filter, 3.0),
        FilterType::Custom(custom) => (custom.func, custom.support),
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

/// The Gaussian Function.
/// `r` is the standard deviation.
fn gaussian(x: f64, r: f64) -> f64 {
    ((2.0 * PI).sqrt() * r).recip() * (-x.powi(2) / (2.0 * r.powi(2))).exp()
}

/// Calculate the gaussian function with a
/// standard deviation of 0.5.
fn gaussian_filter(x: f64) -> f64 {
    if (-3.0..3.0).contains(&x) {
        gaussian(x, 0.5)
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
