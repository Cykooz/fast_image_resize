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
- Bilinear
- CatmullRom
- Lanczos3

### Resize 4928x3279 => 852x567

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is time of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  101.98 |  204.79  |    307.0   |  421.58  |
| resize     |  23.139 |  103.84  |   182.36   |  261.64  |
| fir native |  0.486  |  54.479  |   81.625   |  113.37  |
| fir sse4.1 |    -    |   12.08  |   18.733   |  26.689  |
| fir avx2   |    -    |   9.58   |   14.319   |  20.133  |

fir - fast_image_resize


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
