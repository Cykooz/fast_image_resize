use crate::pixels::InnerPixel;
use crate::{ArrayChunks, ImageError, PixelTrait, PixelType};

/// A trait for getting access to image data.
///
/// # Safety
///
/// The length of the image rows returned by methods of this trait
/// must be equal or greater than the image width.
pub unsafe trait ImageView {
    type Pixel: InnerPixel;

    fn width(&self) -> u32;

    fn height(&self) -> u32;

    /// Returns iterator by slices with image rows.
    fn iter_rows(&self, start_row: u32) -> impl Iterator<Item = &[Self::Pixel]>;

    /// Returns iterator by arrays with two image rows.
    fn iter_2_rows(
        &self,
        start_y: u32,
        max_rows: u32,
    ) -> ArrayChunks<impl Iterator<Item = &[Self::Pixel]>, 2> {
        ArrayChunks::new(self.iter_rows(start_y).take(max_rows as usize))
    }

    /// Returns iterator by arrays with four image rows.
    fn iter_4_rows(
        &self,
        start_y: u32,
        max_rows: u32,
    ) -> ArrayChunks<impl Iterator<Item = &[Self::Pixel]>, 4> {
        ArrayChunks::new(self.iter_rows(start_y).take(max_rows as usize))
    }

    /// Returns iterator by slices with image rows selected from
    /// the image with the given step.
    fn iter_rows_with_step(
        &self,
        start_y: f64,
        step: f64,
        max_rows: u32,
    ) -> impl Iterator<Item = &[Self::Pixel]> {
        let steps = (self.height() as f64 - start_y) / step;
        let steps = (steps.max(0.).ceil() as usize).min(max_rows as usize);
        let mut rows = self.iter_rows(start_y as u32);
        let mut y = start_y;
        let mut next_row_y = start_y as usize;
        let mut cur_row = None;
        (0..steps).filter_map(move |_| {
            let req_row_y = y as usize;
            if next_row_y <= req_row_y {
                for _ in next_row_y..=req_row_y {
                    cur_row = rows.next();
                }
                next_row_y = req_row_y + 1;
            }
            y += step;
            cur_row
        })
    }
}

/// A trait for getting mutable access to image data.
///
/// # Safety
///
/// The length of the image rows returned by methods of this trait
/// must be equal or greater than the image width.
pub unsafe trait ImageViewMut: ImageView {
    /// Returns iterator by mutable slices with image rows.
    fn iter_rows_mut(&mut self, start_row: u32) -> impl Iterator<Item = &mut [Self::Pixel]>;

    /// Returns iterator by arrays with two mutable image rows.
    fn iter_2_rows_mut(&mut self) -> ArrayChunks<impl Iterator<Item = &mut [Self::Pixel]>, 2> {
        ArrayChunks::new(self.iter_rows_mut(0))
    }

    /// Returns iterator by arrays with four mutable image rows.
    fn iter_4_rows_mut(&mut self) -> ArrayChunks<impl Iterator<Item = &mut [Self::Pixel]>, 4> {
        ArrayChunks::new(self.iter_rows_mut(0))
    }
}

/// Conversion into an [ImageView].
pub trait IntoImageView {
    /// Returns pixel's type of the image.
    fn pixel_type(&self) -> Option<PixelType>;

    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn image_view<P: PixelTrait>(&self) -> Option<impl ImageView<Pixel = P>>;
}

/// Conversion into an [ImageViewMut].
pub trait IntoImageViewMut: IntoImageView {
    fn image_view_mut<P: PixelTrait>(&mut self) -> Option<impl ImageViewMut<Pixel = P>>;
}

/// Returns supported by the crate pixel's type of the image or `ImageError` if the image
/// has not supported pixel's type.
pub(crate) fn try_pixel_type(image: &impl IntoImageView) -> Result<PixelType, ImageError> {
    image.pixel_type().ok_or(ImageError::UnsupportedPixelType)
}
