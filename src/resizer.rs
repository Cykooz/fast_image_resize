use std::num::NonZeroU32;

use crate::convolution::{self, Convolution, FilterType};
use crate::image::InnerImage;
use crate::pixels::PixelExt;
use crate::{
    DifferentTypesOfPixelsError, DynamicImageView, DynamicImageViewMut, ImageView, ImageViewMut,
};

/// SIMD extension of CPU.
/// Specific variants depends from target architecture.
/// Look at source code to see all available variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuExtensions {
    None,
    #[cfg(target_arch = "x86_64")]
    /// SIMD extension of x86_64 architecture
    Sse4_1,
    #[cfg(target_arch = "x86_64")]
    /// SIMD extension of x86_64 architecture
    Avx2,
    #[cfg(target_arch = "aarch64")]
    /// SIMD extension of Arm64 architecture
    Neon,
    #[cfg(target_arch = "wasm32")]
    /// SIMD extension of Wasm32 architecture
    Simd128,
}

impl CpuExtensions {
    /// Returns `true` if your CPU support the extension.
    pub fn is_supported(&self) -> bool {
        match self {
            #[cfg(target_arch = "x86_64")]
            Self::Avx2 => is_x86_feature_detected!("avx2"),
            #[cfg(target_arch = "x86_64")]
            Self::Sse4_1 => is_x86_feature_detected!("sse4.1"),
            #[cfg(target_arch = "aarch64")]
            Self::Neon => true,
            #[cfg(target_arch = "wasm32")]
            Self::Simd128 => true,
            Self::None => true,
        }
    }
}

impl Default for CpuExtensions {
    #[cfg(target_arch = "x86_64")]
    fn default() -> Self {
        if is_x86_feature_detected!("avx2") {
            Self::Avx2
        } else if is_x86_feature_detected!("sse4.1") {
            Self::Sse4_1
        } else {
            Self::None
        }
    }

    #[cfg(target_arch = "aarch64")]
    fn default() -> Self {
        use std::arch::is_aarch64_feature_detected;
        if is_aarch64_feature_detected!("neon") {
            Self::Neon
        } else {
            Self::None
        }
    }
    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        Self::Simd128
    }

    #[cfg(not(any(
        target_arch = "x86_64",
        target_arch = "aarch64",
        target_arch = "wasm32"
    )))]
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ResizeAlg {
    Nearest,
    Convolution(FilterType),
    SuperSampling(FilterType, u8),
}

impl Default for ResizeAlg {
    fn default() -> Self {
        Self::Convolution(FilterType::Lanczos3)
    }
}

/// Methods of this structure used to resize images.
#[derive(Default, Debug, Clone)]
pub struct Resizer {
    pub algorithm: ResizeAlg,
    cpu_extensions: CpuExtensions,
    convolution_buffer: Vec<u8>,
    super_sampling_buffer: Vec<u8>,
}

impl Resizer {
    /// Creates instance of `Resizer`
    ///
    /// By default, instance of `Resizer` created with best CPU-extensions provided by your CPU.
    /// You can change this by use method [Resizer::set_cpu_extensions].
    pub fn new(algorithm: ResizeAlg) -> Self {
        Self {
            algorithm,
            ..Default::default()
        }
    }

    /// Resize source image to the size of destination image and save
    /// the result to the latter's pixel buffer.
    ///
    /// This method doesn't multiply source image and doesn't divide
    /// destination image by alpha channel.
    /// You must use [MulDiv](crate::MulDiv) for these actions.
    pub fn resize(
        &mut self,
        src_image: &DynamicImageView,
        dst_image: &mut DynamicImageViewMut,
    ) -> Result<(), DifferentTypesOfPixelsError> {
        match (src_image, dst_image) {
            (DynamicImageView::U8(src), DynamicImageViewMut::U8(dst)) => {
                self.resize_inner(src, dst);
            }
            (DynamicImageView::U8x2(src), DynamicImageViewMut::U8x2(dst)) => {
                self.resize_inner(src, dst);
            }
            (DynamicImageView::U8x3(src), DynamicImageViewMut::U8x3(dst)) => {
                self.resize_inner(src, dst);
            }
            (DynamicImageView::U8x4(src), DynamicImageViewMut::U8x4(dst)) => {
                self.resize_inner(src, dst);
            }
            (DynamicImageView::U16(src), DynamicImageViewMut::U16(dst)) => {
                self.resize_inner(src, dst);
            }
            (DynamicImageView::U16x2(src), DynamicImageViewMut::U16x2(dst)) => {
                self.resize_inner(src, dst);
            }
            (DynamicImageView::U16x3(src), DynamicImageViewMut::U16x3(dst)) => {
                self.resize_inner(src, dst);
            }
            (DynamicImageView::U16x4(src), DynamicImageViewMut::U16x4(dst)) => {
                self.resize_inner(src, dst);
            }
            (DynamicImageView::I32(src), DynamicImageViewMut::I32(dst)) => {
                self.resize_inner(src, dst);
            }
            (DynamicImageView::F32(src), DynamicImageViewMut::F32(dst)) => {
                self.resize_inner(src, dst);
            }
            _ => {
                return Err(DifferentTypesOfPixelsError);
            }
        }
        Ok(())
    }

    fn resize_inner<P>(&mut self, src_image: &ImageView<P>, dst_image: &mut ImageViewMut<P>)
    where
        P: Convolution,
    {
        if dst_image.copy_from_view(src_image).is_ok() {
            // If `copy_from_view()` has returned `Ok` then
            // the size of the destination image is equal to
            // the size of the cropped source image.
            return;
        }
        match self.algorithm {
            ResizeAlg::Nearest => resample_nearest(src_image, dst_image),
            ResizeAlg::Convolution(filter_type) => {
                let convolution_buffer = &mut self.convolution_buffer;
                resample_convolution(
                    src_image,
                    dst_image,
                    filter_type,
                    self.cpu_extensions,
                    convolution_buffer,
                )
            }
            ResizeAlg::SuperSampling(filter_type, multiplicity) => {
                let convolution_buffer = &mut self.convolution_buffer;
                let super_sampling_buffer = &mut self.super_sampling_buffer;
                resample_super_sampling(
                    src_image,
                    dst_image,
                    filter_type,
                    multiplicity,
                    self.cpu_extensions,
                    super_sampling_buffer,
                    convolution_buffer,
                )
            }
        }
    }

    /// Returns the size of internal buffers used to store the results of
    /// intermediate resizing steps.
    pub fn size_of_internal_buffers(&self) -> usize {
        (self.convolution_buffer.capacity() + self.super_sampling_buffer.capacity())
            * std::mem::size_of::<u8>()
    }

    /// Deallocates the internal buffers used to store the results of
    /// intermediate resizing steps.
    pub fn reset_internal_buffers(&mut self) {
        if self.convolution_buffer.capacity() > 0 {
            self.convolution_buffer = Vec::new();
        }
        if self.super_sampling_buffer.capacity() > 0 {
            self.super_sampling_buffer = Vec::new();
        }
    }

    #[inline(always)]
    pub fn cpu_extensions(&self) -> CpuExtensions {
        self.cpu_extensions
    }

    /// # Safety
    /// This is unsafe because this method allows you to set a CPU-extensions
    /// that is not actually supported by your CPU.
    pub unsafe fn set_cpu_extensions(&mut self, extensions: CpuExtensions) {
        self.cpu_extensions = extensions;
    }
}

/// Create inner image container from part of given buffer.
/// Buffer may be expanded if it size is less than required for image.
fn get_temp_image_from_buffer<P: PixelExt>(
    buffer: &mut Vec<u8>,
    width: NonZeroU32,
    height: NonZeroU32,
) -> InnerImage<P> {
    let pixels_count = (width.get() * height.get()) as usize;
    // Add pixel size as gap for alignment of resulted buffer.
    let buf_size = pixels_count * P::size() + P::size();
    if buffer.len() < buf_size {
        buffer.resize(buf_size, 0);
    }
    let pixels = unsafe { buffer.align_to_mut::<P>().1 };
    InnerImage::new(width, height, &mut pixels[0..pixels_count])
}

fn resample_nearest<P>(src_image: &ImageView<P>, dst_image: &mut ImageViewMut<P>)
where
    P: PixelExt,
{
    let crop_box = src_image.crop_box();
    let dst_width = dst_image.width().get();
    let x_scale = crop_box.width.get() as f64 / dst_width as f64;
    let y_scale = crop_box.height.get() as f64 / dst_image.height().get() as f64;

    // Pretabulate horizontal pixel positions
    let x_in_start = crop_box.left as f64 + x_scale * 0.5;
    let max_src_x = src_image.width().get() as usize;
    let x_in_tab: Vec<usize> = (0..dst_width)
        .map(|x| ((x_in_start + x_scale * x as f64) as usize).min(max_src_x))
        .collect();

    let y_in_start = crop_box.top as f64 + y_scale * 0.5;

    let src_rows =
        src_image.iter_rows_with_step(y_in_start, y_scale, dst_image.height().get() as usize);
    let dst_rows = dst_image.iter_rows_mut();
    for (out_row, in_row) in dst_rows.zip(src_rows) {
        for (&x_in, out_pixel) in x_in_tab.iter().zip(out_row.iter_mut()) {
            // Safety of value of x_in guaranteed by algorithm of creating of x_in_tab
            *out_pixel = unsafe { *in_row.get_unchecked(x_in) };
        }
    }
}

fn resample_convolution<P>(
    src_image: &ImageView<P>,
    dst_image: &mut ImageViewMut<P>,
    filter_type: FilterType,
    cpu_extensions: CpuExtensions,
    temp_buffer: &mut Vec<u8>,
) where
    P: Convolution,
{
    let crop_box = src_image.crop_box();
    let dst_width = dst_image.width();
    let dst_height = dst_image.height();
    let (filter_fn, filter_support) = convolution::get_filter_func(filter_type);

    let need_horizontal = dst_width != crop_box.width;
    let horiz_coeffs = need_horizontal.then(|| {
        test_log!("compute horizontal convolution coefficients");
        convolution::precompute_coefficients(
            src_image.width(),
            crop_box.left as f64,
            crop_box.left as f64 + crop_box.width.get() as f64,
            dst_width,
            filter_fn,
            filter_support,
        )
    });

    let need_vertical = dst_height != crop_box.height;
    let vert_coeffs = need_vertical.then(|| {
        test_log!("compute vertical convolution coefficients");
        convolution::precompute_coefficients(
            src_image.height(),
            crop_box.top as f64,
            crop_box.top as f64 + crop_box.height.get() as f64,
            dst_height,
            filter_fn,
            filter_support,
        )
    });

    match (horiz_coeffs, vert_coeffs) {
        (Some(horiz_coeffs), Some(mut vert_coeffs)) => {
            let y_first = vert_coeffs.bounds[0].start;
            // Last used row in the source image
            let last_y_bound = vert_coeffs.bounds.last().unwrap();
            let y_last = last_y_bound.start + last_y_bound.size;
            let temp_height = NonZeroU32::new(y_last - y_first).unwrap();
            let mut temp_image = get_temp_image_from_buffer(temp_buffer, dst_width, temp_height);
            let mut tmp_dst_view = temp_image.dst_view();
            P::horiz_convolution(
                src_image,
                &mut tmp_dst_view,
                y_first,
                horiz_coeffs,
                cpu_extensions,
            );

            // Shift bounds for vertical pass
            vert_coeffs
                .bounds
                .iter_mut()
                .for_each(|b| b.start -= y_first);
            P::vert_convolution(
                &tmp_dst_view.into(),
                dst_image,
                0,
                vert_coeffs,
                cpu_extensions,
            );
        }
        (Some(horiz_coeffs), None) => {
            P::horiz_convolution(
                src_image,
                dst_image,
                crop_box.top,
                horiz_coeffs,
                cpu_extensions,
            );
        }
        (None, Some(vert_coeffs)) => {
            P::vert_convolution(
                src_image,
                dst_image,
                crop_box.left,
                vert_coeffs,
                cpu_extensions,
            );
        }
        _ => {}
    }
}

fn resample_super_sampling<P>(
    src_image: &ImageView<P>,
    dst_image: &mut ImageViewMut<P>,
    filter_type: FilterType,
    multiplicity: u8,
    cpu_extensions: CpuExtensions,
    temp_buffer: &mut Vec<u8>,
    convolution_temp_buffer: &mut Vec<u8>,
) where
    P: Convolution,
{
    let crop_box = src_image.crop_box();
    let dst_width = dst_image.width().get();
    let dst_height = dst_image.height().get();
    let width_scale = crop_box.width.get() as f32 / dst_width as f32;
    let height_scale = crop_box.height.get() as f32 / dst_height as f32;
    // It makes sense to resize the image in two steps only if the image
    // size is greater than the required size by multiplicity times.
    let factor = width_scale.min(height_scale) / multiplicity as f32;
    if factor > 1.2 {
        // First step is resizing the source image by fastest algorithm.
        // The temporary image will be about ``multiplicity`` times larger
        // than required.
        let tmp_width =
            NonZeroU32::new((crop_box.width.get() as f32 / factor).round() as u32).unwrap();
        let tmp_height =
            NonZeroU32::new((crop_box.height.get() as f32 / factor).round() as u32).unwrap();

        let mut tmp_img = get_temp_image_from_buffer(temp_buffer, tmp_width, tmp_height);
        resample_nearest(src_image, &mut tmp_img.dst_view());
        // Second step is resizing the temporary image with a convolution.
        resample_convolution(
            &tmp_img.src_view(),
            dst_image,
            filter_type,
            cpu_extensions,
            convolution_temp_buffer,
        );
    } else {
        // There is no point in doing the resizing in two steps.
        // We immediately resize the original image with a convolution.
        resample_convolution(
            src_image,
            dst_image,
            filter_type,
            cpu_extensions,
            convolution_temp_buffer,
        );
    }
}
