# fast_image_resize

Rust library for fast image resizing with using of SIMD instructions.

[CHANGELOG](https://github.com/Cykooz/fast_image_resize/blob/main/CHANGELOG.md)

Supported pixel formats and available optimisations:
- `U8x4` - four `u8` components per pixel:
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
- Ubuntu 20.04 (linux 5.8)
- Rust 1.54
- fast_image_resize = "0.3"
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
| image      | 106.303 | 199.011  |  291.423   | 382.793  |
| resize     | 16.099  |  71.947  |  132.481   | 205.961  |
| fir rust   |  0.478  |  55.376  |   84.567   | 117.269  |
| fir sse4.1 |    -    |  11.847  |   17.823   |  25.459  |
| fir avx2   |    -    |  8.598   |   11.864   |  17.549  |

Compiled with `rustflags = ["-C", "target-cpu=native"]`

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      | 91.787  | 182.502  |  282.821   | 385.492  |
| resize     | 15.908  |  62.011  |  114.741   | 167.815  |
| fir rust   |  0.471  |  56.330  |   61.589   |  85.443  |
| fir sse4.1 |    -    |  11.204  |   16.510   |  23.442  |
| fir avx2   |    -    |  8.402   |   11.550   |  16.987  |

### Resize RGBA image 4928x3279 => 852x567

Pipeline: 

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      | 102.564 | 205.981  |  309.851   | 423.252  |
| resize     | 19.003  |  92.075  |  169.876   | 247.993  |
| fir rust   | 13.619  |  67.097  |   97.157   | 125.865  |
| fir sse4.1 | 12.182  |  23.555  |   29.506   |  37.118  |
| fir avx2   |  6.931  |  15.052  |   18.391   |  23.812  |

Compiled with `rustflags = ["-C", "target-cpu=native"]`

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      | 90.421  | 193.379  |  306.635   | 422.674  |
| resize     | 19.572  |  69.091  |  130.327   | 192.235  |
| fir rust   |  9.987  |  69.005  |   76.551   | 103.646  |
| fir sse4.1 |  7.860  |  18.375  |   23.601   |  30.603  |
| fir avx2   |  6.897  |  14.907  |   18.034   |  23.677  |

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
    let mut src_image = fr::ImageData::from_vec_u8(
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
        .multiply_alpha_inplace(&mut src_image.dst_view())
        .unwrap();

    // Create wrapper that own data of destination image
    let dst_width = NonZeroU32::new(1024).unwrap();
    let dst_height = NonZeroU32::new(768).unwrap();
    let mut dst_image = fr::ImageData::new(dst_width, dst_height, src_image.pixel_type());

    // Get mutable view of destination image data
    let mut dst_view = dst_image.dst_view();

    // Create Resizer instance and resize source image
    // into buffer of destination image
    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));
    resizer.resize(&src_image.src_view(), &mut dst_view);

    // Divide RGB channels of destination image by alpha
    alpha_mul_div.divide_alpha_inplace(&mut dst_view).unwrap();

    // Write destination image as PNG-file
    let mut result_buf = BufWriter::new(Vec::new());
    let encoder = PngEncoder::new(&mut result_buf);
    encoder
        .encode(
            dst_image.get_buffer(),
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
    // ...
}
```
