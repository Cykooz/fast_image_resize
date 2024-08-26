use crate::{ArrayChunks, ImageView, ImageViewMut};
use std::marker::PhantomData;
use std::num::NonZeroU32;

#[derive(Copy)]
pub(crate) struct UnsafeImageMut<'a, V>
where
    V: ImageViewMut,
{
    image: std::ptr::NonNull<V>,
    p: PhantomData<&'a V>,
}

impl<'a, V> Clone for UnsafeImageMut<'a, V>
where
    V: ImageViewMut,
{
    fn clone(&self) -> Self {
        Self {
            image: self.image,
            p: PhantomData,
        }
    }
}

unsafe impl<'a, V: ImageViewMut> Send for UnsafeImageMut<'a, V> {}
unsafe impl<'a, V: ImageViewMut> Sync for UnsafeImageMut<'a, V> {}

impl<'a, V: ImageViewMut> UnsafeImageMut<'a, V> {
    pub fn new(image: &'a mut V) -> Self {
        let ptr = std::ptr::NonNull::new(image as *mut V).unwrap();
        Self {
            image: ptr,
            p: PhantomData,
        }
    }

    fn get(&self) -> &V {
        unsafe { self.image.as_ref() }
    }

    fn get_mut(&mut self) -> &mut V {
        unsafe { self.image.as_mut() }
    }
}

unsafe impl<'a, V: ImageViewMut> ImageView for UnsafeImageMut<'a, V> {
    type Pixel = V::Pixel;

    fn width(&self) -> u32 {
        self.get().width()
    }

    fn height(&self) -> u32 {
        self.get().height()
    }

    fn iter_rows(&self, start_row: u32) -> impl Iterator<Item = &[Self::Pixel]> {
        self.get().iter_rows(start_row)
    }

    fn iter_2_rows(
        &self,
        start_y: u32,
        max_rows: u32,
    ) -> ArrayChunks<impl Iterator<Item = &[Self::Pixel]>, 2> {
        self.get().iter_2_rows(start_y, max_rows)
    }

    fn iter_4_rows(
        &self,
        start_y: u32,
        max_rows: u32,
    ) -> ArrayChunks<impl Iterator<Item = &[Self::Pixel]>, 4> {
        self.get().iter_4_rows(start_y, max_rows)
    }

    fn iter_rows_with_step(
        &self,
        start_y: f64,
        step: f64,
        max_rows: u32,
    ) -> impl Iterator<Item = &[Self::Pixel]> {
        self.get().iter_rows_with_step(start_y, step, max_rows)
    }

    fn split_by_height(
        &self,
        start_row: u32,
        height: NonZeroU32,
        num_parts: NonZeroU32,
    ) -> Option<Vec<impl ImageView<Pixel = Self::Pixel>>> {
        self.get().split_by_height(start_row, height, num_parts)
    }

    fn split_by_width(
        &self,
        start_col: u32,
        width: NonZeroU32,
        num_parts: NonZeroU32,
    ) -> Option<Vec<impl ImageView<Pixel = Self::Pixel>>> {
        self.get().split_by_width(start_col, width, num_parts)
    }
}

unsafe impl<'a, V: ImageViewMut> ImageViewMut for UnsafeImageMut<'a, V> {
    fn iter_rows_mut(&mut self, start_row: u32) -> impl Iterator<Item = &mut [Self::Pixel]> {
        self.get_mut().iter_rows_mut(start_row)
    }

    fn iter_2_rows_mut(&mut self) -> ArrayChunks<impl Iterator<Item = &mut [Self::Pixel]>, 2> {
        self.get_mut().iter_2_rows_mut()
    }

    fn iter_4_rows_mut(&mut self) -> ArrayChunks<impl Iterator<Item = &mut [Self::Pixel]>, 4> {
        self.get_mut().iter_4_rows_mut()
    }

    fn split_by_height_mut(
        &mut self,
        start_row: u32,
        height: NonZeroU32,
        num_parts: NonZeroU32,
    ) -> Option<Vec<impl ImageViewMut<Pixel = Self::Pixel>>> {
        self.get_mut()
            .split_by_height_mut(start_row, height, num_parts)
    }

    fn split_by_width_mut(
        &mut self,
        start_col: u32,
        width: NonZeroU32,
        num_parts: NonZeroU32,
    ) -> Option<Vec<impl ImageViewMut<Pixel = Self::Pixel>>> {
        self.get_mut()
            .split_by_width_mut(start_col, width, num_parts)
    }
}
