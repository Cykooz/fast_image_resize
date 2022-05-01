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
- `U8x2` - two `u8` components per pixel (e.g. LA):
    - native Rust-code without forced SIMD
    - SSE4.1 (partial)
    - AVX2 (partial)
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

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.15.0)
- Rust 1.60.0
- fast_image_resize = "0.9.0"
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

### Resize RGB8 image (U8x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  20.53  |  77.41   |   135.87   |  184.40  |
| resize     |    -    |  45.60   |   87.53    |  129.11  |
| fir rust   |  0.26   |  37.42   |   63.68    |  92.93   |
| fir sse4.1 |  0.26   |  25.76   |   39.52    |  52.82   |
| fir avx2   |  0.26   |   6.89   |    8.69    |  12.64   |

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  20.71  |  77.57   |   131.22   |  183.61  |
| resize     |    -    |  53.38   |   101.44   |  150.50  |
| fir rust   |  0.17   |  33.06   |   47.36    |  68.15   |
| fir sse4.1 |  0.17   |  12.00   |   15.64    |  20.46   |
| fir avx2   |  0.17   |   9.13   |   11.38    |  15.12   |

### Resize grayscale image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  17.09  |  47.71   |   75.01    |  102.30  |
| resize     |    -    |  16.23   |   32.66    |  55.36   |
| fir rust   |  0.14   |  13.00   |   14.74    |  22.12   |
| fir sse4.1 |  0.14   |  11.06   |   11.03    |  16.75   |
| fir avx2   |  0.13   |   5.90   |    4.39    |   7.12   |

### Resize grayscale image with alpha channel (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (two bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `resize` crate does not support this pixel format.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  18.96  |  62.93   |   112.37   |  149.47  |
| fir rust   |  0.17   |  23.41   |   27.85    |  39.30   |
| fir sse4.1 |  0.17   |  19.56   |   20.62    |  28.77   |
| fir avx2   |  0.17   |  19.41   |   20.55    |  28.24   |

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  20.31  |  72.79   |   122.71   |  171.64  |
| resize     |    -    |  47.23   |   90.10    |  132.22  |
| fir rust   |  0.30   |  42.43   |   78.20    |  115.72  |
| fir sse4.1 |  0.31   |  22.08   |   35.60    |  50.43   |
| fir avx2   |  0.31   |  19.07   |   28.24    |  34.08   |

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

    // Create MulDiv instance
    let alpha_mul_div = fr::MulDiv::default();
    // Multiple RGB channels of source image by alpha channel 
    // (not required for the Nearest algorithm) 
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
