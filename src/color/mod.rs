//! Functions and structs for working with colorspace and gamma.
use num_traits::bounds::UpperBounded;
use num_traits::Zero;

use crate::pixels::{GetCount, IntoPixelComponent, PixelComponent, PixelExt, Values};
use crate::{DynamicImageView, DynamicImageViewMut, MappingError};
use crate::{ImageView, ImageViewMut};

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

    pub fn map_image<S, D, In, CC>(&self, src_image: &ImageView<S>, dst_image: &mut ImageViewMut<D>)
    where
        In: PixelComponent<CountOfComponentValues = Values<SIZE>>
            + IntoPixelComponent<Out>
            + Into<usize>,
        CC: GetCount,
        S: PixelExt<Component = In, CountOfComponents = CC>,
        D: PixelExt<Component = Out, CountOfComponents = CC>,
    {
        for (s_row, d_row) in src_image.iter_rows(0).zip(dst_image.iter_rows_mut()) {
            let s_comp = S::components(s_row);
            let d_comp = D::components_mut(d_row);
            match CC::count() {
                2 => self.map_with_gaps(s_comp, d_comp, 2), // Don't map alpha channel
                4 => self.map_with_gaps(s_comp, d_comp, 4), // Don't map alpha channel
                _ => self.map(s_comp, d_comp),
            }
        }
    }

    pub fn map_image_inplace<S, CC>(&self, image: &mut ImageViewMut<S>)
    where
        CC: GetCount,
        S: PixelExt<Component = Out, CountOfComponents = CC>,
        Out: Into<usize>,
    {
        for row in image.iter_rows_mut() {
            let comp = S::components_mut(row);
            match CC::count() {
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
/// Supported all pixel types exclude `I32` and `F32`.
///
/// Source and destination images may have different bit depth of one pixel component.
/// But count of components must be equal.
/// For example, you may convert `U8x3` image with sRGB colorspace into
/// `U16x3` image with linear colorspace.
///
/// Alpha channel from such pixel types as `U8x2`, `U8x4`, `U16x2` and `U16x4`
/// not mapped with tables. This component transformed into destination
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
        src_image: &DynamicImageView,
        dst_image: &mut DynamicImageViewMut,
    ) -> Result<(), MappingError> {
        if src_image.width() != dst_image.width() || src_image.height() != dst_image.height() {
            return Err(MappingError::DifferentDimensions);
        }

        use DynamicImageView as DI;
        use DynamicImageViewMut as DIMut;

        macro_rules! match_img {
            (
                $tables: ident, $src_image: ident, $dst_image: ident,
                $(($p8: path, $p16: path, $p8_mut: path, $p16_mut: path),)*
            ) => {
                match ($src_image, $dst_image) {
                    $(
                        ($p8(src), $p8_mut(dst)) => $tables.u8_u8.map_image(src, dst),
                        ($p8(src), $p16_mut(dst)) => $tables.u8_u16.map_image(src, dst),
                        ($p16(src), $p8_mut(dst)) => $tables.u16_u8.map_image(src, dst),
                        ($p16(src), $p16_mut(dst)) => $tables.u16_u16.map_image(src, dst),
                    )*
                    _ => return Err(MappingError::UnsupportedCombinationOfImageTypes),
                }
            };
        }

        match_img!(
            tables,
            src_image,
            dst_image,
            (DI::U8, DI::U16, DIMut::U8, DIMut::U16),
            (DI::U8x2, DI::U16x2, DIMut::U8x2, DIMut::U16x2),
            (DI::U8x3, DI::U16x3, DIMut::U8x3, DIMut::U16x3),
            (DI::U8x4, DI::U16x4, DIMut::U8x4, DIMut::U16x4),
        );
        Ok(())
    }

    fn map_inplace(
        tables: &MappingTablesGroup,
        image: &mut DynamicImageViewMut,
    ) -> Result<(), MappingError> {
        use DynamicImageViewMut as DIMut;

        macro_rules! match_img {
            (
                $tables: ident, $image: ident,
                $(($p8_mut: path, $p16_mut: path),)*
            ) => {
                match $image {
                    $(
                        $p8_mut(img) => $tables.u8_u8.map_image_inplace(img),
                        $p16_mut(img) => $tables.u16_u16.map_image_inplace(img),
                    )*
                    _ => return Err(MappingError::UnsupportedCombinationOfImageTypes),
                }
            };
        }

        match_img!(
            tables,
            image,
            (DIMut::U8, DIMut::U16),
            (DIMut::U8x2, DIMut::U16x2),
            (DIMut::U8x3, DIMut::U16x3),
            (DIMut::U8x4, DIMut::U16x4),
        );
        Ok(())
    }

    /// Mapping in the forward direction of pixel's components of source image
    /// into corresponding components of destination image.
    pub fn forward_map(
        &self,
        src_image: &DynamicImageView,
        dst_image: &mut DynamicImageViewMut,
    ) -> Result<(), MappingError> {
        Self::map(&self.forward_mapping_tables, src_image, dst_image)
    }

    pub fn forward_map_inplace(&self, image: &mut DynamicImageViewMut) -> Result<(), MappingError> {
        Self::map_inplace(&self.forward_mapping_tables, image)
    }

    /// Mapping in the backward direction of pixel's components of source image
    /// into corresponding components of destination image.
    pub fn backward_map(
        &self,
        src_image: &DynamicImageView,
        dst_image: &mut DynamicImageViewMut,
    ) -> Result<(), MappingError> {
        Self::map(&self.backward_mapping_tables, src_image, dst_image)
    }

    pub fn backward_map_inplace(
        &self,
        image: &mut DynamicImageViewMut,
    ) -> Result<(), MappingError> {
        Self::map_inplace(&self.backward_mapping_tables, image)
    }
}
