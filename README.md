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

## Rust version

Rust 1.95-1.98 (actually LLVM) have [regression](https://github.com/rust-lang/rust/issues/157456)
in Wasm32 SIMD128 implementation. It affects the implementation of dividing U8x4 pixels by
alpha channel for Wasm32 SIMD128.

## Colorspace

Resizer from this crate does not convert image into linear colorspace during a resize
process. If it is important for you to resize images with a non-linear color space (e.g.
sRGB) correctly, then you have to convert it to a linear color space before resizing and
convert back to the color space of the result
image. [Read more](http://www.ericbrasseur.org/gamma.html) about resizing with respect to
color space.

This crate provides the [PixelComponentMapper](https://docs.rs/fast_image_resize/latest/fast_image_resize/struct.PixelComponentMapper.html)
structure that allows you to create colorspace converters for images whose pixels are
based on `u8` and `u16` components.

In addition, the crate contains functions `create_gamma_22_mapper()`
and `create_srgb_mapper()` to create an instance of `PixelComponentMapper`
that converts images from sRGB or gamma 2.2 into linear colorspace and back.

## Multi-threading

You should enable `"rayon"` feature to turn on image processing in
[rayon](https://docs.rs/rayon/latest/rayon/) thread pool.

## Use in a `no_std` environment

To use the crate in a `no_std` environment you must disable default features and enable
the `no_std` feature:

```toml
[dependencies]
fast_image_resize = { version = "6.2", default-features = false, features = ["no_std"] }
```

## Some benchmarks in single-threaded mode for x86_64

_All benchmarks:_
[
_x86_64_](https://github.com/Cykooz/fast_image_resize/blob/main/benchmarks-x86_64.md),
[
_ARM64_](https://github.com/Cykooz/fast_image_resize/blob/main/benchmarks-arm64.md),
[
_WASM32_](https://github.com/Cykooz/fast_image_resize/blob/main/benchmarks-wasm32.md).

Other libraries used to compare of resizing speed:

- image (<https://crates.io/crates/image>)
- resize (<https://crates.io/crates/resize>, single-threaded mode)
- libvips (single-threaded mode)

<!-- bench_compare_rgb start -->

### Resize RGB8 image (U8x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in the table mean the duration of the image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  28.92  |   -   |  78.41   | 127.38  |  173.98  |
| resize     |  8.49   | 24.35 |  48.44   |  92.29  |  137.61  |
| libvips    |  2.41   | 61.63 |   5.67   |  9.76   |  16.07   |
| fir rust   |  0.28   | 10.86 |  15.71   |  26.43  |  37.47   |
| fir sse4.1 |  0.28   | 3.94  |   5.80   |  10.40  |  15.67   |
| fir avx2   |  0.28   | 2.63  |   3.53   |  7.23   |  13.26   |

<!-- bench_compare_rgb end -->

<!-- bench_compare_rgba start -->

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean the duration of the image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize     |  13.97  | 44.38  |  88.03   | 150.33  |  216.80  |
| libvips    |  4.19   | 169.24 |  137.35  | 228.62  |  329.39  |
| fir rust   |  0.19   | 20.99  |  26.40   |  38.18  |  51.62   |
| fir sse4.1 |  0.19   | 11.28  |  13.44   |  19.69  |  26.88   |
| fir avx2   |  0.19   |  9.90  |  10.69   |  15.47  |  24.48   |

<!-- bench_compare_rgba end -->

<!-- bench_compare_l start -->

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in the table mean the duration of the image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  26.33  |   -   |  57.97   |  86.30  |  113.91  |
| resize     |  6.59   | 11.63 |  21.04   |  45.28  |  68.73   |
| libvips    |  2.66   | 24.92 |   6.83   |  9.81   |  12.72   |
| fir rust   |  0.16   | 4.23  |   5.48   |  8.74   |  12.13   |
| fir sse4.1 |  0.16   | 1.62  |   2.25   |  3.68   |   5.88   |
| fir avx2   |  0.16   | 1.78  |   1.81   |  2.87   |   4.08   |

<!-- bench_compare_l end -->

## Examples

### Resize RGBA8 image

Note: You must enable `"image"` feature to support of
[image::DynamicImage](https://docs.rs/image/latest/image/enum.DynamicImage.html).
Otherwise, you have to convert such images into a supported by the crate image type.

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
