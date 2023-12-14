<!-- introduction start -->
## Benchmarks of fast_image_resize crate for x86_64 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 6.2.0)
- Rust 1.74.0
- criterion = "0.5.1"
- fast_image_resize = "2.7.3"


Other libraries used to compare of resizing speed:

- image = "0.24.7" (<https://crates.io/crates/image>)
- resize = "0.8.2" (<https://crates.io/crates/resize>)
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
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  31.83  |   -   |  90.73   | 157.56  |  210.36  |
| resize     |    -    | 26.82 |  54.07   |  97.90  |  144.95  |
| libvips    |  7.65   | 59.54 |  19.80   |  30.02  |  39.42   |
| fir rust   |  0.28   | 9.79  |  15.47   |  27.34  |  39.58   |
| fir sse4.1 |  0.28   | 3.96  |   5.68   |  10.12  |  15.86   |
| fir avx2   |  0.28   | 2.73  |   3.59   |  6.82   |  13.16   |
<!-- bench_compare_rgb end -->

<!-- bench_compare_rgba start -->
### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize     |    -    | 43.26  |  85.63   | 147.73  |  211.88  |
| libvips    |  10.14  | 121.46 |  190.25  | 337.97  |  500.06  |
| fir rust   |  0.19   | 20.22  |  27.17   |  41.57  |  56.84   |
| fir sse4.1 |  0.19   | 10.04  |  12.29   |  18.50  |  25.14   |
| fir avx2   |  0.19   |  7.12  |   8.20   |  13.86  |  22.36   |
<!-- bench_compare_rgba end -->

<!-- bench_compare_l start -->
### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  25.71  |   -   |  57.23   |  86.20  |  113.09  |
| resize     |    -    | 10.85 |  18.90   |  38.28  |  65.62   |
| libvips    |  4.69   | 25.00 |   9.63   |  13.40  |  17.98   |
| fir rust   |  0.16   | 4.29  |   5.27   |  7.50   |  11.41   |
| fir sse4.1 |  0.16   | 1.88  |   2.32   |  3.61   |   5.95   |
| fir avx2   |  0.16   | 1.70  |   1.92   |  2.35   |   4.47   |
<!-- bench_compare_l end -->

<!-- bench_compare_la start -->
### Resize LA8 image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (two bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| libvips    |  6.44   | 72.80 |  117.74  | 205.61  |  293.02  |
| fir rust   |  0.17   | 11.10 |  13.06   |  17.49  |  24.11   |
| fir sse4.1 |  0.17   | 6.24  |   7.32   |  9.88   |  13.85   |
| fir avx2   |  0.17   | 4.13  |   4.74   |  6.44   |   9.22   |
<!-- bench_compare_la end -->

<!-- bench_compare_rgb16 start -->
### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  29.30  |   -   |  83.82   | 135.59  |  187.38  |
| resize     |    -    | 26.35 |  50.23   |  96.71  |  143.53  |
| libvips    |  16.00  | 63.15 |  54.30   | 101.65  |  125.21  |
| fir rust   |  0.35   | 24.72 |  41.75   |  75.50  |  107.55  |
| fir sse4.1 |  0.35   | 16.26 |  23.22   |  36.94  |  52.17   |
| fir avx2   |  0.35   | 14.07 |  19.69   |  30.93  |  38.58   |
<!-- bench_compare_rgb16 end -->

<!-- bench_compare_rgba16 start -->
### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize     |    -    | 43.62  |  84.14   | 144.98  |  207.50  |
| libvips    |  22.68  | 128.33 |  205.29  | 364.70  |  535.40  |
| fir rust   |  0.38   | 60.18  |  78.69   | 116.18  |  155.10  |
| fir sse4.1 |  0.38   | 31.96  |  42.41   |  64.24  |  86.47   |
| fir avx2   |  0.38   | 20.40  |  26.04   |  36.97  |  48.46   |
<!-- bench_compare_rgba16 end -->

<!-- bench_compare_l16 start -->
### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  26.44  |   -   |  59.57   |  88.45  |  118.20  |
| resize     |    -    | 10.07 |  16.27   |  33.82  |  59.02   |
| libvips    |  7.71   | 26.87 |  21.95   |  37.06  |  46.97   |
| fir rust   |  0.18   | 14.41 |  21.24   |  30.19  |  40.49   |
| fir sse4.1 |  0.18   | 5.72  |   7.92   |  13.43  |  19.29   |
| fir avx2   |  0.18   | 5.77  |   6.68   |  8.97   |  13.78   |
<!-- bench_compare_l16 end -->

<!-- bench_compare_la16 start -->
### Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (four bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| libvips    |  12.54  | 79.43 |  133.76  | 231.51  |  328.53  |
| fir rust   |  0.19   | 26.33 |  34.62   |  52.63  |  72.48   |
| fir sse4.1 |  0.19   | 15.64 |  21.74   |  33.77  |  46.25   |
| fir avx2   |  0.19   | 11.71 |  14.89   |  21.91  |  29.22   |
<!-- bench_compare_la16 end -->
