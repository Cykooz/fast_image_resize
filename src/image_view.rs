use std::fmt::Debug;
use std::mem::ManuallyDrop;
use std::num::NonZeroU32;
use std::slice;

use crate::pixels::{GetCount, IntoPixelComponent, PixelComponent, PixelExt};
use crate::{CropBoxError, DifferentDimensionsError, ImageBufferError, ImageRowsError, PixelType};

/// Parameters of crop box that may be used with [`ImageView`]
/// and [`DynamicImageView`](crate::DynamicImageView)
#[derive(Debug, Clone, Copy)]
pub struct CropBox {
    pub left: u32,
    pub top: u32,
    pub width: NonZeroU32,
    pub height: NonZeroU32,
}

/// Generic immutable image view.
#[derive(Debug, Clone)]
pub struct ImageView<'a, P>
where
    P: PixelExt,
{
    width: NonZeroU32,
    height: NonZeroU32,
    crop_box: CropBox,
    rows: Vec<&'a [P]>,
}

impl<'a, P> ImageView<'a, P>
where
    P: PixelExt,
{
    pub fn new(
        width: NonZeroU32,
        height: NonZeroU32,
        rows: Vec<&'a [P]>,
    ) -> Result<Self, ImageRowsError> {
        check_rows_count_and_size(width, height, &rows)?;
        Ok(Self {
            width,
            height,
            crop_box: CropBox {
                left: 0,
                top: 0,
                width,
                height,
            },
            rows,
        })
    }

    pub fn from_buffer(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: &'a [u8],
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize * P::size();
        if buffer.len() < size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        let rows_count = height.get() as usize;
        let pixels = align_buffer_to(buffer)?;
        let rows = pixels
            .chunks_exact(width.get() as usize)
            .take(rows_count)
            .collect();
        Ok(Self {
            width,
            height,
            crop_box: CropBox {
                left: 0,
                top: 0,
                width,
                height,
            },
            rows,
        })
    }

    pub fn from_pixels(
        width: NonZeroU32,
        height: NonZeroU32,
        pixels: &'a [P],
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize;
        if pixels.len() < size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        let rows_count = height.get() as usize;
        let rows = pixels
            .chunks_exact(width.get() as usize)
            .take(rows_count)
            .collect();
        Ok(Self {
            width,
            height,
            crop_box: CropBox {
                left: 0,
                top: 0,
                width,
                height,
            },
            rows,
        })
    }

    pub fn pixel_type(&self) -> PixelType {
        P::pixel_type()
    }

    pub fn width(&self) -> NonZeroU32 {
        self.width
    }

    pub fn height(&self) -> NonZeroU32 {
        self.height
    }

    pub fn crop_box(&self) -> CropBox {
        self.crop_box
    }

    pub fn set_crop_box(&mut self, crop_box: CropBox) -> Result<(), CropBoxError> {
        if crop_box.left >= self.width.get() || crop_box.top >= self.height.get() {
            return Err(CropBoxError::PositionIsOutOfImageBoundaries);
        }
        let right = crop_box.left + crop_box.width.get();
        let bottom = crop_box.top + crop_box.height.get();
        if right > self.width.get() || bottom > self.height.get() {
            return Err(CropBoxError::SizeIsOutOfImageBoundaries);
        }
        self.crop_box = crop_box;
        Ok(())
    }

    /// Set a crop box to resize the source image into the
    /// aspect ratio of destination image without distortions.
    ///
    /// `centering` used to control the cropping position. Use (0.5, 0.5) for
    /// center cropping (e.g. if cropping the width, take 50% off
    /// of the left side, and therefore 50% off the right side).
    /// (0.0, 0.0) will crop from the top left corner (i.e. if
    /// cropping the width, take all the crop off of the right
    /// side, and if cropping the height, take all of it off the
    /// bottom). (1.0, 0.0) will crop from the bottom left
    /// corner, etc. (i.e. if cropping the width, take all the
    /// crop off the left side, and if cropping the height take
    /// none from the top, and therefore all off the bottom).
    pub fn set_crop_box_to_fit_dst_size(
        &mut self,
        dst_width: NonZeroU32,
        dst_height: NonZeroU32,
        centering: Option<(f32, f32)>,
    ) {
        // This function based on code of ImageOps.fit() from Pillow package.
        // https://github.com/python-pillow/Pillow/blob/master/src/PIL/ImageOps.py
        let centering = if let Some((x, y)) = centering {
            (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0))
        } else {
            (0.5, 0.5)
        };

        // calculate aspect ratios
        let width = self.width.get() as f32;
        let height = self.height.get() as f32;
        let image_ratio = width / height;
        let required_ration = dst_width.get() as f32 / dst_height.get() as f32;

        let crop_width;
        let crop_height;
        // figure out if the sides or top/bottom will be cropped off
        if (image_ratio - required_ration).abs() < f32::EPSILON {
            // The image is already the needed ratio
            crop_width = width;
            crop_height = height;
        } else if image_ratio >= required_ration {
            // The image is wider than what's needed, crop the sides
            crop_width = required_ration * height;
            crop_height = height;
        } else {
            // The image is taller than what's needed, crop the top and bottom
            crop_width = width;
            crop_height = width / required_ration;
        }

        let crop_left = (width - crop_width) * centering.0;
        let crop_top = (height - crop_height) * centering.1;

        self.set_crop_box(CropBox {
            left: crop_left.round() as u32,
            top: crop_top.round() as u32,
            width: NonZeroU32::new(crop_width.round() as u32).unwrap(),
            height: NonZeroU32::new(crop_height.round() as u32).unwrap(),
        })
        .unwrap();
    }

    #[inline(always)]
    pub(crate) fn iter_4_rows<'s>(
        &'s self,
        start_y: u32,
        max_y: u32,
    ) -> impl Iterator<Item = [&'a [P]; 4]> + 's {
        let start_y = start_y as usize;
        let max_y = max_y.min(self.height.get()) as usize;
        let rows = self.rows.get(start_y..max_y).unwrap_or(&[]);
        rows.chunks_exact(4).map(|rows| match *rows {
            [r0, r1, r2, r3] => [r0, r1, r2, r3],
            _ => unreachable!(),
        })
    }

    #[inline(always)]
    pub(crate) fn iter_2_rows<'s>(
        &'s self,
        start_y: u32,
        max_y: u32,
    ) -> impl Iterator<Item = [&'a [P]; 2]> + 's {
        let start_y = start_y as usize;
        let max_y = max_y.min(self.height.get()) as usize;
        let rows = self.rows.get(start_y..max_y).unwrap_or(&[]);
        rows.chunks_exact(2).map(|rows| match *rows {
            [r0, r1] => [r0, r1],
            _ => unreachable!(),
        })
    }

    #[inline(always)]
    pub(crate) fn iter_rows<'s>(&'s self, start_y: u32) -> impl Iterator<Item = &'a [P]> + 's {
        let start_y = start_y as usize;
        let rows = self.rows.get(start_y..).unwrap_or(&[]);
        rows.iter().copied()
    }

    #[inline(always)]
    pub(crate) fn get_row(&self, y: u32) -> Option<&'a [P]> {
        self.rows.get(y as usize).copied()
    }

    #[inline(always)]
    pub(crate) fn iter_rows_with_step<'s>(
        &'s self,
        mut y: f64,
        step: f64,
        max_count: usize,
    ) -> impl Iterator<Item = &'a [P]> + 's {
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
    pub(crate) fn iter_cropped_rows<'s>(&'s self) -> impl Iterator<Item = &'a [P]> + 's {
        let first_row = self.crop_box.top as usize;
        let last_row = first_row + self.crop_box.height.get() as usize;
        let rows = unsafe { self.rows.get_unchecked(first_row..last_row) };

        let first_col = self.crop_box.left as usize;
        let last_col = first_col + self.crop_box.width.get() as usize;
        rows.iter()
            // Safety guaranteed by method 'set_crop_box'
            .map(move |row| unsafe { row.get_unchecked(first_col..last_col) })
    }
}

/// Generic mutable image view.
#[derive(Debug)]
pub struct ImageViewMut<'a, P>
where
    P: PixelExt,
{
    width: NonZeroU32,
    height: NonZeroU32,
    rows: Vec<&'a mut [P]>,
}

impl<'a, P> ImageViewMut<'a, P>
where
    P: PixelExt,
{
    pub fn new(
        width: NonZeroU32,
        height: NonZeroU32,
        rows: Vec<&'a mut [P]>,
    ) -> Result<Self, ImageRowsError> {
        check_rows_count_and_size(width, height, &rows)?;
        Ok(Self {
            width,
            height,
            rows,
        })
    }

    pub fn from_buffer(
        width: NonZeroU32,
        height: NonZeroU32,
        buffer: &'a mut [u8],
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize * P::size();
        if buffer.len() < size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        let rows_count = height.get() as usize;
        let pixels = align_buffer_to_mut(buffer)?;
        let rows = pixels
            .chunks_exact_mut(width.get() as usize)
            .take(rows_count)
            .collect();
        Ok(Self {
            width,
            height,
            rows,
        })
    }

    pub fn from_pixels(
        width: NonZeroU32,
        height: NonZeroU32,
        pixels: &'a mut [P],
    ) -> Result<Self, ImageBufferError> {
        let size = (width.get() * height.get()) as usize;
        if pixels.len() < size {
            return Err(ImageBufferError::InvalidBufferSize);
        }
        let rows_count = height.get() as usize;
        let rows = pixels
            .chunks_exact_mut(width.get() as usize)
            .take(rows_count)
            .collect();
        Ok(Self {
            width,
            height,
            rows,
        })
    }

    pub fn pixel_type(&self) -> PixelType {
        P::pixel_type()
    }

    pub fn width(&self) -> NonZeroU32 {
        self.width
    }

    pub fn height(&self) -> NonZeroU32 {
        self.height
    }

    #[inline(always)]
    pub(crate) fn iter_rows_mut(&mut self) -> slice::IterMut<&'a mut [P]> {
        self.rows.iter_mut()
    }

    #[inline(always)]
    pub(crate) fn iter_4_rows_mut<'s>(
        &'s mut self,
    ) -> impl Iterator<Item = [&'s mut &'a mut [P]; 4]> {
        self.rows.chunks_exact_mut(4).map(|rows| match rows {
            [a, b, c, d] => [a, b, c, d],
            _ => unreachable!(),
        })
    }

    #[inline(always)]
    pub(crate) fn get_row_mut<'s>(&'s mut self, y: u32) -> Option<&'s mut &'a mut [P]> {
        self.rows.get_mut(y as usize)
    }

    /// Copy pixels from src_view.
    pub(crate) fn copy_from_view(
        &mut self,
        src_view: &ImageView<P>,
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

    /// Create cropped version of the view.
    pub fn crop(self, crop_box: CropBox) -> Result<Self, CropBoxError> {
        if crop_box.left >= self.width.get() || crop_box.top >= self.height.get() {
            return Err(CropBoxError::PositionIsOutOfImageBoundaries);
        }
        let right = crop_box.left + crop_box.width.get();
        let bottom = crop_box.top + crop_box.height.get();
        if right > self.width.get() || bottom > self.height.get() {
            return Err(CropBoxError::SizeIsOutOfImageBoundaries);
        }
        let row_range = (crop_box.left as usize)..(right as usize);
        let rows = self
            .rows
            .into_iter()
            .skip(crop_box.top as usize)
            .take(crop_box.height.get() as usize)
            .map(|row| unsafe { row.get_unchecked_mut(row_range.clone()) })
            .collect();
        Ok(Self {
            width: crop_box.width,
            height: crop_box.height,
            rows,
        })
    }
}

impl<'a, P> From<ImageViewMut<'a, P>> for ImageView<'a, P>
where
    P: PixelExt,
{
    fn from(view: ImageViewMut<'a, P>) -> Self {
        let rows = {
            let mut old_rows = ManuallyDrop::new(view.rows);
            let (ptr, length, capacity) =
                (old_rows.as_mut_ptr(), old_rows.len(), old_rows.capacity());
            unsafe { Vec::from_raw_parts(ptr as *mut &[P], length, capacity) }
        };
        ImageView {
            width: view.width,
            height: view.height,
            crop_box: CropBox {
                left: 0,
                top: 0,
                width: view.width,
                height: view.height,
            },
            rows,
        }
    }
}

fn check_rows_count_and_size<T>(
    width: NonZeroU32,
    height: NonZeroU32,
    rows: &[impl AsRef<[T]>],
) -> Result<(), ImageRowsError> {
    if rows.len() != height.get() as usize {
        return Err(ImageRowsError::InvalidRowsCount);
    }
    let row_size = width.get() as usize;
    if rows.iter().any(|row| row.as_ref().len() != row_size) {
        return Err(ImageRowsError::InvalidRowSize);
    }
    Ok(())
}

fn align_buffer_to<T>(buffer: &[u8]) -> Result<&[T], ImageBufferError> {
    let (head, pixels, _) = unsafe { buffer.align_to::<T>() };
    if !head.is_empty() {
        return Err(ImageBufferError::InvalidBufferAlignment);
    }
    Ok(pixels)
}

fn align_buffer_to_mut<T>(buffer: &mut [u8]) -> Result<&mut [T], ImageBufferError> {
    let (head, pixels, _) = unsafe { buffer.align_to_mut::<T>() };
    if !head.is_empty() {
        return Err(ImageBufferError::InvalidBufferAlignment);
    }
    Ok(pixels)
}

pub fn change_type_of_pixel_components<S, D, In, Out, CC>(
    src_image: &ImageView<S>,
    dst_image: &mut ImageViewMut<D>,
) -> Result<(), DifferentDimensionsError>
where
    Out: PixelComponent,
    In: IntoPixelComponent<Out>,
    CC: GetCount,
    S: PixelExt<Component = In, CountOfComponents = CC>,
    D: PixelExt<Component = Out, CountOfComponents = CC>,
{
    if src_image.width() != dst_image.width() || src_image.height() != dst_image.height() {
        return Err(DifferentDimensionsError);
    }

    for (s_row, d_row) in src_image.rows.iter().zip(dst_image.rows.iter_mut()) {
        let s_components = S::components(s_row);
        let d_components = D::components_mut(d_row);
        for (&s_comp, d_comp) in s_components.iter().zip(d_components) {
            *d_comp = s_comp.into_component();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_view_mut() {
        let mut image = crate::Image::new(
            NonZeroU32::new(64).unwrap(),
            NonZeroU32::new(32).unwrap(),
            PixelType::U8,
        );

        let image_view: ImageViewMut<crate::pixels::U8> =
            ImageViewMut::from_buffer(image.width(), image.height(), image.buffer_mut()).unwrap();
        let cropped_view = image_view
            .crop(CropBox {
                left: 10,
                top: 10,
                width: NonZeroU32::new(44).unwrap(),
                height: NonZeroU32::new(12).unwrap(),
            })
            .unwrap();
        assert_eq!(cropped_view.width().get(), 44);
        assert_eq!(cropped_view.height().get(), 12);
        assert_eq!(cropped_view.rows.len(), 12);
        for row in cropped_view.rows.iter() {
            assert_eq!(row.len(), 44);
        }
    }
}
