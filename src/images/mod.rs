//! Contains different types of images and wrappers for them.
pub use cropped_image::*;
pub use dyn_image::*;
pub use typed_image::*;

mod cropped_image;
mod dyn_image;
mod typed_image;

#[cfg(feature = "image")]
mod image_crate;
