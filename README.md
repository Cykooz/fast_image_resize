# fast_image_resize

Rust library for fast image resizing with using of SIMD instructions.

_Note: This library does not support converting image color spaces.
If it is important for you to resize images with a non-linear color space
(e.g. sRGB) correctly, then you need to convert it to a linear color space
before resizing. [Read more](https://legacy.imagemagick.org/Usage/resize/#resize_colorspace)
about resizing with respect to color space._

[CHANGELOG](https://github.com/Cykooz/fast_image_resize/blob/main/CHANGELOG.md)

Supported pixel formats and available optimisations:
- `U8` - one `u8` component per pixel:
    - native Rust-code without forced SIMD
    - SSE4.1 (partial)
    - AVX2
- `U8x3` - three `u8` components per pixel (e.g. RGB):
    - native Rust-code without forced SIMD
    - SSE4.1 (partial)
    - AVX2
- `U8x4` - four `u8` components per pixel (RGBA, RGBx, CMYK and other):
    - native Rust-code without forced SIMD
    - SSE4.1
    - AVX2
- `U16x3` - three `u16` components per pixel (e.g. RGB):
    - native Rust-code without forced SIMD
    - SSE4.1
    - AVX2
- `I32` - one `i32` component per pixel:
    - native Rust-code without forced SIMD
- `F32` - one `f32` component per pixel:
    - native Rust-code without forced SIMD

## Benchmarks

Environment:
- CPU: Intel(R) Core(TM) i7-6700K CPU @ 4.00GHz
- RAM: DDR4 3000 MHz
- Ubuntu 20.04 (linux 5.13)
- Rust 1.59.0
- fast_image_resize = "0.8.0"
- glassbench = "0.3.1"
- `rustflags = ["-C", "llvm-args=-x86-branches-within-32B-boundaries"]`

Other Rust libraries used to compare of resizing speed:
- image = "0.24.1" (<https://crates.io/crates/image>)
- resize = "0.7.2" (<https://crates.io/crates/resize>)

Resize algorithms:
- Nearest
- Convolution with Bilinear filter
- Convolution with CatmullRom filter
- Convolution with Lanczos3 filter

### Resize RGB image (U8x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  53.21  |  116.37  |   191.14   |  271.65  |
| resize     |  15.30  |  65.72   |   120.28   |  175.05  |
| fir rust   |  0.49   |  54.24   |   90.68    |  131.56  |
| fir sse4.1 |    -    |  37.70   |   43.88    |  61.05   |
| fir avx2   |    -    |  10.92   |   14.15    |  19.81   |

### Resize RGBA image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  53.09  |  110.62  |   181.02   |  260.38  |
| resize     |  18.28  |  79.18   |   148.75   |  218.27  |
| fir rust   |  12.08  |  62.61   |   87.89    |  116.05  |
| fir sse4.1 |  9.13   |  20.19   |   26.36    |  33.99   |
| fir avx2   |  7.37   |  15.23   |   18.72    |  24.08   |

### Resize grayscale image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  48.05  |  72.53   |   109.80   |  161.38  |
| resize     |  9.34   |  24.47   |   47.70    |  80.72   |
| fir rust   |  0.20   |  19.53   |   22.60    |  32.46   |
| fir sse4.1 |    -    |  17.41   |   18.66    |  27.81   |
| fir avx2   |    -    |   9.58   |    7.74    |  11.70   |

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  52.81  |  113.30  |   180.99   |  256.26  |
| resize     |  17.05  |  68.81   |   121.87   |  175.41  |
| fir rust   |  0.79   |  58.25   |   96.49    |  132.55  |
| fir sse4.1 |    -    |  39.03   |   63.09    |  89.33   |
| fir avx2   |    -    |  33.29   |   48.96    |  58.77   |

## Examples

### Resize image

```rust
use std::io::BufWriter;
use std::num::NonZeroU32;

use image::codecs::png::PngEncoder;
use image::io::Reader as ImageReader;
use image::{ColorType, GenericImageView};

use fast_image_resize as fr;

#[test]
fn resize_image_example() {
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

    // Create MulDiv instance
    let alpha_mul_div = fr::MulDiv::default();
    // Multiple RGB channels of source image by alpha channel
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
        fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3)
    );
    resizer.resize(&src_image.view(), &mut dst_view).unwrap();

    // Divide RGB channels of destination image by alpha
    alpha_mul_div.divide_alpha_inplace(&mut dst_view).unwrap();

    // Write destination image as PNG-file
    let mut result_buf = BufWriter::new(Vec::new());
    PngEncoder::new(&mut result_buf)
        .encode(
            dst_image.buffer(),
            dst_width.get(),
            dst_height.get(),
            ColorType::Rgba8,
        )
        .unwrap();
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
