#![doc = include_str!("../README.md")]
//!
//! ## Feature flags
#![doc = document_features::document_features!()]

pub use alpha::errors::*;
pub use array_chunks::*;
pub use change_components_type::*;
pub use color::mappers::*;
pub use color::PixelComponentMapper;
pub use convolution::*;
pub use cpu_extensions::CpuExtensions;
pub use crop_box::*;
pub use errors::*;
pub use image_view::*;
pub use mul_div::MulDiv;
pub use pixels::PixelType;
pub use resizer::{ResizeAlg, ResizeOptions, Resizer, SrcCropping};

use crate::alpha::AlphaMulDiv;

#[macro_use]
mod utils;

mod alpha;
mod array_chunks;
mod change_components_type;
mod color;
mod convolution;
mod cpu_extensions;
mod crop_box;
mod errors;
mod image_view;
pub mod images;
mod mul_div;
#[cfg(target_arch = "aarch64")]
mod neon_utils;
pub mod pixels;
mod resizer;
#[cfg(target_arch = "x86_64")]
mod simd_utils;
#[cfg(feature = "for_testing")]
pub mod testing;
#[cfg(feature = "rayon")]
pub(crate) mod threading;
#[cfg(target_arch = "wasm32")]
mod wasm32_utils;

/// A trait implemented by all pixel types from the crate.
///
/// This trait must be used in your code instead of [InnerPixel](pixels::InnerPixel).
#[allow(private_bounds)]
pub trait PixelTrait: Convolution + AlphaMulDiv {}

impl<P: Convolution + AlphaMulDiv> PixelTrait for P {}
