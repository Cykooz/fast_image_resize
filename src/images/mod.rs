//! Contains different types of images and wrappers for them.
use std::fmt::Debug;

pub use cropped_image::*;
pub use image::*;
pub use typed_cropped_image::*;
pub use typed_image::*;

mod cropped_image;
mod image;
mod typed_cropped_image;
mod typed_image;

#[cfg(feature = "image")]
mod image_crate;

#[derive(Debug)]
enum BufferContainer<'a, T: Copy + Debug> {
    Borrowed(&'a mut [T]),
    Owned(Vec<T>),
}

impl<'a, T: Copy + Debug> BufferContainer<'a, T> {
    fn as_vec(&self) -> Vec<T> {
        match self {
            Self::Borrowed(slice) => slice.to_vec(),
            Self::Owned(vec) => vec.clone(),
        }
    }

    pub fn borrow(&self) -> &[T] {
        match self {
            Self::Borrowed(p_ref) => p_ref,
            Self::Owned(vec) => vec,
        }
    }

    pub fn borrow_mut(&mut self) -> &mut [T] {
        match self {
            Self::Borrowed(p_ref) => p_ref,
            Self::Owned(vec) => vec,
        }
    }
}
