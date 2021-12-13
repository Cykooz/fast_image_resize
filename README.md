# fast_image_resize

Rust library for fast image resizing with using of SIMD instructions.

[CHANGELOG](https://github.com/Cykooz/fast_image_resize/blob/main/CHANGELOG.md)

Supported pixel formats and available optimisations:
- `U8` - one `u8` component per pixel:
    - native Rust-code without forced SIMD
    - AVX2
- `U8x3` - three `u8` components per pixel (e.g. RGB):
    - native Rust-code without forced SIMD
    - SSE4.1 (auto-vectorization)
    - AVX2
- `U8x4` - four `u8` components per pixel (RGBA, RGBx, CMYK and other):
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
- Ubuntu 20.04 (linux 5.11)
- Rust 1.57.0
- fast_image_resize = "0.5.3"
- glassbench = "0.3.1"
- `rustflags = ["-C", "llvm-args=-x86-branches-within-32B-boundaries"]`

Other Rust libraries used to compare of resizing speed:
- image = "0.23.14" (<https://crates.io/crates/image>)
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
| image      | 91.896  | 176.959  |  256.786   | 341.548  |
| resize     | 15.453  |  71.340  |  131.232   | 191.469  |
| fir rust   |  0.481  |  53.733  |   90.519   | 121.121  |
| fir sse4.1 |    -    |  43.113  |   53.484   |  75.127  |
| fir avx2   |    -    |  10.765  |   14.131   |  19.827  |

### Resize RGBA image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      | 98.113  | 177.039  |  254.666   | 337.147  |
| resize     | 17.875  |  79.014  |  148.691   | 218.400  |
| fir rust   | 13.188  |  63.942  |   89.681   | 119.664  |
| fir sse4.1 | 11.868  |  22.957  |   29.164   |  36.799  |
| fir avx2   |  6.949  |  14.854  |   18.399   |  23.772  |

### Resize grayscale image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| image    | 76.981  | 126.595  |  166.765   | 209.593  |
| resize   |  9.632  |  24.332  |   47.533   |  80.667  |
| fir rust |  0.197  |  21.773  |   24.476   |  34.909  |
| fir avx2 |    -    |  9.467   |   7.691    |  11.776  |

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
    let alpha_mul_div: fr::MulDiv = Default::default();
    // Multiple RGB channels of source image by alpha channel
    alpha_mul_div
        .multiply_alpha_inplace(&mut src_image.view_mut())
        .unwrap();

    // Create container for data of destination image
    let dst_width = NonZeroU32::new(1024).unwrap();
    let dst_height = NonZeroU32::new(768).unwrap();
    let mut dst_image = fr::Image::new(dst_width, dst_height, src_image.pixel_type());

    // Get mutable view of destination image data
    let mut dst_view = dst_image.view_mut();

    // Create Resizer instance and resize source image
    // into buffer of destination image
    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));
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

```ignore
use fast_image_resize as fr;

fn main() {
    le mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(fr::CpuExtensions::Sse4_1);
    }
}
```
