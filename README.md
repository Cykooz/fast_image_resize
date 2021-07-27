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
- fast_image_resize = "0.1"
- glassbench = "0.3.0"

Other Rust libraries used to compare of resizing speed: 
- image = "0.23.14" (https://crates.io/crates/image)
- resize = "0.7.0" (https://crates.io/crates/resize)

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
| image      | 106.841 |  205.303 |   302.171  |  400.717 |
| resize     |  16.077 |  72.027  |   132.17   |  192.05  |
| fir rust   |  0.477  |  57.038  |   87.254   |  120.589 |
| fir sse4.1 |    -    |  12.333  |   18.391   |  25.872  |
| fir avx2   |    -    |   9.369  |   13.371   |  19.002  |

Compiled with `rustflags = ["-C", "target-cpu=native"]`

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  91.607 |   184.2  |   280.471  |  382.259 |
| resize     |  15.884 |  62.297  |   115.057  |  168.398 |
| fir rust   |  0.473  |  56.827  |   62.832   |  84.949  |
| fir sse4.1 |    -    |  11.561  |   16.621   |   23.75  |
| fir avx2   |    -    |   8.747  |   11.873   |   17.31  |

### Resize RGBA image 4928x3279 => 852x567

Pipeline: 

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      | 103.377 |  209.701 |   318.407  |  428.659 |
| resize     |  20.121 |  88.308  |   161.366  |  234.437 |
| fir rust   |  13.742 |  70.499  |   100.809  |  133.927 |
| fir sse4.1 |  12.031 |  23.732  |   30.216   |  37.862  |
| fir avx2   |  6.894  |  15.442  |   18.875   |  24.157  |

Compiled with `rustflags = ["-C", "target-cpu=native"]`

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  91.512 |  186.247 |   289.125  |  399.234 |
| resize     |  20.377 |  74.147  |   139.552  |  205.352 |
| fir rust   |  9.944  |  65.808  |   72.315   |  94.223  |
| fir sse4.1 |  7.838  |   18.97  |   24.053   |  31.164  |
| fir avx2   |  6.866  |  15.158  |   18.259   |  23.687  |

## Examples of code

```rust
use std::num::NonZeroU32;

use fast_image_resize::{
    CropBox, FilterType, ImageData, PixelType, ResizeAlg, Resizer, SrcImageView,
};

fn resize_lanczos3(src_pixels: &[u32], width: NonZeroU32, height: NonZeroU32) -> Vec<u8> {
  // Create immutable view of source image data
  let src_view = SrcImageView::from_pixels(width, height, src_pixels, PixelType::U8x4).unwrap();

  let dst_width = NonZeroU32::new(1024).unwrap();
  let dst_height = NonZeroU32::new(768).unwrap();
  // Create wrapper that own data of destination image
  let mut dst_image = ImageData::new_owned(dst_width, dst_height, src_view.pixel_type());
  // Get mutable view of destination image data
  let mut dst_view = dst_image.dst_view();

  // Create Resizer instance and resize source image into buffer of destination image
  let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
  resizer.resize(&src_view, &mut dst_view);

  // Return destination buffer as Vec<u8>
  dst_image.get_buffer().to_owned()
}

fn crop_and_resize_image(mut src_view: SrcImageView) -> ImageData<Vec<u32>> {
  // Set crop-box for view of source image
  src_view
          .set_crop_box(CropBox {
            left: 10,
            top: 10,
            width: NonZeroU32::new(100).unwrap(),
            height: NonZeroU32::new(200).unwrap(),
          })
          .unwrap();
  let dst_width = NonZeroU32::new(1024).unwrap();
  let dst_height = NonZeroU32::new(768).unwrap();
  let mut dst_image = ImageData::new_owned(dst_width, dst_height, src_view.pixel_type());
  let mut dst_view = dst_image.dst_view();

  let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
  resizer.resize(&src_view, &mut dst_view);

  dst_image
}
```
