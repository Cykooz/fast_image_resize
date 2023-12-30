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
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  28.20  |   -   |  82.45   | 134.07  |  192.70  |
| resize     |    -    | 26.83 |  53.56   |  97.73  |  144.63  |
| libvips    |  7.73   | 60.66 |  19.84   |  30.15  |  39.46   |
| fir rust   |  0.28   | 9.78  |  15.46   |  27.36  |  39.57   |
| fir sse4.1 |  0.28   | 3.87  |   5.59   |  9.89   |  15.44   |
| fir avx2   |  0.28   | 2.67  |   3.54   |  6.96   |  13.22   |
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
| resize     |    -    | 42.96  |  85.43   | 147.79  |  211.49  |
| libvips    |  10.06  | 122.80 |  188.97  | 338.18  |  499.99  |
| fir rust   |  0.19   | 20.10  |  27.08   |  41.32  |  56.79   |
| fir sse4.1 |  0.19   | 10.03  |  12.24   |  18.57  |  25.15   |
| fir avx2   |  0.19   |  6.98  |   8.26   |  13.97  |  21.55   |
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
| image      |  25.96  |   -   |  56.78   |  84.17  |  112.12  |
| resize     |    -    | 10.67 |  18.54   |  39.06  |  62.71   |
| libvips    |  4.72   | 24.93 |   9.70   |  13.68  |  18.07   |
| fir rust   |  0.15   | 4.08  |   5.24   |  7.48   |  11.33   |
| fir sse4.1 |  0.15   | 1.86  |   2.30   |  3.58   |   5.88   |
| fir avx2   |  0.15   | 1.66  |   1.86   |  2.24   |   4.21   |
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
| libvips    |  6.48   | 73.12 |  117.76  | 207.96  |  293.16  |
| fir rust   |  0.17   | 11.19 |  12.90   |  17.42  |  23.90   |
| fir sse4.1 |  0.17   | 6.16  |   7.21   |  9.74   |  13.56   |
| fir avx2   |  0.17   | 3.95  |   4.57   |  6.41   |   9.24   |
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
| image      |  28.92  |   -   |  82.94   | 134.72  |  185.59  |
| resize     |    -    | 26.91 |  49.69   |  95.90  |  141.39  |
| libvips    |  16.00  | 63.29 |  54.38   | 102.70  |  126.07  |
| fir rust   |  0.34   | 26.13 |  42.62   |  77.16  |  112.71  |
| fir sse4.1 |  0.34   | 16.06 |  23.04   |  36.76  |  51.99   |
| fir avx2   |  0.34   | 13.99 |  19.70   |  30.89  |  38.32   |
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
| resize     |    -    | 43.62  |  84.16   | 144.69  |  207.01  |
| libvips    |  22.70  | 130.12 |  205.66  | 365.03  |  536.16  |
| fir rust   |  0.38   | 60.71  |  79.18   | 116.62  |  155.54  |
| fir sse4.1 |  0.38   | 32.14  |  42.66   |  64.57  |  86.63   |
| fir avx2   |  0.38   | 20.34  |  25.74   |  36.85  |  48.39   |
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
| image      |  26.00  |   -   |  57.17   |  85.75  |  114.97  |
| resize     |    -    | 9.95  |  16.13   |  33.72  |  58.90   |
| libvips    |  7.87   | 26.35 |  21.70   |  36.52  |  45.96   |
| fir rust   |  0.17   | 14.11 |  20.97   |  29.53  |  40.03   |
| fir sse4.1 |  0.17   | 5.59  |   7.71   |  13.11  |  19.02   |
| fir avx2   |  0.17   | 5.70  |   6.67   |  8.79   |  13.84   |
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
| libvips    |  12.55  | 79.43 |  133.92  | 232.67  |  328.01  |
| fir rust   |  0.19   | 27.70 |  36.68   |  51.82  |  72.60   |
| fir sse4.1 |  0.19   | 15.27 |  21.39   |  33.56  |  46.19   |
| fir avx2   |  0.19   | 11.53 |  14.72   |  21.77  |  28.98   |
<!-- bench_compare_la16 end -->
