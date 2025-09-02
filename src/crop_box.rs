use crate::{CropBoxError, ImageView};

/// A crop box parameters.
#[derive(Debug, Clone, Copy)]
pub struct CropBox {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

impl CropBox {
    /// Get a crop box to resize the source image into the
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
    pub fn fit_src_into_dst_size(
        src_width: u32,
        src_height: u32,
        dst_width: u32,
        dst_height: u32,
        centering: Option<(f64, f64)>,
    ) -> Self {
        if src_width == 0 || src_height == 0 || dst_width == 0 || dst_height == 0 {
            return Self {
                left: 0.,
                top: 0.,
                width: src_width as _,
                height: src_height as _,
            };
        }

        // This function based on code of ImageOps.fit() from Pillow package.
        // https://github.com/python-pillow/Pillow/blob/master/src/PIL/ImageOps.py
        let centering = if let Some((x, y)) = centering {
            (x.clamp(0.0, 1.0), y.clamp(0.0, 1.0))
        } else {
            (0.5, 0.5)
        };

        // calculate aspect ratios
        let width = src_width as f64;
        let height = src_height as f64;
        let image_ratio = width / height;
        let required_ration = dst_width as f64 / dst_height as f64;

        let crop_width;
        let crop_height;
        // figure out if the sides or top/bottom will be cropped off
        if (image_ratio - required_ration).abs() < f64::EPSILON {
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

        Self {
            left: crop_left,
            top: crop_top,
            width: crop_width,
            height: crop_height,
        }
    }
}

pub(crate) struct CroppedSrcImageView<'a, T: ImageView> {
    image_view: &'a T,
    crop_box: CropBox,
}

impl<'a, T: ImageView> CroppedSrcImageView<'a, T> {
    pub fn new(image_view: &'a T) -> Self {
        Self {
            image_view,
            crop_box: CropBox {
                left: 0.0,
                top: 0.0,
                width: image_view.width() as _,
                height: image_view.height() as _,
            },
        }
    }

    pub fn crop(image_view: &'a T, crop_box: CropBox) -> Result<Self, CropBoxError> {
        if crop_box.width < 0. || crop_box.height < 0. {
            return Err(CropBoxError::WidthOrHeightLessThanZero);
        }

        let img_width = image_view.width() as _;
        let img_height = image_view.height() as _;

        if crop_box.left >= img_width || crop_box.top >= img_height {
            return Err(CropBoxError::PositionIsOutOfImageBoundaries);
        }
        let right = crop_box.left + crop_box.width;
        let bottom = crop_box.top + crop_box.height;
        if right > img_width || bottom > img_height {
            return Err(CropBoxError::SizeIsOutOfImageBoundaries);
        }
        Ok(Self {
            image_view,
            crop_box,
        })
    }

    pub unsafe fn crop_unchecked(image_view: &'a T, crop_box: CropBox) -> Self {
        Self {
            image_view,
            crop_box,
        }
    }

    /// Returns a reference to the wrapped image view.
    #[inline]
    pub fn image_view(&self) -> &T {
        self.image_view
    }

    #[inline]
    pub fn crop_box(&self) -> CropBox {
        self.crop_box
    }
}
