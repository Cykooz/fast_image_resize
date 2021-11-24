## [Unreleased] - ReleaseDate

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

- Added optimisation of convolution grayscale images (U8) with helps of ``AVX2`` instructions.

## [0.4.0] - 2021-10-23

- Added support of new type of pixels `PixelType::U8` (without forced SIMD).
- Breaking changes:
  - ``ImageData`` renamed into ``Image``.
  - ``SrcImageView`` and ``DstImageView`` replaced by ``ImageView``
    and ``ImageViewMut``.
  - Method ``Resizer.resize()`` now returns ``Result<(), DifferentTypesOfPixelsError>``.

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
