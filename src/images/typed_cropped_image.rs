use crate::images::{View, ViewMut};
use crate::{CropBoxError, ImageView, ImageViewMut};
use std::num::NonZeroU32;

pub(crate) fn check_crop_box(
    img_width: u32,
    img_height: u32,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
) -> Result<(), CropBoxError> {
    if left >= img_width || top >= img_height {
        return Err(CropBoxError::PositionIsOutOfImageBoundaries);
    }
    let right = left + width;
    let bottom = top + height;
    if right > img_width || bottom > img_height {
        return Err(CropBoxError::SizeIsOutOfImageBoundaries);
    }
    Ok(())
}

/// It is a typed wrapper that provides [ImageView] for part of wrapped image.
pub struct TypedCroppedImage<'a, V: ImageView + 'a> {
    image_view: View<'a, V>,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
}

/// It is a typed wrapper that provides [ImageView] and [ImageViewMut] for part of wrapped image.
pub struct TypedCroppedImageMut<'a, V: ImageViewMut> {
    image_view: ViewMut<'a, V>,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
}

impl<'a, V: ImageView + 'a> TypedCroppedImage<'a, V> {
    pub fn new(
        image_view: V,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
    ) -> Result<Self, CropBoxError> {
        check_crop_box(
            image_view.width(),
            image_view.height(),
            left,
            top,
            width,
            height,
        )?;
        Ok(Self {
            image_view: View::Owned(image_view),
            left,
            top,
            width,
            height,
        })
    }

    pub fn from_ref(
        image_view: &'a V,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
    ) -> Result<Self, CropBoxError> {
        check_crop_box(
            image_view.width(),
            image_view.height(),
            left,
            top,
            width,
            height,
        )?;
        Ok(Self {
            image_view: View::Borrowed(image_view),
            left,
            top,
            width,
            height,
        })
    }
}

impl<'a, V: ImageViewMut> TypedCroppedImageMut<'a, V> {
    pub fn new(
        image_view: V,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
    ) -> Result<Self, CropBoxError> {
        check_crop_box(
            image_view.width(),
            image_view.height(),
            left,
            top,
            width,
            height,
        )?;
        Ok(Self {
            image_view: ViewMut::Owned(image_view),
            left,
            top,
            width,
            height,
        })
    }

    pub fn from_ref(
        image_view: &'a mut V,
        left: u32,
        top: u32,
        width: u32,
        height: u32,
    ) -> Result<Self, CropBoxError> {
        check_crop_box(
            image_view.width(),
            image_view.height(),
            left,
            top,
            width,
            height,
        )?;
        Ok(Self {
            image_view: ViewMut::Borrowed(image_view),
            left,
            top,
            width,
            height,
        })
    }
}

macro_rules! image_view_impl {
    ($wrapper_name:ident<$view_trait:ident>) => {
        unsafe impl<'a, V: $view_trait> ImageView for $wrapper_name<'a, V> {
            type Pixel = V::Pixel;

            fn width(&self) -> u32 {
                self.width
            }

            fn height(&self) -> u32 {
                self.height
            }

            fn iter_rows(&self, start_row: u32) -> impl Iterator<Item = &[Self::Pixel]> {
                let left = self.left as usize;
                let right = left + self.width as usize;
                self.image_view
                    .get_ref()
                    .iter_rows(self.top + start_row)
                    .take((self.height - start_row) as usize)
                    // SAFETY: correct values of the left and the right
                    // are guaranteed by new() method.
                    .map(move |row| unsafe { row.get_unchecked(left..right) })
            }

            fn split_by_height(
                &self,
                start_row: u32,
                height: NonZeroU32,
                num_parts: NonZeroU32,
            ) -> Option<Vec<impl ImageView<Pixel = Self::Pixel>>> {
                let height_u32 = height.get();
                if num_parts > height
                    || height_u32 > self.height()
                    || start_row > self.height() - height_u32
                {
                    return None;
                }
                let image_view = self.image_view.get_ref();
                let images = image_view.split_by_height(start_row + self.top, height, num_parts);
                images.map(|v| {
                    v.into_iter()
                        .map(|img| {
                            let img_height = img.height();
                            TypedCroppedImage::new(img, self.left, 0, self.width, img_height)
                                .unwrap()
                        })
                        .collect()
                })
            }

            fn split_by_width(
                &self,
                start_col: u32,
                width: NonZeroU32,
                num_parts: NonZeroU32,
            ) -> Option<Vec<impl ImageView<Pixel = Self::Pixel>>> {
                let width_u32 = width.get();
                if num_parts > width
                    || width_u32 > self.width()
                    || start_col > self.width() - width_u32
                {
                    return None;
                }
                let image_view = self.image_view.get_ref();
                let images = image_view.split_by_width(start_col + self.left, width, num_parts);
                images.map(|v| {
                    v.into_iter()
                        .map(|img| {
                            let img_width = img.width();
                            TypedCroppedImage::new(img, 0, self.top, img_width, self.height)
                                .unwrap()
                        })
                        .collect()
                })
            }
        }
    };
}

image_view_impl!(TypedCroppedImage<ImageView>);

image_view_impl!(TypedCroppedImageMut<ImageViewMut>);

unsafe impl<'a, V: ImageViewMut> ImageViewMut for TypedCroppedImageMut<'a, V> {
    fn iter_rows_mut(&mut self, start_row: u32) -> impl Iterator<Item = &mut [Self::Pixel]> {
        let left = self.left as usize;
        let right = left + self.width as usize;
        self.image_view
            .get_mut()
            .iter_rows_mut(self.top + start_row)
            .take((self.height - start_row) as usize)
            // SAFETY: correct values of the left and the right
            // are guaranteed by new() method.
            .map(move |row| unsafe { row.get_unchecked_mut(left..right) })
    }

    fn split_by_height_mut(
        &mut self,
        start_row: u32,
        height: NonZeroU32,
        num_parts: NonZeroU32,
    ) -> Option<Vec<impl ImageViewMut<Pixel = Self::Pixel>>> {
        let height_u32 = height.get();
        if num_parts > height
            || height_u32 > self.height()
            || start_row > self.height() - height_u32
        {
            return None;
        }
        let image_view = self.image_view.get_mut();
        let images = image_view.split_by_height_mut(start_row + self.top, height, num_parts);
        images.map(|v| {
            v.into_iter()
                .map(|img| {
                    let img_height = img.height();
                    TypedCroppedImageMut::new(img, self.left, 0, self.width, img_height).unwrap()
                })
                .collect()
        })
    }

    fn split_by_width_mut(
        &mut self,
        start_col: u32,
        width: NonZeroU32,
        num_parts: NonZeroU32,
    ) -> Option<Vec<impl ImageViewMut<Pixel = Self::Pixel>>> {
        let width_u32 = width.get();
        if num_parts > width || width_u32 > self.width() || start_col > self.width() - width_u32 {
            return None;
        }
        let image_view = self.image_view.get_mut();
        let images = image_view.split_by_width_mut(start_col + self.left, width, num_parts);
        images.map(|v| {
            v.into_iter()
                .map(|img| {
                    let img_width = img.width();
                    TypedCroppedImageMut::new(img, 0, self.top, img_width, self.height).unwrap()
                })
                .collect()
        })
    }
}
