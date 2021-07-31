# fast_image_resize

Rust library for fast image resizing with using of SIMD instructions.

[CHANGELOG](https://github.com/Cykooz/fast_image_resize/blob/master/CHANGELOG.md)

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
- fast_image_resize = "0.1"
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
| image      | 105.194 | 198.359  |  289.810   | 381.861  |
| resize     | 16.186  |  71.964  |  132.501   | 192.672  |
| fir rust   |  0.476  |  55.284  |   84.732   | 117.519  |
| fir sse4.1 |    -    |  11.848  |   17.864   |  25.978  |
| fir avx2   |    -    |  8.753   |   12.569   |  18.112  |

Compiled with `rustflags = ["-C", "target-cpu=native"]`

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      | 91.138  | 182.251  |  279.838   | 380.956  |
| resize     | 15.967  |  62.125  |  114.870   | 168.013  |
| fir rust   |  0.467  |  56.376  |   61.565   |  85.433  |
| fir sse4.1 |    -    |  11.229  |   16.503   |  23.465  |
| fir avx2   |    -    |  8.464   |   11.602   |  17.076  |

### Resize RGBA image 4928x3279 => 852x567

Pipeline: 

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      | 101.499 | 216.029  |  335.151   | 455.582  |
| resize     | 19.671  |  85.199  |  155.205   | 224.660  |
| fir rust   | 13.583  |  67.031  |   94.816   | 124.344  |
| fir sse4.1 | 12.012  |  23.370  |   29.345   |  36.925  |
| fir avx2   |  6.893  |  15.162  |   18.959   |  24.518  |

Compiled with `rustflags = ["-C", "target-cpu=native"]`

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      | 90.750  | 186.545  |  284.383   | 392.403  |
| resize     | 19.918  |  69.129  |  130.433   | 191.987  |
| fir rust   |  9.957  |  68.817  |   76.454   | 103.521  |
| fir sse4.1 |  7.865  |  18.407  |   23.586   |  30.649  |
| fir avx2   |  6.882  |  14.847  |   18.026   |  23.450  |

## Example

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
    let src_buffer = img.to_rgba8();

    // Create immutable view of source image data
    let src_view =
        fr::SrcImageView::from_buffer(width, height, src_buffer.as_raw(), fr::PixelType::U8x4)
            .unwrap();

    // Create wrapper that own data of destination image
    let dst_width = NonZeroU32::new(1024).unwrap();
    let dst_height = NonZeroU32::new(768).unwrap();
    let mut dst_image = fr::ImageData::new(dst_width, dst_height, src_view.pixel_type());

    // Get mutable view of destination image data
    let mut dst_view = dst_image.dst_view();

    // Create Resizer instance and resize source image into buffer of destination image
    let mut resizer = fr::Resizer::new(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3));
    resizer.resize(&src_view, &mut dst_view);

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
