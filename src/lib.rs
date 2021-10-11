#![doc = include_str!("../README.md")]

pub use alpha::{MulDiv, MulDivImageError, MulDivImagesError};
pub use convolution::FilterType;
pub use errors::*;
pub use image_view::{CropBox, ImageRows, ImageRowsMut, ImageView, ImageViewMut};
pub use pixels::PixelType;
pub use resizer::{CpuExtensions, ResizeAlg, Resizer};

pub use crate::image::Image;

mod alpha;
mod convolution;
mod errors;
mod image;
mod image_view;
mod pixels;
mod resizer;
#[cfg(target_arch = "x86_64")]
mod simd_utils;
