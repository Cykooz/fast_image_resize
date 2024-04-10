use std::num::ParseIntError;
use std::str::FromStr;

use fast_image_resize as fr;

#[derive(Copy, Clone, Debug)]
pub enum Size {
    Pixels(u32),
    Percent(u16),
}

impl Size {
    pub fn calculate_size(&self, src_size: u32) -> u32 {
        match *self {
            Self::Pixels(size) => size,
            Self::Percent(percent) => (src_size as f32 * percent as f32 / 100.).round() as u32,
        }
    }
}

impl FromStr for Size {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(percent_str) = s.strip_suffix('%') {
            u16::from_str(percent_str).map(Self::Percent)
        } else {
            u32::from_str(s).map(Self::Pixels)
        }
    }
}

#[derive(Copy, Clone, Debug, clap::ValueEnum)]
pub enum Algorithm {
    Nearest,
    Convolution,
    SuperSampling,
}

#[derive(Copy, Clone, Debug, clap::ValueEnum)]
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

impl From<FilterType> for fr::FilterType {
    fn from(filter_type: FilterType) -> Self {
        match filter_type {
            FilterType::Box => fr::FilterType::Box,
            FilterType::Bilinear => fr::FilterType::Bilinear,
            FilterType::Hamming => fr::FilterType::Hamming,
            FilterType::CatmullRom => fr::FilterType::CatmullRom,
            FilterType::Mitchell => fr::FilterType::Mitchell,
            FilterType::Lanczos3 => fr::FilterType::Lanczos3,
        }
    }
}

#[derive(Copy, Clone, Debug, clap::ValueEnum)]
pub enum ColorSpace {
    Linear,
    /// sRGB for color images or gamma 2.2 for grayscale images
    NonLinear,
}
