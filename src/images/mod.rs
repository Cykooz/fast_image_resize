//! Contains different types of images and wrappers for them.
use std::fmt::Debug;

pub use cropped_image::*;
pub use image::*;
pub use typed_cropped_image::*;
pub use typed_image::*;
pub(crate) use unsafe_image::UnsafeImageMut;

mod cropped_image;
mod image;
mod typed_cropped_image;
mod typed_image;
mod unsafe_image;

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

enum View<'a, V: 'a> {
    Borrowed(&'a V),
    Owned(V),
}

impl<'a, V> View<'a, V> {
    fn get_ref(&self) -> &V {
        match self {
            Self::Borrowed(v_ref) => v_ref,
            Self::Owned(v_own) => v_own,
        }
    }
}

enum ViewMut<'a, V: 'a> {
    Borrowed(&'a mut V),
    Owned(V),
}

impl<'a, V> ViewMut<'a, V> {
    fn get_ref(&self) -> &V {
        match self {
            Self::Borrowed(v_ref) => v_ref,
            Self::Owned(v_own) => v_own,
        }
    }

    fn get_mut(&mut self) -> &mut V {
        match self {
            Self::Borrowed(p_ref) => p_ref,
            Self::Owned(vec) => vec,
        }
    }
}
