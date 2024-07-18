use crate::pixels::{
    F32x2, F32x3, F32x4, InnerPixel, IntoPixelComponent, U16x2, U16x3, U16x4, U8x2, U8x3, U8x4,
    F32, I32, U16, U8,
};
use crate::{
    try_pixel_type, DifferentDimensionsError, ImageView, ImageViewMut, IntoImageView,
    IntoImageViewMut, MappingError, PixelTrait, PixelType,
};

pub fn change_type_of_pixel_components(
    src_image: &impl IntoImageView,
    dst_image: &mut impl IntoImageViewMut,
) -> Result<(), MappingError> {
    macro_rules! map_dst {
        (
            $src_pt:ty, $dst_type:expr,
            $(($dst_enum:path, $dst_pt:ty)),*
        ) => {
            match $dst_type {
                $(
                    $dst_enum =>
                        change_components_type::<$src_pt, $dst_pt>(src_image, dst_image),
                )*
                _ => Err(MappingError::UnsupportedCombinationOfImageTypes),
            }
        }
    }

    let src_pixel_type = try_pixel_type(src_image)?;
    let dst_pixel_type = try_pixel_type(dst_image)?;

    use PixelType as PT;

    #[cfg(not(feature = "only_u8x4"))]
    match src_pixel_type {
        PixelType::U8 => map_dst!(
            U8,
            dst_pixel_type,
            (PT::U8, U8),
            (PT::U16, U16),
            (PT::I32, I32),
            (PT::F32, F32)
        ),
        PixelType::U8x2 => map_dst!(
            U8x2,
            dst_pixel_type,
            (PT::U8x2, U8x2),
            (PT::U16x2, U16x2),
            (PT::F32x2, F32x2)
        ),
        PixelType::U8x3 => map_dst!(
            U8x3,
            dst_pixel_type,
            (PT::U8x3, U8x3),
            (PT::U16x3, U16x3),
            (PT::F32x3, F32x3)
        ),
        PixelType::U8x4 => map_dst!(
            U8x4,
            dst_pixel_type,
            (PT::U8x4, U8x4),
            (PT::U16x4, U16x4),
            (PT::F32x4, F32x4)
        ),
        PixelType::U16 => map_dst!(
            U16,
            dst_pixel_type,
            (PT::U8, U8),
            (PT::U16, U16),
            (PT::I32, I32),
            (PT::F32, F32)
        ),
        PixelType::U16x2 => map_dst!(
            U16x2,
            dst_pixel_type,
            (PT::U8x2, U8x2),
            (PT::U16x2, U16x2),
            (PT::F32x2, F32x2)
        ),
        PixelType::U16x3 => map_dst!(
            U16x3,
            dst_pixel_type,
            (PT::U8x3, U8x3),
            (PT::U16x3, U16x3),
            (PT::F32x3, F32x3)
        ),
        PixelType::U16x4 => map_dst!(
            U16x4,
            dst_pixel_type,
            (PT::U8x4, U8x4),
            (PT::U16x4, U16x4),
            (PT::F32x4, F32x4)
        ),
        PixelType::I32 => map_dst!(
            I32,
            dst_pixel_type,
            (PT::U8, U8),
            (PT::U16, U16),
            (PT::I32, I32),
            (PT::F32, F32)
        ),
        PixelType::F32 => map_dst!(
            F32,
            dst_pixel_type,
            (PT::U8, U8),
            (PT::U16, U16),
            (PT::I32, I32),
            (PT::F32, F32)
        ),
        PixelType::F32x2 => map_dst!(
            F32x2,
            dst_pixel_type,
            (PT::U8x2, U8x2),
            (PT::U16x2, U16x2),
            (PT::F32x2, F32x2)
        ),
        PixelType::F32x3 => map_dst!(
            F32x3,
            dst_pixel_type,
            (PT::U8x3, U8x3),
            (PT::U16x3, U16x3),
            (PT::F32x3, F32x3)
        ),
        PixelType::F32x4 => map_dst!(
            F32x4,
            dst_pixel_type,
            (PT::U8x4, U8x4),
            (PT::U16x4, U16x4),
            (PT::F32x4, F32x4)
        ),
    }

    #[cfg(feature = "only_u8x4")]
    match src_pixel_type {
        PixelType::U8x4 => map_dst!(U8x4, dst_pixel_type, (PT::U8x4, U8x4)),
        _ => Err(MappingError::UnsupportedCombinationOfImageTypes),
    }
}

#[inline(always)]
fn change_components_type<S, D>(
    src_image: &impl IntoImageView,
    dst_image: &mut impl IntoImageViewMut,
) -> Result<(), MappingError>
where
    S: PixelTrait,
    D: PixelTrait<CountOfComponents = S::CountOfComponents>,
    <S as InnerPixel>::Component: IntoPixelComponent<<D as InnerPixel>::Component>,
{
    match (src_image.image_view::<S>(), dst_image.image_view_mut::<D>()) {
        (Some(src_view), Some(mut dst_view)) => {
            change_type_of_pixel_components_typed(&src_view, &mut dst_view).map_err(|e| e.into())
        }
        _ => Err(MappingError::UnsupportedCombinationOfImageTypes),
    }
}

pub fn change_type_of_pixel_components_typed<S, D>(
    src_image: &impl ImageView<Pixel = S>,
    dst_image: &mut impl ImageViewMut<Pixel = D>,
) -> Result<(), DifferentDimensionsError>
where
    S: InnerPixel,
    D: InnerPixel<CountOfComponents = S::CountOfComponents>,
    <S as InnerPixel>::Component: IntoPixelComponent<<D as InnerPixel>::Component>,
{
    if src_image.width() != dst_image.width() || src_image.height() != dst_image.height() {
        return Err(DifferentDimensionsError);
    }

    for (s_row, d_row) in src_image.iter_rows(0).zip(dst_image.iter_rows_mut(0)) {
        let s_components = S::components(s_row);
        let d_components = D::components_mut(d_row);
        for (&s_comp, d_comp) in s_components.iter().zip(d_components) {
            *d_comp = s_comp.into_component();
        }
    }
    Ok(())
}
