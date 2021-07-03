# fast_image_resize

Rust library for fast image resizing with using of SIMD instructions.

[CHANGELOG](https://github.com/Cykooz/fast_image_resize/blob/master/CHANGELOG.md)

Supported optimisations:
- native Rust-code without forced SIMD
- with using SSE4.1
- with using AVX2

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

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is time of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  105.38 |  204.21  |   301.58   |  399.52  |
| resize     |  16.223 |  72.447  |   132.64   |  193.26  |
| fir native |  0.476  |  57.003  |   87.564   |  120.65  |
| fir sse4.1 |    -    |  12.143  |   18.662   |  26.334  |
| fir avx2   |    -    |   9.346  |   13.342   |  18.934  |

### Resize RGBA image 4928x3279 => 852x567

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is time of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  110.45 |  264.84  |   421.25   |  596.17  |
| resize     |  23.919 |  100.38  |   173.62   |  247.39  |
| fir native |  13.832 |  70.448  |   100.87   |  133.84  |
| fir sse4.1 |  12.03  |  23.721  |   30.266   |  37.874  |
| fir avx2   |  6.949  |  15.873  |   19.956   |  25.527  |


## Examples of code

```rust
use std::num::NonZeroU32;

use fast_image_resize::{
    CropBox, FilterType, ImageData, PixelType, ResizeAlg, Resizer, SrcImageView,
};

fn resize_lanczos3(src_pixels: &[u8], width: NonZeroU32, height: NonZeroU32) -> Vec<u8> {
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
    let src_image = ImageData::new(width, height, src_pixels, PixelType::U8x4).unwrap();
    let dst_width = NonZeroU32::new(1024).unwrap();
    let dst_height = NonZeroU32::new(768).unwrap();
    let mut dst_image = ImageData::new_owned(dst_width, dst_height, src_image.pixel_type());

    let src_view = src_image.src_view();
    let mut dst_view = dst_image.dst_view();
    resizer.resize(&src_view, &mut dst_view);

    dst_image.get_buffer().to_owned()
}

fn crop_and_resize_image(mut src_view: SrcImageView) -> ImageData<Vec<u8>> {
    let mut resizer = Resizer::new(ResizeAlg::Convolution(FilterType::Lanczos3));
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
    resizer.resize(&src_view, &mut dst_view);

    dst_image
}
```
