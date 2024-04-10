use crate::{CropBoxError, ImageView, ImageViewMut};

fn check_crop_box(
    image_view: &impl ImageView,
    left: u32,
    top: u32,
    width: u32,
    height: u32,
) -> Result<(), CropBoxError> {
    let img_width = image_view.width();
    let img_height = image_view.height();

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

macro_rules! cropped_image_impl {
    ($wrapper_name:ident<$view_trait:ident>, $doc:expr) => {
        #[doc = $doc]
        pub struct $wrapper_name<V: $view_trait + Sized> {
            image_view: V,
            left: u32,
            top: u32,
            width: u32,
            height: u32,
        }

        impl<V: $view_trait + Sized> $wrapper_name<V> {
            pub fn new(
                image_view: V,
                left: u32,
                top: u32,
                width: u32,
                height: u32,
            ) -> Result<Self, CropBoxError> {
                check_crop_box(&image_view, left, top, width, height)?;
                Ok(Self {
                    image_view,
                    left,
                    top,
                    width,
                    height,
                })
            }
        }

        impl<V: $view_trait> ImageView for $wrapper_name<V> {
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
                    .iter_rows(self.top + start_row)
                    .take((self.height - start_row) as usize)
                    // SAFETY: correct values of the left and the right
                    // are guaranteed by new() method.
                    .map(move |row| unsafe { row.get_unchecked(left..right) })
            }
        }
    };
}

cropped_image_impl!(
    CroppedImage<ImageView>,
    "It is wrapper that provides [ImageView] for part of wrapped image."
);
cropped_image_impl!(
    CroppedImageMut<ImageViewMut>,
    "It is wrapper that provides [ImageViewMut] for part of wrapped image."
);

impl<V: ImageViewMut> ImageViewMut for CroppedImageMut<V> {
    fn iter_rows_mut(&mut self, start_row: u32) -> impl Iterator<Item = &mut [Self::Pixel]> {
        let left = self.left as usize;
        let right = left + self.width as usize;
        self.image_view
            .iter_rows_mut(self.top + start_row)
            .take((self.height - start_row) as usize)
            // SAFETY: correct values of the left and the right
            // are guaranteed by new() method.
            .map(move |row| unsafe { row.get_unchecked_mut(left..right) })
    }
}
