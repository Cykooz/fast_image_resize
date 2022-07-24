# fast_image_resize

[![github](https://img.shields.io/badge/github-Cykooz%2Ffast__image__resize-8da0cb?logo=github)](https://github.com/Cykooz/fast_image_resize)
[![crates.io](https://img.shields.io/crates/v/fast_image_resize.svg?logo=rust)](https://crates.io/crates/fast_image_resize)
[![docs.rs](https://img.shields.io/badge/docs.rs-fast__image__resize-66c2a5?logo=docs.rs)](https://docs.rs/fast_image_resize)

Rust library for fast image resizing with using of SIMD instructions.

_Note: This library does not support converting image color spaces.
If it is important for you to resize images with a non-linear color space
(e.g. sRGB) correctly, then you need to convert it to a linear color space
before resizing. [Read more](https://legacy.imagemagick.org/Usage/resize/#resize_colorspace)
about resizing with respect to color space._

[CHANGELOG](https://github.com/Cykooz/fast_image_resize/blob/main/CHANGELOG.md)

Supported pixel formats and available optimisations:

| Format | Description                                                   | Native Rust | SSE4.1  | AVX2 |
|:------:|:--------------------------------------------------------------|:-----------:|:-------:|:----:|
|   U8   | One `u8` component per pixel (e.g. L)                         |      +      | partial |  +   |
|  U8x2  | Two `u8` components per pixel (e.g. LA)                       |      +      |    +    |  +   |
|  U8x3  | Three `u8` components per pixel (e.g. RGB)                    |      +      | partial |  +   |
|  U8x4  | Four `u8` components per pixel (e.g. RGBA, RGBx, CMYK)        |      +      |    +    |  +   |
|  U16   | One `u16` components per pixel (e.g. L16)                     |      +      |    +    |  +   |
| U16x2  | Two `u16` components per pixel (e.g. LA16)                    |      +      |    +    |  +   |
| U16x3  | Three `u16` components per pixel (e.g. RGB16)                 |      +      |    +    |  +   |
| U16x4  | Four `u16` components per pixel (e.g. RGBA16, RGBx16, CMYK16) |      +      |    +    |  +   |
|  I32   | One `i32` component per pixel                                 |      +      |    -    |  -   |
|  F32   | One `f32` component per pixel                                 |      +      |    -    |  -   |

## Some benchmarks

[All benchmarks.](https://github.com/Cykooz/fast_image_resize/blob/main/benchmarks.md)

Rust libraries used to compare of resizing speed:

- image (<https://crates.io/crates/image>)
- resize (<https://crates.io/crates/resize>)

### Resize RGB8 image (U8x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  19.01  |  84.06   |   148.84   |  205.25  |
| resize     |    -    |  52.21   |   104.32   |  154.11  |
| fir rust   |  0.28   |  41.75   |   76.85    |  113.11  |
| fir sse4.1 |  0.28   |  28.16   |   43.00    |  59.62   |
| fir avx2   |  0.28   |   7.41   |    9.61    |  13.84   |

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  61.94   |   122.19   |  182.60  |
| fir rust   |  0.19   |  37.33   |   53.29    |  76.49   |
| fir sse4.1 |  0.19   |  13.14   |   17.25    |  22.59   |
| fir avx2   |  0.19   |   9.59   |   12.09    |  16.18   |

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  15.60  |  45.30   |   72.95    |  102.13  |
| resize     |    -    |  17.29   |   35.45    |  61.14   |
| fir rust   |  0.15   |  13.91   |   15.37    |  22.86   |
| fir sse4.1 |  0.15   |  12.16   |   12.16    |  18.19   |
| fir avx2   |  0.15   |   6.33   |    4.66    |   7.69   |

## Examples

### Resize RGBA8 image

```rust
use std::io::BufWriter;
use std::num::NonZeroU32;

use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{ColorType, ImageEncoder};

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
    )
        .unwrap();

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
            ColorType::Rgba8,
        )
        .unwrap();
}
```

### Resize with cropping

```rust
use std::io::BufWriter;
use std::num::NonZeroU32;

use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{ColorType, GenericImageView};

use fast_image_resize as fr;

fn resize_image_with_cropping(
    mut src_view: fr::ImageView,
    dst_width: NonZeroU32,
    dst_height: NonZeroU32
) -> fr::Image {
    // Set cropping parameters
    src_view.set_crop_box_to_fit_dst_size(dst_width, dst_height, None);

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
        img.to_rgb8().into_raw(),
        fr::PixelType::U8x3,
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
