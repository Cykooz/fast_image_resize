use crate::pixels::InnerPixel;
use crate::{ImageView, ImageViewMut};
use rayon::current_num_threads;
use rayon::prelude::*;
use std::num::NonZeroU32;

#[inline]
pub(crate) fn split_h_two_images_for_threading<'a, P: InnerPixel>(
    src_view: &'a impl ImageView<Pixel = P>,
    dst_view: &'a mut impl ImageViewMut<Pixel = P>,
    src_offset: u32,
) -> Option<
    impl ParallelIterator<
        Item = (
            impl ImageView<Pixel = P> + 'a,
            impl ImageViewMut<Pixel = P> + 'a,
        ),
    >,
> {
    debug_assert!(src_view.height() - src_offset >= dst_view.height());

    let dst_width = dst_view.width();
    let dst_height = dst_view.height();
    let max_num_parts = calculate_max_h_parts_number(dst_width, dst_height);

    let num_threads = current_num_threads() as u32;
    if num_threads > 1 && max_num_parts > 1 {
        let num_parts = NonZeroU32::new(num_threads.min(max_num_parts)).unwrap();
        let dst_height = NonZeroU32::new(dst_height).unwrap();
        if let Some(src_parts) = src_view.split_by_height(src_offset, dst_height, num_parts) {
            if let Some(dst_parts) = dst_view.split_by_height_mut(0, dst_height, num_parts) {
                let src_iter = src_parts.into_par_iter();
                let dst_iter = dst_parts.into_par_iter();
                return Some(src_iter.zip(dst_iter));
            }
        }
    }
    None
}

#[inline]
pub(crate) fn split_h_one_image_for_threading<P: InnerPixel>(
    image_view: &mut impl ImageViewMut<Pixel = P>,
) -> Option<impl ParallelIterator<Item = impl ImageViewMut<Pixel = P> + '_>> {
    let width = image_view.width();
    let height = image_view.height();
    let max_num_parts = calculate_max_h_parts_number(width, height);

    let num_threads = current_num_threads() as u32;
    if num_threads > 1 && max_num_parts > 1 {
        let num_parts = NonZeroU32::new(num_threads.min(max_num_parts)).unwrap();
        let height = NonZeroU32::new(height).unwrap();
        let img_parts = image_view.split_by_height_mut(0, height, num_parts);
        return img_parts.map(|parts| parts.into_par_iter());
    }
    None
}

/// It is not optimal to split images on too small parts.
/// We have to calculate minimal height of one part.
/// For small images, it is equal to `constant / area`.
/// For tall images, it is equal to `height / 256`.
fn calculate_max_h_parts_number(width: u32, height: u32) -> u32 {
    if width == 0 || height == 0 {
        return 1;
    }
    let area = height * height.max(width);
    let min_height = ((1 << 14) / area).max(height / 256);
    height / min_height.max(1)
}

#[inline]
pub(crate) fn split_v_two_images_for_threading<'a, P: InnerPixel>(
    src_view: &'a impl ImageView<Pixel = P>,
    dst_view: &'a mut impl ImageViewMut<Pixel = P>,
    src_offset: u32,
) -> Option<
    impl ParallelIterator<
        Item = (
            impl ImageView<Pixel = P> + 'a,
            impl ImageViewMut<Pixel = P> + 'a,
        ),
    >,
> {
    debug_assert!(src_view.width() - src_offset >= dst_view.width());

    let dst_width = dst_view.width();
    let dst_height = dst_view.height();
    let max_num_parts = calculate_max_v_parts_number(dst_width, dst_height);

    let num_threads = current_num_threads() as u32;
    if num_threads > 1 && max_num_parts > 1 {
        let num_parts = NonZeroU32::new(num_threads.min(max_num_parts)).unwrap();
        let dst_width = NonZeroU32::new(dst_width).unwrap();
        if let Some(src_parts) = src_view.split_by_width(src_offset, dst_width, num_parts) {
            if let Some(dst_parts) = dst_view.split_by_width_mut(0, dst_width, num_parts) {
                let src_iter = src_parts.into_par_iter();
                let dst_iter = dst_parts.into_par_iter();
                return Some(src_iter.zip(dst_iter));
            }
        }
    }
    None
}

/// It is not optimal to split images on too small parts.
/// We have to calculate minimal width of one part.
/// For small images, it is equal to `constant / area`.
/// For wide images, it is equal to `width / 256`.
fn calculate_max_v_parts_number(width: u32, height: u32) -> u32 {
    if width == 0 || height == 0 {
        return 1;
    }
    let area = width * height.max(width);
    let min_width = ((1 << 14) / area).max(width / 256);
    width / min_width.max(1)
}
