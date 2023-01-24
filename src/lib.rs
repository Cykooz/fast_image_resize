#![doc = include_str!("../README.md")]

pub use alpha::errors::*;
pub use color::mappers::*;
pub use color::PixelComponentMapper;
pub use convolution::FilterType;
pub use dynamic_image_view::{
    change_type_of_pixel_components_dyn, DynamicImageView, DynamicImageViewMut,
};
pub use errors::*;
pub use image_view::{change_type_of_pixel_components, CropBox, ImageView, ImageViewMut};
pub use mul_div::MulDiv;
pub use pixels::PixelType;
pub use resizer::{CpuExtensions, ResizeAlg, Resizer};

pub use crate::image::Image;

#[macro_use]
mod utils;

mod alpha;
mod color;
mod convolution;
mod dynamic_image_view;
mod errors;
mod image;
mod image_view;
mod mul_div;
#[cfg(target_arch = "aarch64")]
mod neon_utils;
pub mod pixels;
mod resizer;
#[cfg(target_arch = "x86_64")]
mod simd_utils;
#[cfg(feature = "for_test")]
pub mod testing;
#[cfg(target_arch = "wasm32")]
mod wasm32_utils;
