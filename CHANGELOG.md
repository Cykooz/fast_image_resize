## [5.3.0] - 2025-09-02

### Added

- Added support for multi-thread image resizing using the `ResizeAlg::Nearest` algorithm.
  ([#54](https://github.com/Cykooz/fast_image_resize/issues/54)).

## [5.2.2] - 2025-08-29

### Fixed

- Fixed a "divide by zero" error in case of using multithreading to resize images
  with particular sizes ([#55](https://github.com/Cykooz/fast_image_resize/issues/55)).

## [5.2.1] - 2025-07-27

### Changed

- Added minimum supported Rust version (MSRV) into `Cargo.toml`.

## [5.2.0] - 2025-07-12

### Added

- Added support of `DynamicImage::ImageRgb32F` and `DynamicImage::ImageRgba32F`
  form the `image` crate ([#50](https://github.com/Cykooz/fast_image_resize/pull/50)).

## [5.1.4] - 2025-05-16

### Fixed

- Fixed `SSE4.1` and `AVX2` implementation for dividing image by
  alpha channel for images with `U16x2` pixels.
- Fixed `NEON` implementation for dividing image by
  alpha channel for images with `U16x2` and `U16x4` pixels .

## [5.1.3] - 2025-04-06

### Fixed

- Fixed error in `NEON` implementation of `MulDiv::multiply_alpha()` and
  `MulDiv::multiply_alpha_inplace()` for `U8x2` pixels
  ([#49](https://github.com/Cykooz/fast_image_resize/issues/49)).
- Replaced the internal crate `testing` on the corresponding module in the `tests` directory
  ([#48](https://github.com/Cykooz/fast_image_resize/issues/48)).

## [5.1.2] - 2025-02-16

### Fixed

- Fixed error in implementation of `ImageView::split_by_width()`, `ImageView::split_by_height()`,
  `ImageViewMut::split_by_width_mut()` and `ImageViewMut::split_by_height_mut()`
  ([#46](https://github.com/Cykooz/fast_image_resize/issues/46)).

## [5.1.1] - 2025-01-13

### Fixed

- Fixed error in implementation of `ImageView::split_by_width()`, `ImageView::split_by_height()`,
  `ImageViewMut::split_by_width_mut()` and `ImageViewMut::split_by_height_mut()`
  ([#43](https://github.com/Cykooz/fast_image_resize/issues/43)).

## [5.1.0] - 2024-12-09

### Changed

- Improved speed (about 9%) of `SSE4.1` implementation for vertical
  convolution pass for pixel types based on `u8` components.

### Fixed

- `is_aarch64_feature_detected()` is used now in
  the `CpuExtensions::is_supported()` method for `aarch64` architecture.

## [5.0.0] - 2024-10-03

### Added

- Added support for multi-thread image processing with the help of `rayon` crate.
  You should enable `rayon` feature to turn on this behavior.
- Added methods to split image in different directions:
    - `ImageView::split_by_height()`
    - `ImageView::split_by_width()`
    - `ImageViewMut::split_by_height_mut()`
    - `ImageViewMut::split_by_width_mut()`

  These methods have default implementation and are used for multi-thread
  image processing.

### Changed

- **BREAKING**: Added supertraits `Send`, `Sync` and `Sized` to the `ImageView` trait.
- Optimized convolution algorythm by deleting zero coefficients from start and
  end of bounds.

## [4.2.3] - 2025-05-16

### Fixed

- Fixed `SSE4.1` and `AVX2` implementation fo dividing image by
  alpha channel for images with `U16x2` pixels.
- Fixed `NEON` implementation for dividing image by
  alpha channel for images with `U16x2` and `U16x4` pixels.

## [4.2.2] - 2025-04-06

## Fixed

- Fixed error in `NEON` implementation of `MulDiv::multiply_alpha()` and
  `MulDiv::multiply_alpha_inplace()` for `U8x2` pixels
  ([#49](https://github.com/Cykooz/fast_image_resize/issues/49)).

## [4.2.1] - 2024-07-24

### Fixed

- Disabled default features of the `image` crate (#36).

## [4.2.0] - 2024-07-19

### Added

- Added new resize algorithm `ResizeAlg::Interpolation` (#32).

  It is like `ResizeAlg::Convolution` but with fixed kernel size.
  This algorithm can be useful if you want to get a result similar
  to `OpenCV` (except `INTER_AREA` interpolation).

## [4.1.0] - 2024-07-14

### Added

- Added support for optimization with help of `SSE4.1` and `AVX2` for
  the `F32` pixel type.
- Added support for new pixel types `F32x2`, `F32x3` and `F32x4` with
  optimizations for `SSE4.1` and `AVX2` (#30).

## [4.0.0] - 2024-05-13

### Added

- Added Gaussian filter for convolution algorithm.
- Method `PixelType::size()` was made public.
- Added new image containers:
    - `ImageRef`
    - `TypedImageRef`
    - `TypedImage`
    - `TypedCroppedImage`
    - `TypedCroppedImageMut`
    - `CroppedImage`
    - `CroppedImageMut`

### Fixed

- Fixed dividing image by alpha channel.

### Changed

A lot of **breaking changes** have been done in this release:

- Structures `ImageView` and `ImageViewMut` have been removed. They always
  did unnecessary memory allocation to store references to image rows.
  Instead of these structures, the `ImageView` and `ImageViewMut` traits
  have been added. The crate accepts any image container that provides
  these traits.
- Also, traits `IntoImageView` and `IntoImageViewMut` have been added.
  They allow you to write runtime adapters to convert your particular
  image container into something that provides `ImageView`/`ImageViewMut` trait.
- `Resizer` now has two methods for resize (dynamic and typed):
    - `resize()` accepts references to `impl IntoImageView` and `impl IntoImageViewMut`;
    - `resize_typed()` accepts references to `impl ImageView` and `impl ImageViewMut`.
- Resize methods also accept the `options` argument.
  With the help of this argument, you can specify:
    - resize algorithm (default: Lanczos3);
    - how to crop the source image;
    - whether to multiply the source image by the alpha channel and
      divide the destination image by the alpha channel.
      By default, Resizer multiplies and divides by alpha channel
      images with `U8x2`, `U8x4`, `U16x2` and `U16x4` pixels.
- Argument `resize_alg` was removed from `Resizer::new()` method, use
  `options` argument of methods to resize instead.
- The `MulDiv` implementation has been changed in the same way as `Resizer`.
  It now has two versions of each method: dynamic and typed.
- Type of image dimensions has been changed from `NonZeroU32` into `u32`.
  Now you can create and use zero-sized images.
- `Image` (embedded implementation of image container) moved from root of
  the crate into module `images`.
- Added optional feature "image".
  It adds implementation of traits `IntoImageView` and `IntoImageViewMut` for the
  [DynamicImage](https://docs.rs/image/latest/image/enum.DynamicImage.html) type
  from the `image` crate. This implementation allows you to use `DynamicImage`
  instances as arguments for methods of this crate.

Look at the difference between versions 3 and 4 on example
of resizing RGBA8 image from given u8-buffer with pixels-data.

3.x version:

```rust
use fast_image_resize::{Image, MulDiv, PixelType, Resizer};
use std::num::NonZeroU32;

fn my_resize(
    src_width: u32,
    src_height: u32,
    src_pixels: &mut [u8],
    dst_width: u32,
    dst_height: u32,
) -> Image {
    let src_width = NonZeroU32::new(src_width).unwrap();
    let src_height = NonZeroU32::new(src_height).unwrap();
    let src_image = Image::from_slice_u8(
        src_width,
        src_height,
        src_pixels,
        PixelType::U8x4,
    ).unwrap();

    // Multiple RGB channels of source image by alpha channel.
    let alpha_mul_div = MulDiv::default();
    let mut tmp_image = Image::new(
        src_width,
        src_height,
        PixelType::U8x4,
    );
    alpha_mul_div
        .multiply_alpha(
            &src_image.view(),
            &mut tmp_image.view_mut(),
        ).unwrap();

    // Create container for data of destination image.
    let dst_width = NonZeroU32::new(dst_width).unwrap();
    let dst_height = NonZeroU32::new(dst_height).unwrap();
    let mut dst_image = Image::new(
        dst_width,
        dst_height,
        PixelType::U8x4,
    );

    // Get mutable view of destination image data.
    let mut dst_view = dst_image.view_mut();

    // Create Resizer instance and resize source image
    // into buffer of destination image.
    let mut resizer = Resizer::default();
    resizer.resize(&tmp_image.view(), &mut dst_view).unwrap();

    // Divide RGB channels of destination image by alpha.
    alpha_mul_div.divide_alpha_inplace(&mut dst_view).unwrap();

    dst_image
}
```

4.x version:

```rust
use fast_image_resize::images::{Image, ImageRef};
use fast_image_resize::{PixelType, Resizer};

fn my_resize(
    src_width: u32,
    src_height: u32,
    src_pixels: &[u8],
    dst_width: u32,
    dst_height: u32,
) -> Image {
    let src_image = ImageRef::new(
        src_width,
        src_height,
        src_pixels,
        PixelType::U8x4,
    ).unwrap();

    // Create container for data of destination image.
    let mut dst_image = Image::new(
        dst_width,
        dst_height,
        PixelType::U8x4,
    );

    // Create Resizer instance and resize source image
    // into buffer of destination image.
    let mut resizer = Resizer::new();
    // By default, Resizer multiplies and divides by alpha channel
    // images with U8x2, U8x4, U16x2 and U16x4 pixels.
    resizer.resize(&src_image, &mut dst_image, None).unwrap();

    dst_image
}
```

## [3.0.4] - 2024-02-15

### Fixed

- Fixed error with incorrect cropping of source image.

## [3.0.3] - 2024-02-07

### Fixed

- Fixed version of `num-traits` in the `Cargo.toml`.

## [3.0.2] - 2024-02-07

### Added

- Added `Custom` variant for `FilterType` enum and corresponding `Filter` structure.
- **BREAKING**: Added a new variant of enum `CropBoxError::WidthOrHeightLessOrEqualToZero`.

### Changed

- Slightly improved (about 3%) speed of `AVX2` implementation of `Convolution` trait
  for `U8x3` and `U8x4` images.
- **BREAKING**: Changed internal data type for `U8x4` structure.
  Now it is `[u8; 4]` instead of `u32`.
- Significantly improved (4.5 times on `x86_64`) speed of vertical convolution pass implemented
  in native Rust for `U8`, `U8x2`, `U8x3` and `U8x4` images.
- Changed order of convolution passes for `U8`, `U8x2`, `U8x3` and `U8x4` images.
  Now a vertical pass is the first and a horizontal pass is the second.
- **BREAKING**: Type of the `CropBox` fields has been changed to `f64`. Now you can use
  fractional size and position of crop box.
- **BREAKING**: Type of the `centering` argument of `ImageView::set_crop_box_to_fit_dst_size()`
  and `DynamicImageView::set_crop_box_to_fit_dst_size()` methods has been changed to `Optional<(f64, f64)>`.
- **BREAKING**: The `crop_box` argument of `ImageViewMut::crop()` and `DynamicImageViewMut::crop()`
  methods has been replaced with separate `left`, `top`, `width` and `height` arguments.

## [2.7.3] - 2023-05-07

### Fixed

- Fixed size of rows in cropped `ImageViewMut` created by
  `ImageViewMut::crop` method
  ([#17](https://github.com/Cykooz/fast_image_resize/issues/17)).

## [2.7.2] - 2023-05-04

### Fixed

- Added using of (read|write)_unaligned for unaligned pointers
  on `arm64` and `wasm32` architectures.
  ([#15](https://github.com/Cykooz/fast_image_resize/issues/15)).

## [2.7.1] - 2023-04-28

### Fixed

- Added using of (read|write)_unaligned for unaligned pointers on `x86_64` architecture.
  ([#16](https://github.com/Cykooz/fast_image_resize/pull/16)).

## [2.7.0] - 2023-03-24

### Added

- Added method `DynamicImageViewMut::crop()` to create cropped version of `DynamicImageViewMut`
  ([#13](https://github.com/Cykooz/fast_image_resize/issues/13)).
- Added method `ImageViewMut::crop()` to create cropped version of `ImageViewMut`.

## [2.6.0] - 2023-03-01

### Crate

- Slightly improved speed of `Convolution` implementation for `U8x2` images
  and `Wasm32 SIMD128` instructions.
- Method `Image::buffer_mut()` was made public
  ([#14](https://github.com/Cykooz/fast_image_resize/pull/14))

## [2.5.0] - 2023-01-29

### Crate

- Added support of optimisation with helps of `Wasm32 SIMD128` for
  all types of images exclude `I32` and `F32`
  (thanks to @cdmurph32, [#11](https://github.com/Cykooz/fast_image_resize/pull/11)).

### Benchmarks

- Benchmark framework `glassbench` replaced by `criterion`.
- Added report with results of benchmarks for `wasm32-wasi` target.

## [2.4.0] - 2022-12-11

### Crate

- Slightly improved speed of `MulDiv` implementation for `U8x2`, `U8x4`, `U16x2` and `U16x4` images.
- Added optimisation for processing `U16x2` images by `MulDiv` with
  helps of `NEON SIMD` instructions.
- Excluded possibility of unnecessary operations during resize
  of cropped image by convolution algorithm.
- Added implementation `From` trait to convert `ImageViewMut` into `ImageView`.
- Added implementation `From` trait to convert `DynamicImageViewMut` into `DynamicImageView`.

## [2.3.0] - 2022-11-25

### Crate

- Added support of optimisation with helps of `NEON SIMD` for convolution of `U16` images.
- Added support of optimisation with helps of `NEON SIMD` for convolution of `U16x2` images.
- Added support of optimisation with helps of `NEON SIMD` for convolution of `U16x3` images.
- Improved optimisation of convolution with helps of `NEON SIMD` for `U8` images.

## [2.2.0] - 2022-11-18

### Crate

- Added support of optimisation with helps of `NEON SIMD` for convolution of `U8` images.
- Added support of optimisation with helps of `NEON SIMD` for convolution of `U8x2` images.
- Added support of optimisation with helps of `NEON SIMD` for convolution of `U8x3` images.
- Added optimisation for processing `U8x2` images by `MulDiv` with
  helps of `NEON SIMD` instructions.

## [2.1.0] - 2022-11-11

### Crate

- Added method `CpuExtensions::is_supported(&self)`.
- Internals of `PixelComponentMapper` changed to use heap to store its data.
- Added support of optimisation with helps of `NEON SIMD` for convolution of `U16x4` images.
- Added optimisation for processing `U16x4` images by `MulDiv` with
  helps of `NEON SIMD` instructions.
- Added full optimisation for convolution of `U8` images with helps of
  `SSE4.1` instructions.
- Fixed link to documentation page in `README.md` file.
- Fixed error in implementation of `MulDiv::divide_alpha()` and `MulDiv::divide_alpha_inplace()`
  for `U16x4` pixels with optimisation with helps of `SSE4.1` and `AVX2`.
- Improved optimisation of `MulDiv` with helps of `NEON SIMD` for `U8x4` pixels.

## [2.0.0] - 2022-10-28

### Crate

- Breaking changes:
    - Struct `ImageView` replaced by enum `DynamicImageView`.
    - Struct `ImageViewMut` replaced by enum `DynamicImageViewMut`.
    - Trait `Pixel` renamed into `PixelExt` and some its internals changed:
        - associated type `ComponentsCount` renamed into `CountOfComponents`.
        - associated type `ComponentCountOfValues` deleted.
        - associated method `components_count` renamed into `count_of_components`.
        - associated method `component_count_of_values` renamed into `count_of_component_values`.
    - All pixel types (`U8`, `U8x2`, ...) replaced by type aliases for new
      generic structure `Pixel`. Use method `new()` to create
      instance of one pixel.
- Added structure `PixelComponentMapper` that holds tables for mapping values of pixel's
  components in forward and backward directions.
- Added function `create_gamma_22_mapper()` to create instance of `PixelComponentMapper`
  that converts images with gamma 2.2 to linear colorspace and back.
- Added function `create_srgb_mapper()` to create instance of `PixelComponentMapper`
  that converts images from SRGB colorspace to linear RGB and back.
- Added generic structs `ImageView` and `ImageViewMut`.
- Added functions `change_type_of_pixel_components` and
  `change_type_of_pixel_components_dyn` that change type of pixel's
  components in whole image.
- Added generic trait `IntoPixelComponent<Out: PixelComponent>`.
- Added generic structure `Pixel` for create all types of pixels.
- Added full support of optimisation with helps of `SSE4.1` for convolution of `U8x3` images.
- Added support of optimisation with helps of `NEON SIMD` for convolution of `U8x4` images.
- Added optimisation for processing `U8x4` images by `MulDiv` with
  helps of `NEON SIMD` instructions.

### Example application

- Added option `--high_precision` to use `u16` as pixel components
  for intermediate image representation.
- Added converting of source image into linear colorspace before it will be resized.
  Destination image will be returned into original colorspace before it will be saved.

## [1.0.0] - 2022-07-24

- Added example of command line application "resizer".

## [0.9.7] - 2022-07-14

- Fixed resizing when the destination image has the same dimensions
  as the source image
  ([#9](https://github.com/Cykooz/fast_image_resize/issues/9)).

## [0.9.6] - 2022-06-28

- Added support of new type of pixels `PixelType::U16x4`.
- Fixed benchmarks for resizing images with alpha channel using
  the `resizer` crate.
- Removed `image` crate from benchmarks for resizing images with alpha.
- Added method `Image::copy(&self) -> Image<'static>`.

## [0.9.5] - 2022-06-22

- Fixed README.md

## [0.9.4] - 2022-06-22

- Added support of new type of pixels `PixelType::U16x2`
  (e.g. luma with alpha channel).

## [0.9.3] - 2022-05-31

- Added support of new type of pixels `PixelType::U16`.

## [0.9.2] - 2022-05-19

- Added optimisation for convolution of `U8x2` images with helps of `SSE4.1`.

## [0.9.1] - 2022-05-12

- Added optimisation for processing `U8x2` images by `MulDiv` with
  helps of `SSE4.1` and `AVX2` instructions.
- Added optimisation for convolution of `U16x2` images with helps of
  `AVX2` instructions.

## [0.9.0] - 2022-05-01

- Added support of new type of pixels `PixelType::U8x2`.
- Added into `MulDiv` support of images with pixel type `U8x2`.
- Added method `Image::into_vec(self) -> Vec<u8>`
  ([#7](https://github.com/Cykooz/fast_image_resize/pull/7)).

## [0.8.0] - 2022-03-23

- Added optimisation for convolution of U16x3 images with helps of `SSE4.1`
  and `AVX2` instructions.
- Added partial optimisation for convolution of U8 images with helps of
  `SSE4.1` instructions.
- Allowed to create an instance of `Image`, `ImageVew` and `ImageViewMut`
  from a buffer larger than necessary
  ([#5](https://github.com/Cykooz/fast_image_resize/issues/5)).
- Breaking changes:
    - Removed methods: `Image::from_vec_u32()`, `Image::from_slice_u32()`.
    - Removed error `InvalidBufferSizeError`.

## [0.7.0] - 2022-01-27

- Added support of new type of pixels `PixelType::U16x3`.
- Breaking changes:
    - Added variant `U16x3` into the enum `PixelType`.

## [0.6.0] - 2022-01-12

- Added optimisation of multiplying and dividing image by alpha channel with helps
  of `SSE4.1` instructions.
- Improved performance of dividing image by alpha channel without forced
  SIMD instructions.
- Breaking changes:
    - Deleted variant `SSE2` from enum `CpuExtensions`.

## [0.5.3] - 2021-12-14

- Added optimisation of convolution U8x3 images with helps of `AVX2` instructions.
- Fixed error in code for convolution U8x4 images with helps of `SSE4.1` instructions.
- Fixed error in code for convolution U8 images with helps of `AVX2` instructions.

## [0.5.2] - 2021-11-26

- Fixed compile errors on non-x86 architectures.

## [0.5.1] - 2021-11-24

- Fixed compile errors on non-x86 architectures.

## [0.5.0] - 2021-11-18

- Added support of new type of pixels `PixelType::U8x3` (with
  auto-vectorization for SSE4.1).
- Exposed module `fast_image_resize::pixels` with types `U8x3`,
  `U8x4`, `F32`, `I32`, `U8` used as wrappers for represent type of
  one pixel of image.
- Some optimisations in code of convolution written in Rust (without
  intrinsics for SIMD).
- Breaking changes:
    - Added variant `U8x3` into the enum `PixelType`.
    - Changed internal tuple structures inside of variant of `ImageRows`
      and `ImageRowsMut` enums.

## [0.4.1] - 2021-11-13

- Added optimisation of convolution grayscale images (U8) with helps of `AVX2` instructions.

## [0.4.0] - 2021-10-23

- Added support of new type of pixels `PixelType::U8` (without forced SIMD).
- Breaking changes:
    - `ImageData` renamed into `Image`.
    - `SrcImageView` and `DstImageView` replaced by `ImageView`
      and `ImageViewMut`.
    - Method `Resizer.resize()` now returns `Result<(), DifferentTypesOfPixelsError>`.

## [0.3.1] - 2021-10-09

- Added support of compilation for architectures other than x86_64.

## [0.3.0] - 2021-08-28

- Added method `SrcImageView.set_crop_box_to_fit_dst_size()`.
- Fixed out-of-bounds error during resize with cropping.
- Refactored `ImageData`.
    - Added methods: `from_vec_u32()`, `from_vec_u8()`, `from_slice_u32()`,
      `from_slice_u8()`.
    - Removed methods: `from_buffer()`, `from_pixels()`.

## [0.2.0] - 2021-08-02

- Fixed typo in name of CatmullRom filter type.
