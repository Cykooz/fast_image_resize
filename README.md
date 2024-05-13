# fast_image_resize

[![github](https://img.shields.io/badge/github-Cykooz%2Ffast__image__resize-8da0cb?logo=github)](https://github.com/Cykooz/fast_image_resize)
[![crates.io](https://img.shields.io/crates/v/fast_image_resize.svg?logo=rust)](https://crates.io/crates/fast_image_resize)
[![docs.rs](https://img.shields.io/badge/docs.rs-fast__image__resize-66c2a5?logo=docs.rs)](https://docs.rs/fast_image_resize)

Rust library for fast image resizing with using of SIMD instructions.

[CHANGELOG](https://github.com/Cykooz/fast_image_resize/blob/main/CHANGELOG.md)

Supported pixel formats and available optimisations:

| Format | Description                                                   | SSE4.1 | AVX2 | Neon | Wasm32 SIMD128 |
|:------:|:--------------------------------------------------------------|:------:|:----:|:----:|:--------------:|
|   U8   | One `u8` component per pixel (e.g. L)                         |   +    |  +   |  +   |       +        |
|  U8x2  | Two `u8` components per pixel (e.g. LA)                       |   +    |  +   |  +   |       +        |
|  U8x3  | Three `u8` components per pixel (e.g. RGB)                    |   +    |  +   |  +   |       +        |
|  U8x4  | Four `u8` components per pixel (e.g. RGBA, RGBx, CMYK)        |   +    |  +   |  +   |       +        |
|  U16   | One `u16` components per pixel (e.g. L16)                     |   +    |  +   |  +   |       +        |
| U16x2  | Two `u16` components per pixel (e.g. LA16)                    |   +    |  +   |  +   |       +        |
| U16x3  | Three `u16` components per pixel (e.g. RGB16)                 |   +    |  +   |  +   |       +        |
| U16x4  | Four `u16` components per pixel (e.g. RGBA16, RGBx16, CMYK16) |   +    |  +   |  +   |       +        |
|  I32   | One `i32` component per pixel                                 |   -    |  -   |  -   |       -        |
|  F32   | One `f32` component per pixel                                 |   -    |  -   |  -   |       -        |

## Colorspace

Resizer from this crate does not convert image into linear colorspace
during resize process. If it is important for you to resize images with a
non-linear color space (e.g. sRGB) correctly, then you have to convert
it to a linear color space before resizing and convert back to the color space of
result image. [Read more](http://www.ericbrasseur.org/gamma.html)
about resizing with respect to color space.

This crate provides the
[PixelComponentMapper](https://docs.rs/fast_image_resize/latest/fast_image_resize/struct.PixelComponentMapper.html)
structure that allows you to create colorspace converters for images
whose pixels based on `u8` and `u16` components.

In addition, the crate contains functions `create_gamma_22_mapper()`
and `create_srgb_mapper()` to create instance of `PixelComponentMapper`
that converts images from sRGB or gamma 2.2 into linear colorspace and back.

## Some benchmarks for x86_64

_All benchmarks:_
[_x86_64_](https://github.com/Cykooz/fast_image_resize/blob/main/benchmarks-x86_64.md),
[_ARM64_](https://github.com/Cykooz/fast_image_resize/blob/main/benchmarks-arm64.md),
[_WASM32_](https://github.com/Cykooz/fast_image_resize/blob/main/benchmarks-wasm32.md).

Other libraries used to compare of resizing speed:

- image (<https://crates.io/crates/image>)
- resize (<https://crates.io/crates/resize>)
- libvips (single-threaded mode, cache disabled)

<!-- bench_compare_rgb start -->
### Resize RGB8 image (U8x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  28.20  |   -   |  82.45   | 134.07  |  192.70  |
| resize     |    -    | 26.83 |  53.56   |  97.73  |  144.63  |
| libvips    |  7.73   | 60.66 |  19.84   |  30.15  |  39.46   |
| fir rust   |  0.28   | 9.78  |  15.46   |  27.36  |  39.57   |
| fir sse4.1 |  0.28   | 3.87  |   5.59   |  9.89   |  15.44   |
| fir avx2   |  0.28   | 2.67  |   3.54   |  6.96   |  13.22   |
<!-- bench_compare_rgb end -->

<!-- bench_compare_rgba start -->
### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize     |    -    | 42.96  |  85.43   | 147.79  |  211.49  |
| libvips    |  10.06  | 122.80 |  188.97  | 338.18  |  499.99  |
| fir rust   |  0.19   | 20.10  |  27.08   |  41.32  |  56.79   |
| fir sse4.1 |  0.19   | 10.03  |  12.24   |  18.57  |  25.15   |
| fir avx2   |  0.19   |  6.98  |   8.26   |  13.97  |  21.55   |
<!-- bench_compare_rgba end -->

<!-- bench_compare_l start -->
### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  25.96  |   -   |  56.78   |  84.17  |  112.12  |
| resize     |    -    | 10.67 |  18.54   |  39.06  |  62.71   |
| libvips    |  4.72   | 24.93 |   9.70   |  13.68  |  18.07   |
| fir rust   |  0.15   | 4.08  |   5.24   |  7.48   |  11.33   |
| fir sse4.1 |  0.15   | 1.86  |   2.30   |  3.58   |   5.88   |
| fir avx2   |  0.15   | 1.66  |   1.86   |  2.24   |   4.21   |
<!-- bench_compare_l end -->

## Examples

### Resize RGBA8 image

```rust
use std::io::BufWriter;
use std::num::NonZeroU32;

use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{ExtendedColorType, ImageEncoder};

use fast_image_resize as fr;

fn main() {
    // Read source image from file
    let img = ImageReader::open("./data/nasa-4928x3279.png")
        .unwrap()
        .decode()
        .unwrap();
    let width = NonZeroU32::new(img.width()).unwrap();
    let height = NonZeroU32::new(img.height()).unwrap();
    let mut src_image = fr::Image::from_vec_u8(
        width,
        height,
        img.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    ).unwrap();

    // Multiple RGB channels of source image by alpha channel 
    // (not required for the Nearest algorithm)
    let alpha_mul_div = fr::MulDiv::default();
    alpha_mul_div
        .multiply_alpha_inplace(&mut src_image.view_mut())
        .unwrap();

    // Create container for data of destination image
    let dst_width = NonZeroU32::new(1024).unwrap();
    let dst_height = NonZeroU32::new(768).unwrap();
    let mut dst_image = fr::Image::new(
        dst_width,
        dst_height,
        src_image.pixel_type(),
    );

    // Get mutable view of destination image data
    let mut dst_view = dst_image.view_mut();

    // Create Resizer instance and resize source image
    // into buffer of destination image
    let mut resizer = fr::Resizer::new(
        fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3),
    );
    resizer.resize(&src_image.view(), &mut dst_view).unwrap();

    // Divide RGB channels of destination image by alpha
    alpha_mul_div.divide_alpha_inplace(&mut dst_view).unwrap();

    // Write destination image as PNG-file
    let mut result_buf = BufWriter::new(Vec::new());
    PngEncoder::new(&mut result_buf)
        .write_image(
            dst_image.buffer(),
            dst_width.get(),
            dst_height.get(),
            ExtendedColorType::Rgba8,
        )
        .unwrap();
}
```

### Resize with cropping

```rust
use std::num::NonZeroU32;

use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{ColorType, GenericImageView};

use fast_image_resize as fr;

fn resize_image_with_cropping(
    mut src_view: fr::DynamicImageView,
    dst_width: NonZeroU32,
    dst_height: NonZeroU32
) -> fr::Image {
    // Set cropping parameters
    src_view.set_crop_box_to_fit_dst_size(
        dst_width, 
        dst_height, 
        None,
    );

    // Create container for data of destination image
    let mut dst_image = fr::Image::new(
        dst_width,
        dst_height,
        src_view.pixel_type(),
    );
    // Get mutable view of destination image data
    let mut dst_view = dst_image.view_mut();

    // Create Resizer instance and resize source image
    // into buffer of destination image
    let mut resizer = fr::Resizer::new(
        fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3)
    );
    resizer.resize(&src_view, &mut dst_view).unwrap();

    dst_image
}

fn main() {
    let img = ImageReader::open("./data/nasa-4928x3279.png")
        .unwrap()
        .decode()
        .unwrap();
    let width = NonZeroU32::new(img.width()).unwrap();
    let height = NonZeroU32::new(img.height()).unwrap();
    let src_image = fr::Image::from_vec_u8(
        width,
        height,
        img.to_rgba8().into_raw(),
        fr::PixelType::U8x4,
    ).unwrap();
    resize_image_with_cropping(
        src_image.view(),
        NonZeroU32::new(1024).unwrap(),
        NonZeroU32::new(768).unwrap(),
    );
}
```

### Change CPU extensions used by resizer

```rust, ignore
use fast_image_resize as fr;

fn main() {
    let mut resizer = fr::Resizer::new(
        fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3),
    );
    unsafe {
        resizer.set_cpu_extensions(fr::CpuExtensions::Sse4_1);
    }
}
```
