#![doc = include_str!("../README.md")]

pub use alpha::errors::*;
pub use convolution::FilterType;
pub use errors::*;
pub use image_rows::ImageRows;
pub use image_rows::ImageRowsMut;
pub use image_view::{CropBox, ImageView, ImageViewMut};
pub use mul_div::MulDiv;
pub use pixels::PixelType;
pub use resizer::{CpuExtensions, ResizeAlg, Resizer};

pub use crate::image::Image;

mod alpha;
mod convolution;
mod errors;
mod image;
mod image_rows;
mod image_view;
mod mul_div;
pub mod pixels;
mod resizer;
#[cfg(target_arch = "x86_64")]
mod simd_utils;
mod typed_image_view;
