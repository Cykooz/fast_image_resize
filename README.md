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
- Numbers in table are mean duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  30.14  |   -   |  90.74   | 149.25  |  208.22  |
| resize     |  7.78   | 26.82 |  53.54   |  97.38  |  144.44  |
| libvips    |  7.78   | 59.56 |  18.69   |  30.36  |  39.69   |
| fir rust   |  0.28   | 9.17  |  14.72   |  26.24  |  38.98   |
| fir sse4.1 |  0.28   | 4.08  |   5.79   |  10.32  |  15.94   |
| fir avx2   |  0.28   | 3.01  |   3.86   |  6.89   |  12.69   |

<!-- bench_compare_rgb end -->

<!-- bench_compare_rgba start -->

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table are mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize     |  11.30  | 42.85  |  85.27   | 147.28  |  211.34  |
| libvips    |  9.15   | 120.11 |  188.46  | 337.77  |  499.37  |
| fir rust   |  0.20   | 20.69  |  27.88   |  41.83  |  56.67   |
| fir sse4.1 |  0.19   | 10.19  |  12.43   |  17.99  |  24.64   |
| fir avx2   |  0.20   |  7.55  |   8.77   |  13.45  |  20.62   |

<!-- bench_compare_rgba end -->

<!-- bench_compare_l start -->

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table are mean duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  27.05  |   -   |  58.63   |  86.87  |  115.57  |
| resize     |  6.44   | 11.49 |  21.83   |  43.93  |  71.01   |
| libvips    |  4.69   | 25.00 |   9.69   |  12.95  |  16.46   |
| fir rust   |  0.15   | 3.97  |   4.98   |  7.15   |  11.04   |
| fir sse4.1 |  0.15   | 1.69  |   2.13   |  3.32   |   5.71   |
| fir avx2   |  0.15   | 1.73  |   1.94   |  2.30   |   4.33   |

<!-- bench_compare_l end -->

## Examples

### Resize RGBA8 image

Note: You must enable `"image"` feature to support of
[image::DynamicImage](https://docs.rs/image/latest/image/enum.DynamicImage.html).
Otherwise, you have to convert such images into supported by the crate image type.

```rust
use std::io::BufWriter;

use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{ExtendedColorType, ImageEncoder};

use fast_image_resize::{IntoImageView, Resizer};
use fast_image_resize::images::Image;

fn main() {
    // Read source image from file
    let src_image = ImageReader::open("./data/nasa-4928x3279.png")
        .unwrap()
        .decode()
        .unwrap();

    // Create container for data of destination image
    let dst_width = 1024;
    let dst_height = 768;
    let mut dst_image = Image::new(
        dst_width,
        dst_height,
        src_image.pixel_type().unwrap(),
    );

    // Create Resizer instance and resize source image
    // into buffer of destination image
    let mut resizer = Resizer::new();
    resizer.resize(&src_image, &mut dst_image, None).unwrap();

    // Write destination image as PNG-file
    let mut result_buf = BufWriter::new(Vec::new());
    PngEncoder::new(&mut result_buf)
        .write_image(
            dst_image.buffer(),
            dst_width,
            dst_height,
            ExtendedColorType::Rgba8,
        )
        .unwrap();
}
```

### Resize with cropping

```rust
use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{ColorType, GenericImageView};

use fast_image_resize::{IntoImageView, Resizer, ResizeOptions};
use fast_image_resize::images::Image;

fn main() {
    let img = ImageReader::open("./data/nasa-4928x3279.png")
        .unwrap()
        .decode()
        .unwrap();

    // Create container for data of destination image
    let mut dst_image = Image::new(
        1024,
        768,
        img.pixel_type().unwrap(),
    );

    // Create Resizer instance and resize cropped source image
    // into buffer of destination image
    let mut resizer = Resizer::new();
    resizer.resize(
        &img,
        &mut dst_image,
        &ResizeOptions::new().crop(
            10.0,   // left 
            10.0,   // top
            2000.0, // width
            2000.0, // height
        ),
    ).unwrap();
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
