//! Functions and structs for working with colorspace and gamma.
use num_traits::bounds::UpperBounded;
use num_traits::Zero;

use crate::pixels::{
    GetCount, InnerPixel, IntoPixelComponent, PixelComponent, PixelType, U16x2, U16x3, U16x4, U8x2,
    U8x3, U8x4, Values, U16, U8,
};
use crate::{
    try_pixel_type, ImageView, ImageViewMut, IntoImageView, IntoImageViewMut, MappingError,
    PixelTrait,
};

pub(crate) mod mappers;

trait FromF32 {
    fn from_f32(x: f32) -> Self;
}

impl FromF32 for u8 {
    fn from_f32(x: f32) -> Self {
        x as Self
    }
}

impl FromF32 for u16 {
    fn from_f32(x: f32) -> Self {
        x as Self
    }
}

struct MappingTable<Out: PixelComponent, const SIZE: usize>([Out; SIZE]);

impl<Out, const SIZE: usize> MappingTable<Out, SIZE>
where
    Out: PixelComponent + Zero + UpperBounded + FromF32 + Into<f32>,
{
    pub fn new<F>(map_func: &F) -> Self
    where
        F: Fn(f32) -> f32,
    {
        let mut table: [Out; SIZE] = [Out::zero(); SIZE];
        table.iter_mut().enumerate().for_each(|(input, output)| {
            let input_f32 = input as f32 / (SIZE - 1) as f32;
            *output = Out::from_f32((map_func(input_f32) * Out::max_value().into()).round());
        });
        Self(table)
    }

    fn map<In>(&self, src_buffer: &[In], dst_buffer: &mut [Out])
    where
        In: PixelComponent + Into<usize>,
    {
        for (&src, dst) in src_buffer.iter().zip(dst_buffer) {
            *dst = self.0[src.into()];
        }
    }

    fn map_inplace(&self, buffer: &mut [Out])
    where
        Out: Into<usize>,
    {
        for c in buffer.iter_mut() {
            let i: usize = (*c).into();
            *c = self.0[i];
        }
    }

    fn map_with_gaps<In>(&self, src_buffer: &[In], dst_buffer: &mut [Out], gap_step: usize)
    where
        In: IntoPixelComponent<Out> + Into<usize>,
    {
        for (i, (&src, dst)) in src_buffer.iter().zip(dst_buffer).enumerate() {
            if (i + 1) % gap_step != 0 {
                *dst = self.0[src.into()];
            } else {
                *dst = src.into_component();
            }
        }
    }

    fn map_with_gaps_inplace(&self, buffer: &mut [Out], gap_step: usize)
    where
        Out: Into<usize>,
    {
        for (i, c) in buffer.iter_mut().enumerate() {
            if (i + 1) % gap_step != 0 {
                let i: usize = (*c).into();
                *c = self.0[i];
            }
        }
    }

    pub fn map_image<S, D>(
        &self,
        src_image: &impl IntoImageView,
        dst_image: &mut impl IntoImageViewMut,
    ) -> Result<(), MappingError>
    where
        S: PixelTrait,
        <S as InnerPixel>::Component: PixelComponent<CountOfComponentValues = Values<SIZE>>
            + IntoPixelComponent<Out>
            + Into<usize>,
        D: PixelTrait<Component = Out, CountOfComponents = S::CountOfComponents>,
    {
        let (src_view, dst_view) =
            match (src_image.image_view::<S>(), dst_image.image_view_mut::<D>()) {
                (Some(src_view), Some(dst_view)) => (src_view, dst_view),
                _ => return Err(MappingError::UnsupportedCombinationOfImageTypes),
            };

        self.map_image_typed(src_view, dst_view);
        Ok(())
    }

    pub fn map_image_typed<S, D>(
        &self,
        src_view: impl ImageView<Pixel = S>,
        mut dst_view: impl ImageViewMut<Pixel = D>,
    ) where
        S: InnerPixel,
        <S as InnerPixel>::Component: PixelComponent<CountOfComponentValues = Values<SIZE>>
            + IntoPixelComponent<Out>
            + Into<usize>,
        D: InnerPixel<Component = Out, CountOfComponents = S::CountOfComponents>,
    {
        for (s_row, d_row) in src_view.iter_rows(0).zip(dst_view.iter_rows_mut(0)) {
            let s_comp = S::components(s_row);
            let d_comp = D::components_mut(d_row);
            match S::CountOfComponents::count() {
                2 => self.map_with_gaps(s_comp, d_comp, 2), // Don't map alpha channel
                4 => self.map_with_gaps(s_comp, d_comp, 4), // Don't map alpha channel
                _ => self.map(s_comp, d_comp),
            }
        }
    }

    pub fn map_image_inplace<S>(
        &self,
        image: &mut impl IntoImageViewMut,
    ) -> Result<(), MappingError>
    where
        Out: Into<usize>,
        S: PixelTrait<Component = Out>,
    {
        if let Some(image_view) = image.image_view_mut::<S>() {
            self.map_image_inplace_typed(image_view);
            Ok(())
        } else {
            Err(MappingError::UnsupportedCombinationOfImageTypes)
        }
    }

    pub fn map_image_inplace_typed<S>(&self, mut image_view: impl ImageViewMut<Pixel = S>)
    where
        Out: Into<usize>,
        S: InnerPixel<Component = Out>,
    {
        for row in image_view.iter_rows_mut(0) {
            let comp = S::components_mut(row);
            match S::CountOfComponents::count() {
                2 => self.map_with_gaps_inplace(comp, 2), // Don't map alpha channel
                4 => self.map_with_gaps_inplace(comp, 4), // Don't map alpha channel
                _ => self.map_inplace(comp),
            }
        }
    }
}

struct MappingTablesGroup {
    u8_u8: Box<MappingTable<u8, 256>>,
    u8_u16: Box<MappingTable<u16, 256>>,
    u16_u8: Box<MappingTable<u8, 65536>>,
    u16_u16: Box<MappingTable<u16, 65536>>,
}

/// Mapper of pixel's components.
///
/// This structure holds tables for mapping values of pixel's
/// components in forward and backward directions.
///
/// All pixel types except `I32` and `F32xN` are supported.
///
/// Source and destination images may have different bit depth of one
/// pixel component.
/// But count of components must be equal.
/// For example, you can convert `U8x3` image with sRGB colorspace into
/// `U16x3` image with linear colorspace.
///
/// Alpha channel from such pixel types as `U8x2`, `U8x4`, `U16x2` and `U16x4`
/// is not mapped with tables. This component is transformed into destination
/// component type with help of [IntoPixelComponent] trait.
pub struct PixelComponentMapper {
    forward_mapping_tables: MappingTablesGroup,
    backward_mapping_tables: MappingTablesGroup,
}

impl PixelComponentMapper {
    /// Create an instance of the structure by filling its tables with
    /// given functions.
    ///
    /// Each function takes one argument with the value of the pixel component
    /// converted into `f32` in the range `[0.0, 1.0]`.
    /// The return value must also be `f32` in the range `[0.0, 1.0]`.
    ///
    /// Example:
    /// ```
    /// # use fast_image_resize::PixelComponentMapper;
    /// #
    /// fn gamma_into_linear(input: f32) -> f32 {
    ///     input.powf(2.2)
    /// }
    ///
    /// fn linear_into_gamma(input: f32) -> f32 {
    ///     input.powf(1.0 / 2.2)
    /// }
    ///
    /// let gamma22_to_linear = PixelComponentMapper::new(
    ///     gamma_into_linear,
    ///     linear_into_gamma,
    /// );
    /// ```
    pub fn new<FF, BF>(forward_map_func: FF, backward_map_func: BF) -> Self
    where
        FF: Fn(f32) -> f32,
        BF: Fn(f32) -> f32,
    {
        Self {
            forward_mapping_tables: MappingTablesGroup {
                u8_u8: Box::new(MappingTable::new(&forward_map_func)),
                u8_u16: Box::new(MappingTable::new(&forward_map_func)),
                u16_u8: Box::new(MappingTable::new(&forward_map_func)),
                u16_u16: Box::new(MappingTable::new(&forward_map_func)),
            },
            backward_mapping_tables: MappingTablesGroup {
                u8_u8: Box::new(MappingTable::new(&backward_map_func)),
                u8_u16: Box::new(MappingTable::new(&backward_map_func)),
                u16_u8: Box::new(MappingTable::new(&backward_map_func)),
                u16_u16: Box::new(MappingTable::new(&backward_map_func)),
            },
        }
    }

    fn map(
        tables: &MappingTablesGroup,
        src_image: &impl IntoImageView,
        dst_image: &mut impl IntoImageViewMut,
    ) -> Result<(), MappingError> {
        let src_pixel_type = try_pixel_type(src_image)?;
        let dst_pixel_type = try_pixel_type(dst_image)?;

        if src_image.width() != dst_image.width() || src_image.height() != dst_image.height() {
            return Err(MappingError::DifferentDimensions);
        }

        use PixelType as PT;

        macro_rules! match_img {
            (
                $tables: ident,
                $(($p8: path, $pt8: tt, $p16: path, $pt16: tt),)*
            ) => {
                match (src_pixel_type, dst_pixel_type) {
                    $(
                        ($p8, $p8) => $tables.u8_u8.map_image::<$pt8, $pt8>(
                            src_image,
                            dst_image,
                        ),
                        ($p8, $p16) => $tables.u8_u16.map_image::<$pt8, $pt16>(
                            src_image,
                            dst_image,
                        ),
                        ($p16, $p8) => $tables.u16_u8.map_image::<$pt16, $pt8>(
                            src_image,
                            dst_image,
                        ),
                        ($p16, $p16) => $tables.u16_u16.map_image::<$pt16, $pt16>(
                            src_image,
                            dst_image,
                        ),
                    )*
                    _ => return Err(MappingError::UnsupportedCombinationOfImageTypes),
                }
            };
        }

        #[cfg(not(feature = "only_u8x4"))]
        {
            match_img!(
                tables,
                (PT::U8, U8, PT::U16, U16),
                (PT::U8x2, U8x2, PT::U16x2, U16x2),
                (PT::U8x3, U8x3, PT::U16x3, U16x3),
                (PT::U8x4, U8x4, PT::U16x4, U16x4),
            )
        }

        #[cfg(feature = "only_u8x4")]
        match (src_pixel_type, dst_pixel_type) {
            (PT::U8x4, PT::U8x4) => tables.u8_u8.map_image::<U8x4, U8x4>(src_image, dst_image),
            _ => return Err(MappingError::UnsupportedCombinationOfImageTypes),
        }
    }

    fn map_inplace(
        tables: &MappingTablesGroup,
        image: &mut impl IntoImageViewMut,
    ) -> Result<(), MappingError> {
        let pixel_type = try_pixel_type(image)?;

        use PixelType as PT;

        macro_rules! match_img {
            (
                $tables: ident, $image: ident,
                $(($p8: path, $pt8: tt, $p16: path, $pt16: tt),)*
            ) => {
                match pixel_type {
                    $(
                        $p8 => $tables.u8_u8.map_image_inplace::<$pt8>($image),
                        $p16 => $tables.u16_u16.map_image_inplace::<$pt16>($image),
                    )*
                    _ => return Err(MappingError::UnsupportedCombinationOfImageTypes),
                }
            };
        }

        #[cfg(not(feature = "only_u8x4"))]
        {
            match_img!(
                tables,
                image,
                (PT::U8, U8, PT::U16, U16),
                (PT::U8x2, U8x2, PT::U16x2, U16x2),
                (PT::U8x3, U8x3, PT::U16x3, U16x3),
                (PT::U8x4, U8x4, PT::U16x4, U16x4),
            )
        }

        #[cfg(feature = "only_u8x4")]
        match pixel_type {
            PT::U8x4 => tables.u8_u8.map_image_inplace::<U8x4>(image),
            _ => return Err(MappingError::UnsupportedCombinationOfImageTypes),
        }
    }

    /// Mapping in the forward direction of pixel's components of source image
    /// into corresponding components of destination image.
    pub fn forward_map(
        &self,
        src_image: &impl IntoImageView,
        dst_image: &mut impl IntoImageViewMut,
    ) -> Result<(), MappingError> {
        Self::map(&self.forward_mapping_tables, src_image, dst_image)
    }

    pub fn forward_map_inplace(
        &self,
        image: &mut impl IntoImageViewMut,
    ) -> Result<(), MappingError> {
        Self::map_inplace(&self.forward_mapping_tables, image)
    }

    /// Mapping in the backward direction of pixel's components of source image
    /// into corresponding components of destination image.
    pub fn backward_map(
        &self,
        src_image: &impl IntoImageView,
        dst_image: &mut impl IntoImageViewMut,
    ) -> Result<(), MappingError> {
        Self::map(&self.backward_mapping_tables, src_image, dst_image)
    }

    pub fn backward_map_inplace(
        &self,
        image: &mut impl IntoImageViewMut,
    ) -> Result<(), MappingError> {
        Self::map_inplace(&self.backward_mapping_tables, image)
    }
}
