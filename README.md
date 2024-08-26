# fast_image_resize

[![github](https://img.shields.io/badge/github-Cykooz%2Ffast__image__resize-8da0cb?logo=github)](https://github.com/Cykooz/fast_image_resize)
[![crates.io](https://img.shields.io/crates/v/fast_image_resize.svg?logo=rust)](https://crates.io/crates/fast_image_resize)
[![docs.rs](https://img.shields.io/badge/docs.rs-fast__image__resize-66c2a5?logo=docs.rs)](https://docs.rs/fast_image_resize)

Rust library for fast image resizing with using of SIMD instructions.

[CHANGELOG](https://github.com/Cykooz/fast_image_resize/blob/main/CHANGELOG.md)

Supported pixel formats and available optimizations:

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
|  I32   | One `i32` component per pixel (e.g. L32)                      |   -    |  -   |  -   |       -        |
|  F32   | One `f32` component per pixel (e.g. L32F)                     |   +    |  +   |  -   |       -        |
| F32x2  | Two `f32` components per pixel (e.g. LA32F)                   |   +    |  +   |  -   |       -        |
| F32x3  | Three `f32` components per pixel (e.g. RGB32F)                |   +    |  +   |  -   |       -        |
| F32x4  | Four `f32` components per pixel (e.g. RGBA32F)                |   +    |  +   |  -   |       -        |

## Colorspace

Resizer from this crate does not convert image into linear colorspace
during a resize process. If it is important for you to resize images with a
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

## Multi-threading

You should enable `"rayon"` feature to turn on image processing in
[rayon](https://docs.rs/rayon/latest/rayon/) thread pool.

## Some benchmarks in single-threaded mode for x86_64

_All benchmarks:_
[_x86_64_](https://github.com/Cykooz/fast_image_resize/blob/main/benchmarks-x86_64.md),
[_ARM64_](https://github.com/Cykooz/fast_image_resize/blob/main/benchmarks-arm64.md),
[_WASM32_](https://github.com/Cykooz/fast_image_resize/blob/main/benchmarks-wasm32.md).

Other libraries used to compare of resizing speed:

- image (<https://crates.io/crates/image>)
- resize (<https://crates.io/crates/resize>, single-threaded mode)
- libvips (single-threaded mode)

<!-- bench_compare_rgb start -->

### Resize RGB8 image (U8x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in the table mean a duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  34.13  |   -   |  88.09   | 142.99  |  191.80  |
| resize     |  8.87   | 26.97 |  53.08   |  98.26  |  145.74  |
| libvips    |  2.38   | 61.57 |   5.66   |  9.70   |  16.03   |
| fir rust   |  0.28   | 10.93 |  15.35   |  25.77  |  37.09   |
| fir sse4.1 |  0.28   | 3.43  |   5.39   |  9.82   |  15.34   |
| fir avx2   |  0.28   | 2.62  |   3.80   |  6.89   |  13.22   |

<!-- bench_compare_rgb end -->

<!-- bench_compare_rgba start -->

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize     |  9.90   | 37.85  |  74.20   | 133.72  |  201.29  |
| libvips    |  4.17   | 169.03 |  141.52  | 232.30  |  330.89  |
| fir rust   |  0.19   | 20.66  |  26.02   |  37.27  |  50.21   |
| fir sse4.1 |  0.19   |  9.59  |  11.99   |  17.79  |  24.83   |
| fir avx2   |  0.19   |  7.21  |   8.61   |  13.22  |  22.41   |

<!-- bench_compare_rgba end -->

<!-- bench_compare_l start -->

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in the table mean a duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  29.07  |   -   |  60.25   |  89.15  |  117.51  |
| resize     |  6.42   | 11.26 |  20.87   |  42.87  |  69.50   |
| libvips    |  2.57   | 25.05 |   6.82   |  9.85   |  12.68   |
| fir rust   |  0.15   | 4.45  |   5.57   |  9.02   |  12.31   |
| fir sse4.1 |  0.15   | 1.52  |   2.09   |  3.52   |   5.65   |
| fir avx2   |  0.15   | 1.54  |   1.76   |  2.80   |   4.03   |

<!-- bench_compare_l end -->

## Examples

### Resize RGBA8 image

Note: You must enable `"image"` feature to support of
[image::DynamicImage](https://docs.rs/image/latest/image/enum.DynamicImage.html).
Otherwise, you have to convert such images into supported by the crate image type.

```rust
use std::io::BufWriter;

use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, ImageReader};

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
            src_image.color().into(),
        )
        .unwrap();
}
```

### Resize with cropping

```rust
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageReader, GenericImageView};

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

```rust, no_run
use fast_image_resize as fr;

fn main() {
    let mut resizer = fr::Resizer::new();
    #[cfg(target_arch = "x86_64")]
    unsafe {
        resizer.set_cpu_extensions(fr::CpuExtensions::Sse4_1);
    }
}
```
