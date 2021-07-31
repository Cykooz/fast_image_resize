use std::num::NonZeroU32;

use crate::convolution::{self, Convolution, FilterType};
use crate::image_data::ImageData;
use crate::image_view::{DstImageView, PixelType, SrcImageView};

#[derive(Debug, Clone, Copy)]
pub enum CpuExtensions {
    None,
    Sse2,
    Sse4_1,
    Avx2,
}

impl Default for CpuExtensions {
    fn default() -> Self {
        if is_x86_feature_detected!("avx2") {
            Self::Avx2
        } else if is_x86_feature_detected!("sse4.1") {
            Self::Sse4_1
        } else if is_x86_feature_detected!("sse2") {
            Self::Sse2
        } else {
            Self::None
        }
    }
}

impl CpuExtensions {
    #[inline]
    fn get_resampler(&self, pixel_type: PixelType) -> &dyn Convolution {
        match pixel_type {
            PixelType::U8x4 => match self {
                Self::Sse4_1 => &convolution::Sse4U8x4,
                Self::Avx2 => &convolution::Avx2U8x4,
                _ => &convolution::NativeU8x4,
            },
            PixelType::I32 => &convolution::NativeI32,
            PixelType::F32 => &convolution::NativeF32,
        }
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
    convolution_buffer: Vec<u32>,
    super_sampling_buffer: Vec<u32>,
}

impl Resizer {
    /// Creates instance of `Resizer`
    ///
    /// By default instance of `Resizer` created with best CPU-extensions provided by your CPU.
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
    /// You must use [MulDiv](crate::MulDiv) for this actions.
    pub fn resize(&mut self, src_image: &SrcImageView, dst_image: &mut DstImageView) {
        match self.algorithm {
            ResizeAlg::Nearest => resample_nearest(&src_image, dst_image),
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
            * std::mem::size_of::<u32>()
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

fn get_temp_image_from_buffer(
    buffer: &mut Vec<u32>,
    width: NonZeroU32,
    height: NonZeroU32,
    pixel_type: PixelType,
) -> ImageData<&mut [u32]> {
    let buf_size = (width.get() * height.get()) as usize;
    if buffer.len() < buf_size {
        buffer.resize(buf_size, 0);
    }
    let pixels = &mut buffer[0..buf_size];
    ImageData::from_pixels(width, height, pixels, pixel_type).unwrap()
}

fn resample_nearest(src_image: &SrcImageView, dst_image: &mut DstImageView) {
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

fn resample_convolution(
    src_image: &SrcImageView,
    dst_image: &mut DstImageView,
    filter_type: FilterType,
    cpu_extensions: CpuExtensions,
    temp_buffer: &mut Vec<u32>,
) {
    let crop_box = src_image.crop_box();
    let dst_width = dst_image.width();
    let dst_height = dst_image.height();
    let (filter_fn, filter_support) = convolution::get_filter_func(filter_type);
    let resampler = cpu_extensions.get_resampler(src_image.pixel_type());

    let need_horizontal = dst_width != src_image.width() || crop_box.width != src_image.width();
    let need_vertical = dst_height != src_image.height() || crop_box.height != src_image.height();

    let mut vert_coeffs = convolution::precompute_coefficients(
        src_image.height(),
        crop_box.top as f64,
        crop_box.top as f64 + crop_box.height.get() as f64,
        dst_height,
        filter_fn,
        filter_support,
    );

    if need_horizontal {
        let horiz_coeffs = convolution::precompute_coefficients(
            src_image.width(),
            crop_box.left as f64,
            crop_box.left as f64 + crop_box.width.get() as f64,
            dst_width,
            filter_fn,
            filter_support,
        );

        // First used row in the source image
        let y_first = vert_coeffs.bounds[0].start;

        if need_vertical {
            // Shift bounds for vertical pass
            vert_coeffs
                .bounds
                .iter_mut()
                .for_each(|b| b.start -= y_first);

            // Last used row in the source image
            let last_y_bound = vert_coeffs.bounds.last().unwrap();
            let y_last = last_y_bound.start + last_y_bound.size;

            let temp_height = NonZeroU32::new(y_last - y_first).unwrap();
            let mut temp_image = get_temp_image_from_buffer(
                temp_buffer,
                dst_width,
                temp_height,
                src_image.pixel_type(),
            );
            resampler.horiz_convolution(
                src_image,
                &mut temp_image.dst_view(),
                y_first,
                horiz_coeffs,
            );
            resampler.vert_convolution(&temp_image.src_view(), dst_image, vert_coeffs);
        } else {
            resampler.horiz_convolution(src_image, dst_image, y_first, horiz_coeffs);
        }
    } else if need_vertical {
        resampler.vert_convolution(src_image, dst_image, vert_coeffs);
    }
}

fn resample_super_sampling(
    src_image: &SrcImageView,
    dst_image: &mut DstImageView,
    filter_type: FilterType,
    multiplicity: u8,
    cpu_extensions: CpuExtensions,
    temp_buffer: &mut Vec<u32>,
    convolution_temp_buffer: &mut Vec<u32>,
) {
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

        let mut tmp_img =
            get_temp_image_from_buffer(temp_buffer, tmp_width, tmp_height, src_image.pixel_type());
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
