<!-- introduction start -->
## Benchmarks of fast_image_resize crate for x86_64 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 6.2.0)
- Rust 1.75.0
- criterion = "0.5.1"
- fast_image_resize = "3.0.0"


Other libraries used to compare of resizing speed:

- image = "0.24.7" (<https://crates.io/crates/image>)
- resize = "0.8.3" (<https://crates.io/crates/resize>)
- libvips = "8.12.1" (single-threaded mode, cache disabled)


Resize algorithms:

- Nearest
- Box - convolution with minimal kernel size 1x1 px
- Bilinear - convolution with minimal kernel size 2x2 px
- Bicubic (CatmullRom) - convolution with minimal kernel size 4x4 px
- Lanczos3 - convolution with minimal kernel size 6x6 px
<!-- introduction end -->

<!-- bench_compare_rgb start -->
### Resize RGB8 image (U8x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table are mean duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  30.14  |   -   |  90.74   | 149.25  |  208.22  |
| resize     |  7.78   | 26.82 |  53.54   |  97.38  |  144.44  |
| libvips    |  7.78   | 59.56 |  18.69   |  30.36  |  39.69   |
| fir rust   |  0.28   | 9.17  |  14.72   |  26.24  |  38.98   |
| fir sse4.1 |  0.28   | 4.08  |   5.79   |  10.32  |  15.94   |
| fir avx2   |  0.28   | 3.01  |   3.86   |  6.89   |  12.69   |
<!-- bench_compare_rgb end -->

<!-- bench_compare_rgba start -->
### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table are mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize     |  11.30  | 42.85  |  85.27   | 147.28  |  211.34  |
| libvips    |  9.15   | 120.11 |  188.46  | 337.77  |  499.37  |
| fir rust   |  0.20   | 20.69  |  27.88   |  41.83  |  56.67   |
| fir sse4.1 |  0.19   | 10.19  |  12.43   |  17.99  |  24.64   |
| fir avx2   |  0.20   |  7.55  |   8.77   |  13.45  |  20.62   |
<!-- bench_compare_rgba end -->

<!-- bench_compare_l start -->
### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table are mean duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  27.05  |   -   |  58.63   |  86.87  |  115.57  |
| resize     |  6.44   | 11.49 |  21.83   |  43.93  |  71.01   |
| libvips    |  4.69   | 25.00 |   9.69   |  12.95  |  16.46   |
| fir rust   |  0.15   | 3.97  |   4.98   |  7.15   |  11.04   |
| fir sse4.1 |  0.15   | 1.69  |   2.13   |  3.32   |   5.71   |
| fir avx2   |  0.15   | 1.73  |   1.94   |  2.30   |   4.33   |
<!-- bench_compare_l end -->

<!-- bench_compare_la start -->
### Resize LA8 image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (two bytes per pixel).
- Numbers in table are mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| libvips    |  6.54   | 73.82 |  118.25  | 206.24  |  293.99  |
| fir rust   |  0.18   | 10.95 |  12.78   |  17.17  |  23.78   |
| fir sse4.1 |  0.17   | 6.21  |   7.31   |  9.73   |  14.07   |
| fir avx2   |  0.17   | 4.23  |   4.72   |  6.26   |   8.87   |
<!-- bench_compare_la end -->

<!-- bench_compare_rgb16 start -->
### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table are mean duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  30.85  |   -   |  84.62   | 136.64  |  190.22  |
| resize     |  8.09   | 26.36 |  50.24   |  96.74  |  143.86  |
| libvips    |  16.01  | 62.96 |  54.26   | 102.92  |  125.23  |
| fir rust   |  0.34   | 25.73 |  39.52   |  67.32  |  96.71   |
| fir sse4.1 |  0.36   | 16.25 |  23.15   |  36.67  |  51.82   |
| fir avx2   |  0.34   | 14.02 |  19.32   |  30.01  |  37.16   |
<!-- bench_compare_rgb16 end -->

<!-- bench_compare_rgba16 start -->
### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table are mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize     |  11.89  | 43.39  |  83.74   | 144.33  |  206.71  |
| libvips    |  22.60  | 129.23 |  205.57  | 367.46  |  538.41  |
| fir rust   |  0.37   | 58.67  |  77.53   | 114.73  |  153.99  |
| fir sse4.1 |  0.38   | 31.85  |  42.51   |  63.89  |  86.03   |
| fir avx2   |  0.39   | 20.00  |  25.44   |  36.04  |  47.38   |
<!-- bench_compare_rgba16 end -->

<!-- bench_compare_l16 start -->
### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table are mean duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  27.48  |   -   |  59.32   |  88.21  |  117.58  |
| resize     |  6.29   | 11.31 |  20.55   |  44.20  |  68.86   |
| libvips    |  7.89   | 26.35 |  20.67   |  36.39  |  46.13   |
| fir rust   |  0.17   | 13.64 |  19.76   |  28.76  |  39.82   |
| fir sse4.1 |  0.17   | 5.34  |   7.53   |  12.93  |  18.85   |
| fir avx2   |  0.17   | 5.45  |   6.35   |  8.45   |  13.52   |
<!-- bench_compare_l16 end -->

<!-- bench_compare_la16 start -->
### Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (four bytes per pixel).
- Numbers in table are mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| libvips    |  12.50  | 79.75 |  133.81  | 232.26  |  329.04  |
| fir rust   |  0.20   | 25.00 |  32.91   |  51.64  |  71.49   |
| fir sse4.1 |  0.20   | 15.02 |  21.36   |  34.09  |  46.03   |
| fir avx2   |  0.20   | 11.62 |  14.91   |  21.95  |  29.14   |
<!-- bench_compare_la16 end -->
