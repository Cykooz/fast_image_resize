use crate::convolution::{self, FilterType};
use crate::crop_box::CroppedSrcImageView;
use crate::image_view::{try_pixel_type, ImageView, ImageViewMut, IntoImageView, IntoImageViewMut};
use crate::images::TypedImage;
use crate::pixels::{self, InnerPixel};
use crate::{
    CpuExtensions, CropBox, DifferentDimensionsError, MulDiv, PixelTrait, PixelType, ResizeError,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum ResizeAlg {
    Nearest,
    Convolution(FilterType),
    /// It is like `Convolution` but with a fixed kernel size.
    ///
    /// This algorithm can be useful if you want to get a result
    /// similar to `OpenCV` (except `INTER_AREA` interpolation).
    Interpolation(FilterType),
    SuperSampling(FilterType, u8),
}

impl Default for ResizeAlg {
    fn default() -> Self {
        Self::Convolution(FilterType::Lanczos3)
    }
}

#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub enum SrcCropping {
    #[default]
    None,
    Crop(CropBox),
    FitIntoDestination((f64, f64)),
}

/// Options for configuring a resize process.
#[derive(Debug, Clone, Copy)]
pub struct ResizeOptions {
    /// Default: `ResizeAlg::Convolution(FilterType::Lanczos3)`
    pub algorithm: ResizeAlg,
    /// Default: `SrcCropping::None`.
    pub cropping: SrcCropping,
    /// Enable or disable consideration of the alpha channel when resizing.
    ///
    /// Default: `true`.
    pub mul_div_alpha: bool,
}

impl Default for ResizeOptions {
    fn default() -> Self {
        Self {
            algorithm: ResizeAlg::Convolution(FilterType::Lanczos3),
            cropping: SrcCropping::None,
            mul_div_alpha: true,
        }
    }
}

impl ResizeOptions {
    pub fn new() -> Self {
        Default::default()
    }

    /// Set resize algorythm.
    pub fn resize_alg(&self, resize_alg: ResizeAlg) -> Self {
        let mut options = *self;
        options.algorithm = resize_alg;
        options
    }

    /// Set crop box for source image.
    pub fn crop(&self, left: f64, top: f64, width: f64, height: f64) -> Self {
        let mut options = *self;
        options.cropping = SrcCropping::Crop(CropBox {
            left,
            top,
            width,
            height,
        });
        options
    }

    /// Fit a source image into the aspect ratio of a destination image without distortions.
    ///
    /// `centering` is used to control the cropping position. Use (0.5, 0.5) for
    /// center cropping (e.g. if cropping the width, take 50% off
    /// of the left side, and therefore 50% off the right side).
    /// (0.0, 0.0) will crop from the top left corner (i.e. if
    /// cropping the width, take all the crop off of the right
    /// side, and if cropping the height, take all of it off the
    /// bottom). (1.0, 0.0) will crop from the bottom left
    /// corner, etc. (i.e. if cropping the width, take all the
    /// crop off the left side, and if cropping the height, take
    /// none from the top, and therefore all off the bottom).
    pub fn fit_into_destination(&self, centering: Option<(f64, f64)>) -> Self {
        let mut options = *self;
        options.cropping = SrcCropping::FitIntoDestination(centering.unwrap_or((0.5, 0.5)));
        options
    }

    /// Enable or disable consideration of the alpha channel when resizing.
    pub fn use_alpha(&self, v: bool) -> Self {
        let mut options = *self;
        options.mul_div_alpha = v;
        options
    }

    fn get_crop_box<P: PixelTrait>(
        &self,
        src_view: &impl ImageView<Pixel = P>,
        dst_view: &impl ImageView<Pixel = P>,
    ) -> CropBox {
        match self.cropping {
            SrcCropping::None => CropBox {
                left: 0.,
                top: 0.,
                width: src_view.width() as _,
                height: src_view.height() as _,
            },
            SrcCropping::Crop(crop_box) => crop_box,
            SrcCropping::FitIntoDestination(centering) => CropBox::fit_src_into_dst_size(
                src_view.width(),
                src_view.height(),
                dst_view.width(),
                dst_view.height(),
                Some(centering),
            ),
        }
    }
}

/// Methods of this structure used to resize images.
#[derive(Default, Debug, Clone)]
pub struct Resizer {
    cpu_extensions: CpuExtensions,
    mul_div: MulDiv,
    alpha_buffer: Vec<u8>,
    convolution_buffer: Vec<u8>,
    super_sampling_buffer: Vec<u8>,
}

impl Resizer {
    /// Creates an instance of `Resizer`.
    ///
    /// By default, an instance of `Resizer` is created with the best CPU
    /// extensions provided by your CPU.
    /// You can change this by using the method [Resizer::set_cpu_extensions].
    pub fn new() -> Self {
        Default::default()
    }

    /// Resize the source image to the size of the destination image and save
    /// the result to the latter's pixel buffer.
    pub fn resize<'o>(
        &mut self,
        src_image: &impl IntoImageView,
        dst_image: &mut impl IntoImageViewMut,
        options: impl Into<Option<&'o ResizeOptions>>,
    ) -> Result<(), ResizeError> {
        let src_pixel_type = try_pixel_type(src_image)?;
        let dst_pixel_type = try_pixel_type(dst_image)?;
        if src_pixel_type != dst_pixel_type {
            return Err(ResizeError::PixelTypesAreDifferent);
        }

        use PixelType as PT;

        macro_rules! match_img {
            (
                $src_image: ident,
                $dst_image: ident,
                $(($p: path, $pt: path),)*
            ) => (
                match src_pixel_type {
                    $(
                        $p => {
                            match (
                                $src_image.image_view::<$pt>(),
                                $dst_image.image_view_mut::<$pt>(),
                            ) {
                                (Some(src), Some(mut dst)) => self.resize_typed(&src, &mut dst, options),
                                _ => Err(ResizeError::PixelTypesAreDifferent),
                            }
                        }
                    )*
                    _ => Err(ResizeError::PixelTypesAreDifferent),
                }
            )
        }

        #[cfg(not(feature = "only_u8x4"))]
        #[allow(unreachable_patterns)]
        let result = match_img!(
            src_image,
            dst_image,
            (PT::U8, pixels::U8),
            (PT::U8x2, pixels::U8x2),
            (PT::U8x3, pixels::U8x3),
            (PT::U8x4, pixels::U8x4),
            (PT::U16, pixels::U16),
            (PT::U16x2, pixels::U16x2),
            (PT::U16x3, pixels::U16x3),
            (PT::U16x4, pixels::U16x4),
            (PT::I32, pixels::I32),
            (PT::F32, pixels::F32),
            (PT::F32x2, pixels::F32x2),
            (PT::F32x3, pixels::F32x3),
            (PT::F32x4, pixels::F32x4),
        );

        #[cfg(feature = "only_u8x4")]
        let result = match_img!(src_image, dst_image, (PT::U8x4, pixels::U8x4),);

        result
    }

    /// Resize the source image to the size of the destination image
    /// and save the result to the latter's pixel buffer.
    pub fn resize_typed<'o, P: PixelTrait>(
        &mut self,
        src_view: &impl ImageView<Pixel = P>,
        dst_view: &mut impl ImageViewMut<Pixel = P>,
        options: impl Into<Option<&'o ResizeOptions>>,
    ) -> Result<(), ResizeError> {
        let default_options = ResizeOptions::default();
        let options = options.into().unwrap_or(&default_options);

        let crop_box = options.get_crop_box(src_view, dst_view);
        if crop_box.width == 0.
            || crop_box.height == 0.
            || dst_view.width() == 0
            || dst_view.height() == 0
        {
            // Do nothing if any size of the source or destination image is equal to zero.
            return Ok(());
        }

        let cropped_src_view = CroppedSrcImageView::crop(src_view, crop_box)?;

        if copy_image(&cropped_src_view, dst_view).is_ok() {
            // If `copy_image()` returns `Ok` it means that
            // the size of the destination image is equal to
            // the size of the cropped source image and
            // the copy operation has success.
            return Ok(());
        }

        match options.algorithm {
            ResizeAlg::Nearest => resample_nearest(&cropped_src_view, dst_view),
            ResizeAlg::Convolution(filter_type) => self.resample_convolution(
                &cropped_src_view,
                dst_view,
                filter_type,
                true,
                options.mul_div_alpha,
            ),
            ResizeAlg::Interpolation(filter_type) => self.resample_convolution(
                &cropped_src_view,
                dst_view,
                filter_type,
                false,
                options.mul_div_alpha,
            ),
            ResizeAlg::SuperSampling(filter_type, multiplicity) => self.resample_super_sampling(
                &cropped_src_view,
                dst_view,
                filter_type,
                multiplicity,
                options.mul_div_alpha,
            ),
        }
        Ok(())
    }

    /// Returns the size of internal buffers used to store the results of
    /// intermediate resizing steps.
    pub fn size_of_internal_buffers(&self) -> usize {
        (self.alpha_buffer.capacity()
            + self.convolution_buffer.capacity()
            + self.super_sampling_buffer.capacity())
            * size_of::<u8>()
    }

    /// Deallocates the internal buffers used to store the results of
    /// intermediate resizing steps.
    pub fn reset_internal_buffers(&mut self) {
        if self.alpha_buffer.capacity() > 0 {
            self.alpha_buffer = Vec::new();
        }
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
    /// This is unsafe because this method allows you to set a CPU extension
    /// that is not supported by your CPU.
    pub unsafe fn set_cpu_extensions(&mut self, extensions: CpuExtensions) {
        self.cpu_extensions = extensions;
        self.mul_div.set_cpu_extensions(extensions);
    }

    fn resample_convolution<P: PixelTrait>(
        &mut self,
        cropped_src_view: &CroppedSrcImageView<impl ImageView<Pixel = P>>,
        dst_view: &mut impl ImageViewMut<Pixel = P>,
        filter_type: FilterType,
        adaptive_kernel_size: bool,
        use_alpha: bool,
    ) {
        if use_alpha && self.mul_div.is_supported(P::pixel_type()) {
            let src_view = cropped_src_view.image_view();
            let mut alpha_buffer = std::mem::take(&mut self.alpha_buffer);

            let mut premultiplied_src =
                get_temp_image_from_buffer(&mut alpha_buffer, src_view.width(), src_view.height());

            if self
                .mul_div
                .multiply_alpha_typed(src_view, &mut premultiplied_src)
                .is_ok()
            {
                // SAFETY: `premultiplied_src` has the same size as `src_view`
                let cropped_premultiplied_src = unsafe {
                    CroppedSrcImageView::crop_unchecked(
                        &premultiplied_src,
                        cropped_src_view.crop_box(),
                    )
                };
                self.do_convolution(
                    &cropped_premultiplied_src,
                    dst_view,
                    filter_type,
                    adaptive_kernel_size,
                );
                self.mul_div.divide_alpha_inplace_typed(dst_view).unwrap();
                self.alpha_buffer = alpha_buffer;
                return;
            }
            self.alpha_buffer = alpha_buffer;
        }

        self.do_convolution(
            cropped_src_view,
            dst_view,
            filter_type,
            adaptive_kernel_size,
        );
    }

    fn do_convolution<P: PixelTrait>(
        &mut self,
        cropped_src_view: &CroppedSrcImageView<impl ImageView<Pixel = P>>,
        dst_view: &mut impl ImageViewMut<Pixel = P>,
        filter_type: FilterType,
        adaptive_kernel_size: bool,
    ) {
        let src_view = cropped_src_view.image_view();
        let crop_box = cropped_src_view.crop_box();
        let (dst_width, dst_height) = (dst_view.width(), dst_view.height());
        if dst_width == 0 || dst_height == 0 || crop_box.width <= 0. || crop_box.height <= 0. {
            return;
        }

        let (filter_fn, filter_support) = convolution::get_filter_func(filter_type);

        let need_horizontal =
            dst_width as f64 != crop_box.width || crop_box.left != crop_box.left.round();
        let horiz_coeffs = need_horizontal.then(|| {
            test_log!("compute horizontal convolution coefficients");
            convolution::precompute_coefficients(
                src_view.width(),
                crop_box.left,
                crop_box.left + crop_box.width,
                dst_width,
                filter_fn,
                filter_support,
                adaptive_kernel_size,
            )
        });

        let need_vertical =
            dst_height as f64 != crop_box.height || crop_box.top != crop_box.top.round();
        let vert_coeffs = need_vertical.then(|| {
            test_log!("compute vertical convolution coefficients");
            convolution::precompute_coefficients(
                src_view.height(),
                crop_box.top,
                crop_box.top + crop_box.height,
                dst_height,
                filter_fn,
                filter_support,
                adaptive_kernel_size,
            )
        });

        match (horiz_coeffs, vert_coeffs) {
            (Some(mut horiz_coeffs), Some(mut vert_coeffs)) => {
                if P::components_is_u8() {
                    // For u8-based images, it is faster to do the vertical pass first
                    // instead of the horizontal.
                    let x_first = horiz_coeffs.bounds[0].start;
                    // Last used col in the source image
                    let last_x_bound = horiz_coeffs.bounds.last().unwrap();
                    let x_last = last_x_bound.start + last_x_bound.size;
                    let temp_width = x_last - x_first;
                    let mut temp_image = get_temp_image_from_buffer(
                        &mut self.convolution_buffer,
                        temp_width,
                        dst_height,
                    );

                    P::vert_convolution(
                        src_view,
                        &mut temp_image,
                        x_first,
                        vert_coeffs,
                        self.cpu_extensions,
                    );

                    // Shift bounds for the horizontal pass
                    horiz_coeffs
                        .bounds
                        .iter_mut()
                        .for_each(|b| b.start -= x_first);

                    P::horiz_convolution(
                        &temp_image,
                        dst_view,
                        0,
                        horiz_coeffs,
                        self.cpu_extensions,
                    );
                } else {
                    let y_first = vert_coeffs.bounds[0].start;
                    // Last used row in the source image
                    let last_y_bound = vert_coeffs.bounds.last().unwrap();
                    let y_last = last_y_bound.start + last_y_bound.size;
                    let temp_height = y_last - y_first;
                    let mut temp_image = get_temp_image_from_buffer(
                        &mut self.convolution_buffer,
                        dst_width,
                        temp_height,
                    );
                    P::horiz_convolution(
                        src_view,
                        &mut temp_image,
                        y_first,
                        horiz_coeffs,
                        self.cpu_extensions,
                    );

                    // Shift bounds for the vertical pass
                    vert_coeffs
                        .bounds
                        .iter_mut()
                        .for_each(|b| b.start -= y_first);
                    P::vert_convolution(&temp_image, dst_view, 0, vert_coeffs, self.cpu_extensions);
                }
            }
            (Some(horiz_coeffs), None) => {
                P::horiz_convolution(
                    src_view,
                    dst_view,
                    crop_box.top as u32, // crop_box.top is exactly an integer if the vertical pass is not required
                    horiz_coeffs,
                    self.cpu_extensions,
                );
            }
            (None, Some(vert_coeffs)) => {
                P::vert_convolution(
                    src_view,
                    dst_view,
                    crop_box.left as u32, // crop_box.left is exactly an integer if the horizontal pass is not required
                    vert_coeffs,
                    self.cpu_extensions,
                );
            }
            _ => {}
        }
    }

    fn resample_super_sampling<P: PixelTrait>(
        &mut self,
        cropped_src_view: &CroppedSrcImageView<impl ImageView<Pixel = P>>,
        dst_view: &mut impl ImageViewMut<Pixel = P>,
        filter_type: FilterType,
        multiplicity: u8,
        use_alpha: bool,
    ) {
        let crop_box = cropped_src_view.crop_box();
        let dst_width = dst_view.width();
        let dst_height = dst_view.height();
        if dst_width == 0 || dst_height == 0 || crop_box.width <= 0. || crop_box.height <= 0. {
            return;
        }

        let width_scale = crop_box.width / dst_width as f64;
        let height_scale = crop_box.height / dst_height as f64;
        // It makes sense to resize the image in two steps only if the image
        // size is greater than the required size by multiplicity times.
        let factor = width_scale.min(height_scale) / multiplicity as f64;
        if factor > 1.2 {
            // The first step is resizing the source image by the fastest algorithm.
            // The temporary image will be about ``multiplicity`` times larger
            // than required.
            let tmp_width = (crop_box.width / factor).round() as u32;
            let tmp_height = (crop_box.height / factor).round() as u32;

            let mut super_sampling_buffer = std::mem::take(&mut self.super_sampling_buffer);
            let mut tmp_img =
                get_temp_image_from_buffer(&mut super_sampling_buffer, tmp_width, tmp_height);
            resample_nearest(cropped_src_view, &mut tmp_img);
            // The second step is resizing the temporary image with a convolution.
            let cropped_tmp_img = CroppedSrcImageView::new(&tmp_img);
            self.resample_convolution(&cropped_tmp_img, dst_view, filter_type, true, use_alpha);
            self.super_sampling_buffer = super_sampling_buffer;
        } else {
            // There is no point in doing the resizing in two steps.
            // We immediately resize the original image with a convolution.
            self.resample_convolution(cropped_src_view, dst_view, filter_type, true, use_alpha);
        }
    }
}

/// Creates an inner image container from part of the given buffer.
/// Buffer may be expanded if its size is less than required for the image.
fn get_temp_image_from_buffer<P: PixelTrait>(
    buffer: &mut Vec<u8>,
    width: u32,
    height: u32,
) -> TypedImage<'_, P> {
    let pixels_count = width as usize * height as usize;
    // Add pixel size as a gap for alignment of resulted buffer.
    let buf_size = pixels_count * P::size() + P::size();
    if buffer.len() < buf_size {
        buffer.resize(buf_size, 0);
    }
    let pixels = unsafe { buffer.align_to_mut::<P>().1 };
    TypedImage::from_pixels_slice(width, height, &mut pixels[0..pixels_count]).unwrap()
}

fn resample_nearest<P: InnerPixel>(
    cropped_src_view: &CroppedSrcImageView<impl ImageView<Pixel = P>>,
    dst_view: &mut impl ImageViewMut<Pixel = P>,
) {
    let (dst_width, dst_height) = (dst_view.width(), dst_view.height());
    let src_view = cropped_src_view.image_view();
    let crop_box = cropped_src_view.crop_box();
    if dst_width == 0 || dst_height == 0 || crop_box.width <= 0. || crop_box.height <= 0. {
        return;
    }
    let x_scale = crop_box.width / dst_width as f64;
    let y_scale = crop_box.height / dst_height as f64;

    // Pretabulate horizontal pixel positions

    let x_in_start = crop_box.left + x_scale * 0.5;
    let max_src_x = src_view.width() as usize;
    let x_in_tab: Vec<usize> = (0..dst_width)
        .map(|x| ((x_in_start + x_scale * x as f64) as usize).min(max_src_x))
        .collect();

    let y_in_start = crop_box.top + y_scale * 0.5;

    let src_rows = src_view.iter_rows_with_step(y_in_start, y_scale, dst_height);
    let dst_rows = dst_view.iter_rows_mut(0);

    #[cfg(feature = "rayon")]
    {
        use rayon::prelude::*;

        let mut row_refs: Vec<(&mut [P], &[P])> = dst_rows.zip(src_rows).collect();
        row_refs.par_iter_mut().for_each(|(out_row, in_row)| {
            for (&x_in, out_pixel) in x_in_tab.iter().zip(out_row.iter_mut()) {
                // Safety of x_in value guaranteed by algorithm of creating of x_in_tab
                *out_pixel = unsafe { *in_row.get_unchecked(x_in) };
            }
        });
    }
    #[cfg(not(feature = "rayon"))]
    {
        for (out_row, in_row) in dst_rows.zip(src_rows) {
            for (&x_in, out_pixel) in x_in_tab.iter().zip(out_row.iter_mut()) {
                // Safety of x_in value guaranteed by algorithm of creating of x_in_tab
                *out_pixel = unsafe { *in_row.get_unchecked(x_in) };
            }
        }
    }
}

/// Copy pixels from src_view into dst_view.
pub(crate) fn copy_image<S, P: PixelTrait>(
    cropped_src_view: &CroppedSrcImageView<S>,
    dst_view: &mut impl ImageViewMut<Pixel = P>,
) -> Result<(), DifferentDimensionsError>
where
    S: ImageView<Pixel = P>,
{
    let crop_box = cropped_src_view.crop_box();
    if crop_box.left != crop_box.left.round()
        || crop_box.top != crop_box.top.round()
        || crop_box.width != crop_box.width.round()
        || crop_box.height != crop_box.height.round()
    {
        // The crop box has a fractional part in some his part
        return Err(DifferentDimensionsError);
    }
    if dst_view.width() != crop_box.width as u32 || dst_view.height() != crop_box.height as u32 {
        return Err(DifferentDimensionsError);
    }
    if dst_view.width() > 0 && dst_view.height() > 0 {
        dst_view
            .iter_rows_mut(0)
            .zip(iter_cropped_rows(cropped_src_view))
            .for_each(|(d, s)| d.copy_from_slice(s));
    }
    Ok(())
}

fn iter_cropped_rows<'a, S: ImageView>(
    cropped_src_view: &'a CroppedSrcImageView<S>,
) -> impl Iterator<Item = &'a [S::Pixel]> {
    let crop_box = cropped_src_view.crop_box();
    let rows = cropped_src_view
        .image_view()
        .iter_rows(crop_box.top.max(0.) as u32)
        .take(crop_box.height.max(0.) as usize);
    let first_col = crop_box.left.max(0.) as usize;
    let last_col = first_col + crop_box.width.max(0.) as usize;
    rows.map(move |row| unsafe { row.get_unchecked(first_col..last_col) })
}
