//! Functions for working with colorspace and gamma.
//!
//! Supported all pixel types exclude `I32` and `F32`.
//!
//! Source and destination images may have different bit depth of one pixel component.
//! But count of components must be equal.
//! For example, you may convert `U8x3` image with sRGB colorspace into
//! `U16x3` image with linear colorspace.
use crate::pixels::{GetCount, Pixel, PixelComponent, PixelComponentInto, Values};
use crate::typed_image_view::{TypedImageView, TypedImageViewMut};

pub mod gamma;
pub mod srgb;

struct MappingTable<Out: PixelComponent, const N: usize>([Out; N]);

impl<Out, const N: usize> MappingTable<Out, N>
where
    Out: PixelComponent,
{
    fn map<In>(&self, src_buffer: &[In], dst_buffer: &mut [Out])
    where
        In: PixelComponent + Into<usize>,
    {
        for (&src, dst) in src_buffer.iter().zip(dst_buffer) {
            *dst = self.0[src.into()];
        }
    }

    fn map_with_gaps<In>(&self, src_buffer: &[In], dst_buffer: &mut [Out], gap_step: usize)
    where
        In: PixelComponentInto<Out> + Into<usize>,
    {
        for (i, (&src, dst)) in src_buffer.iter().zip(dst_buffer).enumerate() {
            if (i + 1) % gap_step != 0 {
                *dst = self.0[src.into()];
            } else {
                *dst = src.into_component();
            }
        }
    }

    pub fn map_typed_image<S, D, CC, In>(
        &self,
        src_image: TypedImageView<S>,
        mut dst_image: TypedImageViewMut<D>,
    ) where
        In: PixelComponentInto<Out> + Into<usize>,
        CC: GetCount,
        S: Pixel<
            Component = In,
            ComponentsCount = CC, // Count of source pixel's components
            ComponentCountOfValues = Values<N>, // Total count of values of one source pixel's component
        >,
        S::Component: Into<usize>,
        D: Pixel<
            Component = Out,
            ComponentsCount = CC, // Count of destination pixel's components
        >,
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
}
