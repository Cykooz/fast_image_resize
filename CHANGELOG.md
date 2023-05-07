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
