# fast_image_resize

Rust library for fast image resizing with using of SIMD instructions.

[CHANGELOG](https://github.com/Cykooz/fast_image_resize/blob/main/CHANGELOG.md)

Supported pixel formats and available optimisations:
- `U8x4` - four `u8` components per pixel (RGB, RGBA, CMYK and other):
    - native Rust-code without forced SIMD
    - SSE4.1
    - AVX2
- `I32` - one `i32` component per pixel:
    - native Rust-code without forced SIMD
- `F32` - one `f32` component per pixel:
    - native Rust-code without forced SIMD
- `U8` - one `u8` component per pixel:
    - native Rust-code without forced SIMD
    - AVX2 

## Benchmarks

Environment:
- CPU: Intel(R) Core(TM) i7-6700K CPU @ 4.00GHz
- RAM: DDR4 3000 MHz
- Ubuntu 20.04 (linux 5.11)
- Rust 1.56.1
- fast_image_resize = "0.4"
- glassbench = "0.3.0"

Other Rust libraries used to compare of resizing speed:
- image = "0.23.14" (<https://crates.io/crates/image>)
- resize = "0.7.2" (<https://crates.io/crates/resize>)

Resize algorithms:
- Nearest
- Convolution with Bilinear filter
- Convolution with CatmullRom filter
- Convolution with Lanczos3 filter

### Resize RGB image 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      | 106.320 | 199.150  |  288.609   | 380.830  |
| resize     | 15.550  |  72.122  |  132.152   | 192.081  |
| fir rust   |  0.476  |  56.451  |   86.984   | 119.357  |
| fir sse4.1 |    -    |  11.798  |   17.768   |  25.296  |
| fir avx2   |    -    |  8.995   |   13.533   |  19.525  |

### Resize RGBA image 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      | 107.186 | 191.834  |  281.246   | 372.796  |
| resize     | 18.163  |  79.871  |  149.159   | 218.684  |
| fir rust   | 13.630  |  69.949  |  100.425   | 133.011  |
| fir sse4.1 | 12.034  |  23.566  |   29.809   |  37.232  |
| fir avx2   |  6.890  |  15.015  |   18.394   |  23.827  |

### Resize grayscale image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| image    | 92.171  | 141.153  |  184.442   | 230.455  |
| resize   |  9.890  |  26.205  |   53.054   |  81.181  |
| fir rust |  0.197  |  26.333  |   29.450   |  43.266  |
| fir avx2 |    -    |  14.766  |   12.567   |  16.641  |

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

    // Create wrapper that own data of destination image
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

```rust
use fast_image_resize as fr;

fn main() {
    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));
    unsafe {
        resizer.set_cpu_extensions(fr::CpuExtensions::Sse4_1);
    }
}
```
