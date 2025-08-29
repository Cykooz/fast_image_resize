use std::num::NonZeroU32;

use rayon::current_num_threads;
use rayon::prelude::*;

use crate::pixels::InnerPixel;
use crate::{ImageView, ImageViewMut};

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
    let max_num_parts = calculate_max_number_of_horizonal_parts(dst_width, dst_height).get();

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
    let max_num_parts = calculate_max_number_of_horizonal_parts(width, height).get();

    let num_threads = current_num_threads() as u32;
    if num_threads > 1 && max_num_parts > 1 {
        let num_parts = NonZeroU32::new(num_threads.min(max_num_parts)).unwrap();
        let height = NonZeroU32::new(height).unwrap();
        let img_parts = image_view.split_by_height_mut(0, height, num_parts);
        return img_parts.map(|parts| parts.into_par_iter());
    }
    None
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
    let max_num_parts = calculate_max_number_of_vertical_parts(dst_width, dst_height).get();

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

const PIXELS_PER_THREAD: u64 = 1_024; // It was selected as a result of simple benchmarking.

fn calculate_max_number_of_horizonal_parts(width: u32, height: u32) -> NonZeroU32 {
    let area = width as u64 * height as u64;
    let num_parts = (area / PIXELS_PER_THREAD).min(height as _) as u32;
    NonZeroU32::new(num_parts).unwrap_or(NonZeroU32::MIN)
}

fn calculate_max_number_of_vertical_parts(width: u32, height: u32) -> NonZeroU32 {
    let area = width as u64 * height as u64;
    let num_parts = (area / PIXELS_PER_THREAD).min(width as _) as u32;
    NonZeroU32::new(num_parts).unwrap_or(NonZeroU32::MIN)
}
