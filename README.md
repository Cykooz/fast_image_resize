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
- criterion = "0.3.4"

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
- Numbers in table is time of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  105.38 |  204.21  |   301.58   |  399.52  |
| resize     |  16.223 |  72.447  |   132.64   |  193.26  |
| fir rust   |  0.476  |  57.003  |   87.564   |  120.65  |
| fir sse4.1 |    -    |  12.143  |   18.662   |  26.334  |
| fir avx2   |    -    |   9.346  |   13.342   |  18.934  |

Compiled with `rustflags = ["-C", "target-cpu=native"]`

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  91.498 |  183.47  |   282.34   |  386.74  |
| resize     |  15.752 |  63.137  |   116.45   |  169.73  |
| fir rust   |  0.467  |  58.133  |   65.575   |  89.721  |
| fir sse4.1 |    -    |  11.567  |   16.566   |  23.859  |
| fir avx2   |    -    |   8.746  |   11.818   |  17.253  |

### Resize RGBA image 4928x3279 => 852x567

Pipeline: 

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is time of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  110.45 |  264.84  |   421.25   |  596.17  |
| resize     |  23.919 |  100.38  |   173.62   |  247.39  |
| fir rust   |  13.832 |  70.448  |   100.87   |  133.84  |
| fir sse4.1 |  12.03  |  23.721  |   30.266   |  37.874  |
| fir avx2   |  6.949  |  15.873  |   19.956   |  25.527  |

Compiled with `rustflags = ["-C", "target-cpu=native"]`

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  97.782 |  246.27  |   403.20   |  568.91  |
| resize     |  24.282 |  86.244  |   151.41   |  217.74  |
| fir rust   |  9.968  |  67.628  |    74.96   |  99.376  |
| fir sse4.1 |  7.926  |  19.017  |    24.06   |  31.323  |
| fir avx2   |  6.924  |  15.312  |   18.266   |  23.703  |

## Examples of code

```rust
use std::num::NonZeroU32;

use fast_image_resize::{
    CropBox, FilterType, ImageData, PixelType, ResizeAlg, Resizer, SrcImageView,
};

fn resize_lanczos3(src_pixels: &[u8], width: NonZeroU32, height: NonZeroU32) -> Vec<u8> {
    let src_image = ImageData::new(width, height, src_pixels, PixelType::U8x4).unwrap();
    let src_view = src_image.src_view();
    
    let dst_width = NonZeroU32::new(1024).unwrap();
    let dst_height = NonZeroU32::new(768).unwrap();
    let mut dst_image = ImageData::new_owned(dst_width, dst_height, src_image.pixel_type());
    let mut dst_view = dst_image.dst_view();

    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    resizer.resize(&src_view, &mut dst_view);

    dst_image.get_buffer().to_owned()
}

fn crop_and_resize_image(mut src_view: SrcImageView) -> ImageData<Vec<u8>> {
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
