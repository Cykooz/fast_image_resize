use std::num::NonZeroU32;
use std::slice;

use crate::image_view::{FourRows, FourRowsMut, RowMut, TwoRows};
use crate::pixels::Pixel;
use crate::{CropBox, DifferentDimensionsError, ImageView, ImageViewMut};

/// Generic immutable image view.
pub(crate) struct TypedImageView<'a, 'b, P>
where
    P: Pixel,
{
    width: NonZeroU32,
    height: NonZeroU32,
    crop_box: CropBox,
    rows: &'a [&'b [P]],
}

impl<'a, 'b, P> TypedImageView<'a, 'b, P>
where
    P: Pixel,
{
    pub fn new(width: NonZeroU32, height: NonZeroU32, rows: &'a [&'b [P]]) -> Self {
        Self {
            width,
            height,
            crop_box: CropBox {
                left: 0,
                top: 0,
                width,
                height,
            },
            rows,
        }
    }

    pub fn from_image_view(image_view: &'a ImageView<'b>) -> Option<Self> {
        image_view.typed_rows().map(|typed_rows| Self {
            width: image_view.width(),
            height: image_view.height(),
            crop_box: image_view.crop_box(),
            rows: typed_rows,
        })
    }

    #[inline(always)]
    pub fn width(&self) -> NonZeroU32 {
        self.width
    }

    #[inline(always)]
    pub fn height(&self) -> NonZeroU32 {
        self.height
    }

    #[inline(always)]
    pub fn crop_box(&self) -> CropBox {
        self.crop_box
    }

    #[inline(always)]
    pub(crate) fn iter_4_rows<'s>(
        &'s self,
        start_y: u32,
        max_y: u32,
    ) -> impl Iterator<Item = FourRows<'b, P>> + 's {
        let start_y = start_y as usize;
        let max_y = max_y.min(self.height.get()) as usize;
        let rows = self.rows.get(start_y..max_y).unwrap_or(&[]);
        rows.chunks_exact(4).map(|rows| match *rows {
            [r0, r1, r2, r3] => (r0, r1, r2, r3),
            _ => unreachable!(),
        })
    }

    #[inline(always)]
    pub(crate) fn iter_2_rows<'s>(
        &'s self,
        start_y: u32,
        max_y: u32,
    ) -> impl Iterator<Item = TwoRows<'b, P>> + 's {
        let start_y = start_y as usize;
        let max_y = max_y.min(self.height.get()) as usize;
        let rows = self.rows.get(start_y..max_y).unwrap_or(&[]);
        rows.chunks_exact(2).map(|rows| match *rows {
            [r0, r1] => (r0, r1),
            _ => unreachable!(),
        })
    }

    #[inline(always)]
    pub(crate) fn iter_rows<'s>(&'s self, start_y: u32) -> impl Iterator<Item = &'b [P]> + 's {
        let start_y = start_y as usize;
        let rows = self.rows.get(start_y..).unwrap_or(&[]);
        rows.iter().copied()
    }

    #[inline(always)]
    pub(crate) fn get_row(&self, y: u32) -> Option<&'b [P]> {
        self.rows.get(y as usize).copied()
    }

    #[inline(always)]
    pub(crate) fn iter_rows_with_step<'s>(
        &'s self,
        mut y: f64,
        step: f64,
        max_count: usize,
    ) -> impl Iterator<Item = &'b [P]> + 's {
        let steps = (self.height.get() as f64 - y) / step;
        let steps = (steps.max(0.).ceil() as usize).min(max_count);
        (0..steps).map(move |_| {
            // Safety of value of y guaranteed by calculation of steps count
            let row = unsafe { *self.rows.get_unchecked(y as usize) };
            y += step;
            row
        })
    }

    #[inline(always)]
    pub(crate) fn iter_cropped_rows<'s>(&'s self) -> impl Iterator<Item = &'b [P]> + 's {
        let first_row = self.crop_box.top as usize;
        let last_row = first_row + self.crop_box.height.get() as usize;
        let rows = unsafe { self.rows.get_unchecked(first_row..last_row) };

        let first_col = self.crop_box.left as usize;
        let last_col = first_col + self.crop_box.width.get() as usize;
        rows.iter()
            .map(move |row| unsafe { row.get_unchecked(first_col..last_col) })
    }
}

/// Generic mutable image view.
pub(crate) struct TypedImageViewMut<'a, 'b, P>
where
    P: Pixel,
{
    width: NonZeroU32,
    height: NonZeroU32,
    rows: &'a mut [&'b mut [P]],
}

impl<'a, 'b, P> TypedImageViewMut<'a, 'b, P>
where
    P: Pixel,
{
    pub fn new(width: NonZeroU32, height: NonZeroU32, rows: &'a mut [&'b mut [P]]) -> Self {
        Self {
            width,
            height,
            rows,
        }
    }

    pub fn from_image_view(image_view: &'a mut ImageViewMut<'b>) -> Option<Self> {
        let width = image_view.width();
        let height = image_view.height();
        image_view.typed_rows().map(|typed_rows| Self {
            width,
            height,
            rows: typed_rows,
        })
    }

    #[inline(always)]
    pub fn width(&self) -> NonZeroU32 {
        self.width
    }

    #[inline(always)]
    pub fn height(&self) -> NonZeroU32 {
        self.height
    }

    #[inline(always)]
    pub fn iter_rows_mut(&mut self) -> slice::IterMut<&'b mut [P]> {
        self.rows.iter_mut()
    }

    #[inline(always)]
    pub fn iter_4_rows_mut<'s>(&'s mut self) -> impl Iterator<Item = FourRowsMut<'s, 'b, P>> {
        self.rows.chunks_exact_mut(4).map(|rows| match rows {
            [a, b, c, d] => (a, b, c, d),
            _ => unreachable!(),
        })
    }

    #[inline(always)]
    pub fn get_row_mut<'s>(&'s mut self, y: u32) -> Option<RowMut<'s, 'b, P>> {
        self.rows.get_mut(y as usize)
    }

    /// Copy into the view pixels from src_view.
    pub(crate) fn copy_from_view(
        &mut self,
        src_view: &TypedImageView<P>,
    ) -> Result<(), DifferentDimensionsError> {
        let src_crop_box = src_view.crop_box();
        if self.width != src_crop_box.width || self.height != src_crop_box.height {
            return Err(DifferentDimensionsError);
        }
        self.rows
            .iter_mut()
            .zip(src_view.iter_cropped_rows())
            .for_each(|(d, s)| d.copy_from_slice(s));
        Ok(())
    }
}
