use std::num::NonZeroU32;

use crate::image_view::change_type_of_pixel_components;
use crate::pixels::{U16x2, U16x3, U16x4, U8x2, U8x3, U8x4, F32, I32, U16, U8};
use crate::{CropBox, CropBoxError, ImageView, ImageViewMut, MappingError, PixelType};

/// An immutable view of image data used by resizer as source image.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum DynamicImageView<'a> {
    U8(ImageView<'a, U8>),
    U8x2(ImageView<'a, U8x2>),
    U8x3(ImageView<'a, U8x3>),
    U8x4(ImageView<'a, U8x4>),
    U16(ImageView<'a, U16>),
    U16x2(ImageView<'a, U16x2>),
    U16x3(ImageView<'a, U16x3>),
    U16x4(ImageView<'a, U16x4>),
    I32(ImageView<'a, I32>),
    F32(ImageView<'a, F32>),
}

/// A mutable view of image data used by resizer as destination image.
#[derive(Debug)]
#[non_exhaustive]
pub enum DynamicImageViewMut<'a> {
    U8(ImageViewMut<'a, U8>),
    U8x2(ImageViewMut<'a, U8x2>),
    U8x3(ImageViewMut<'a, U8x3>),
    U8x4(ImageViewMut<'a, U8x4>),
    U16(ImageViewMut<'a, U16>),
    U16x2(ImageViewMut<'a, U16x2>),
    U16x3(ImageViewMut<'a, U16x3>),
    U16x4(ImageViewMut<'a, U16x4>),
    I32(ImageViewMut<'a, I32>),
    F32(ImageViewMut<'a, F32>),
}

macro_rules! dynamic_map(
        ($dyn_image: expr, $image: pat => $action: expr) => ({
            use DynamicImageView::*;
            match $dyn_image {
                U8($image) => U8($action),
                U8x2($image) => U8x2($action),
                U8x3($image) => U8x3($action),
                U8x4($image) => U8x4($action),
                U16($image) => U16($action),
                U16x2($image) => U16x2($action),
                U16x3($image) => U16x3($action),
                U16x4($image) => U16x4($action),
                I32($image) => I32($action),
                F32($image) => F32($action),
            }
        });

        ($dyn_image: expr, |$image: pat_param| $action: expr) => (
            match $dyn_image {
                DynamicImageView::U8($image) => $action,
                DynamicImageView::U8x2($image) => $action,
                DynamicImageView::U8x3($image) => $action,
                DynamicImageView::U8x4($image) => $action,
                DynamicImageView::U16($image) => $action,
                DynamicImageView::U16x2($image) => $action,
                DynamicImageView::U16x3($image) => $action,
                DynamicImageView::U16x4($image) => $action,
                DynamicImageView::I32($image) => $action,
                DynamicImageView::F32($image) => $action,
            }
        );
);

macro_rules! dynamic_mut_map (
        ($dyn_image: expr, $image: pat => $action: expr) => ({
            use DynamicImageViewMut::*;
            match $dyn_image {
                U8($image) => U8($action),
                U8x2($image) => U8x2($action),
                U8x3($image) => U8x3($action),
                U8x4($image) => U8x4($action),
                U16($image) => U16($action),
                U16x2($image) => U16x2($action),
                U16x3($image) => U16x3($action),
                U16x4($image) => U16x4($action),
                I32($image) => I32($action),
                F32($image) => F32($action),
            }
        });

        ($dyn_image: expr, |$image: pat_param| $action: expr) => (
            match $dyn_image {
                DynamicImageViewMut::U8($image) => $action,
                DynamicImageViewMut::U8x2($image) => $action,
                DynamicImageViewMut::U8x3($image) => $action,
                DynamicImageViewMut::U8x4($image) => $action,
                DynamicImageViewMut::U16($image) => $action,
                DynamicImageViewMut::U16x2($image) => $action,
                DynamicImageViewMut::U16x3($image) => $action,
                DynamicImageViewMut::U16x4($image) => $action,
                DynamicImageViewMut::I32($image) => $action,
                DynamicImageViewMut::F32($image) => $action,
            }
        );
);

impl<'a> DynamicImageView<'a> {
    pub fn width(&self) -> NonZeroU32 {
        dynamic_map!(self, |typed_image| typed_image.width())
    }

    pub fn height(&self) -> NonZeroU32 {
        dynamic_map!(self, |typed_image| typed_image.height())
    }

    pub fn pixel_type(&self) -> PixelType {
        dynamic_map!(self, |typed_image| typed_image.pixel_type())
    }

    pub fn crop_box(&self) -> CropBox {
        dynamic_map!(self, |typed_image| typed_image.crop_box())
    }

    pub fn set_crop_box(&mut self, crop_box: CropBox) -> Result<(), CropBoxError> {
        dynamic_map!(self, |typed_image| typed_image.set_crop_box(crop_box))
    }

    pub fn set_crop_box_to_fit_dst_size(
        &mut self,
        dst_width: NonZeroU32,
        dst_height: NonZeroU32,
        centering: Option<(f32, f32)>,
    ) {
        dynamic_map!(self, |typed_image| typed_image
            .set_crop_box_to_fit_dst_size(dst_width, dst_height, centering))
    }
}

impl<'a> DynamicImageViewMut<'a> {
    pub fn width(&self) -> NonZeroU32 {
        dynamic_mut_map!(self, |typed_image| typed_image.width())
    }

    pub fn height(&self) -> NonZeroU32 {
        dynamic_mut_map!(self, |typed_image| typed_image.height())
    }

    pub fn pixel_type(&self) -> PixelType {
        dynamic_mut_map!(self, |typed_image| typed_image.pixel_type())
    }

    /// Create cropped version of the view.
    pub fn crop(self, crop_box: CropBox) -> Result<Self, CropBoxError> {
        Ok(dynamic_mut_map!(self, typed_image => typed_image.crop(crop_box)?))
    }
}

macro_rules! from_typed {
    ($pixel_type: ty, $enum: expr, $enum_mut: expr) => {
        impl<'a> From<ImageView<'a, $pixel_type>> for DynamicImageView<'a> {
            fn from(view: ImageView<'a, $pixel_type>) -> Self {
                $enum(view)
            }
        }

        impl<'a> From<ImageViewMut<'a, $pixel_type>> for DynamicImageViewMut<'a> {
            fn from(view: ImageViewMut<'a, $pixel_type>) -> Self {
                $enum_mut(view)
            }
        }
    };
}

from_typed!(U8, DynamicImageView::U8, DynamicImageViewMut::U8);
from_typed!(U8x2, DynamicImageView::U8x2, DynamicImageViewMut::U8x2);
from_typed!(U8x3, DynamicImageView::U8x3, DynamicImageViewMut::U8x3);
from_typed!(U8x4, DynamicImageView::U8x4, DynamicImageViewMut::U8x4);
from_typed!(U16, DynamicImageView::U16, DynamicImageViewMut::U16);
from_typed!(U16x2, DynamicImageView::U16x2, DynamicImageViewMut::U16x2);
from_typed!(U16x3, DynamicImageView::U16x3, DynamicImageViewMut::U16x3);
from_typed!(U16x4, DynamicImageView::U16x4, DynamicImageViewMut::U16x4);
from_typed!(I32, DynamicImageView::I32, DynamicImageViewMut::I32);
from_typed!(F32, DynamicImageView::F32, DynamicImageViewMut::F32);

pub fn change_type_of_pixel_components_dyn(
    src_image: &DynamicImageView,
    dst_image: &mut DynamicImageViewMut,
) -> Result<(), MappingError> {
    macro_rules! map {
        ($value:expr, $(($src_first:path, $src_second:path, $dst_first:path, $dst_second:path)),*) => {
            match $value {
                $(
                    ($src_first(src), $dst_first(dst)) => {
                        change_type_of_pixel_components(src, dst)?;
                    }
                    ($src_first(src), $dst_second(dst)) => {
                        change_type_of_pixel_components(src, dst)?;
                    }
                    ($src_second(src), $dst_first(dst)) => {
                        change_type_of_pixel_components(src, dst)?;
                    }
                    ($src_second(src), $dst_second(dst)) => {
                        change_type_of_pixel_components(src, dst)?;
                    }
                )*
                _ => return Err(MappingError::UnsupportedCombinationOfImageTypes),
            }
        }
    }

    use DynamicImageView as IV;
    use DynamicImageViewMut as IVMut;

    map!(
        (src_image, dst_image),
        (IV::U8, IV::U16, IVMut::U8, IVMut::U16),
        (IV::U8x2, IV::U16x2, IVMut::U8x2, IVMut::U16x2),
        (IV::U8x3, IV::U16x3, IVMut::U8x3, IVMut::U16x3),
        (IV::U8x4, IV::U16x4, IVMut::U8x4, IVMut::U16x4)
    );
    Ok(())
}

impl<'a> From<DynamicImageViewMut<'a>> for DynamicImageView<'a> {
    fn from(dyn_view: DynamicImageViewMut<'a>) -> Self {
        use DynamicImageViewMut::*;
        match dyn_view {
            U8(typed_view) => DynamicImageView::U8(typed_view.into()),
            U8x2(typed_view) => DynamicImageView::U8x2(typed_view.into()),
            U8x3(typed_view) => DynamicImageView::U8x3(typed_view.into()),
            U8x4(typed_view) => DynamicImageView::U8x4(typed_view.into()),
            U16(typed_view) => DynamicImageView::U16(typed_view.into()),
            U16x2(typed_view) => DynamicImageView::U16x2(typed_view.into()),
            U16x3(typed_view) => DynamicImageView::U16x3(typed_view.into()),
            U16x4(typed_view) => DynamicImageView::U16x4(typed_view.into()),
            I32(typed_view) => DynamicImageView::I32(typed_view.into()),
            F32(typed_view) => DynamicImageView::F32(typed_view.into()),
        }
    }
}
