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
- Numbers in the table mean a duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  28.28  |   -   |  82.31   | 134.61  |  186.22  |
| resize     |  7.83   | 26.85 |  53.70   |  97.59  |  144.79  |
| libvips    |  7.76   | 59.65 |  19.83   |  30.73  |  39.80   |
| fir rust   |  0.28   | 12.77 |  17.94   |  27.94  |  40.05   |
| fir sse4.1 |  0.28   | 3.99  |   5.66   |  9.86   |  15.42   |
| fir avx2   |  0.28   | 3.06  |   3.89   |  6.85   |  13.25   |

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
| resize     |  11.34  | 42.76  |  85.43   | 147.41  |  211.31  |
| libvips    |  8.97   | 120.82 |  190.82  | 341.07  |  500.87  |
| fir rust   |  0.19   | 21.79  |  28.34   |  42.24  |  56.72   |
| fir sse4.1 |  0.19   | 10.14  |  12.26   |  18.43  |  24.22   |
| fir avx2   |  0.19   |  7.44  |   8.48   |  14.10  |  21.31   |

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
| image      |  25.53  |   -   |  56.37   |  84.03  |  111.89  |
| resize     |  5.60   | 10.85 |  18.72   |  37.97  |  65.65   |
| libvips    |  4.67   | 25.02 |   9.80   |  13.73  |  18.17   |
| fir rust   |  0.15   | 4.94  |   6.37   |  8.97   |  13.36   |
| fir sse4.1 |  0.15   | 1.66  |   2.10   |  3.28   |   5.57   |
| fir avx2   |  0.15   | 1.76  |   1.96   |  2.37   |   4.41   |

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
