#![doc = include_str!("../README.md")]

pub use alpha::{MulDiv, MulDivImageError, MulDivImagesError};
pub use convolution::FilterType;
pub use errors::{CropBoxError, ImageBufferError, ImageRowsError, InvalidBufferSizeError};
pub use image_data::ImageData;
pub use image_view::{CropBox, DstImageView, PixelType, SrcImageView};
pub use resizer::{CpuExtensions, ResizeAlg, Resizer};

mod alpha;
mod convolution;
mod errors;
mod image_data;
mod image_view;
mod resizer;
mod simd_utils;
