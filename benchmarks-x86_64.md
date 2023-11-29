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
- Lanczos3 -convolution with minimal kernel size 6x6 px
<!-- introduction end -->

<!-- bench_compare_rgb start -->
### Resize RGB8 image (U8x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  29.36  |   -   |  91.28   | 148.79  |  207.66  |
| resize     |    -    | 26.85 |  53.61   |  97.96  |  144.70  |
| libvips    |  7.82   | 60.15 |  20.00   |  30.62  |  39.94   |
| fir rust   |  0.29   | 24.68 |  40.63   |  73.49  |  108.02  |
| fir sse4.1 |  0.29   | 7.97  |   9.55   |  14.06  |  19.68   |
| fir avx2   |  0.29   | 6.96  |   7.62   |  9.77   |  14.05   |
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
| resize     |    -    | 42.84  |  85.35   | 147.60  |  211.91  |
| libvips    |  9.44   | 122.38 |  190.92  | 339.54  |  502.03  |
| fir rust   |  0.19   | 60.50  |  100.79  | 188.09  |  291.29  |
| fir sse4.1 |  0.19   | 11.41  |  13.19   |  17.30  |  22.50   |
| fir avx2   |  0.19   |  9.22  |  10.19   |  12.24  |  16.34   |
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
| image      |  26.92  |   -   |  58.04   |  85.53  |  114.00  |
| resize     |    -    | 10.95 |  18.66   |  38.15  |  65.44   |
| libvips    |  4.70   | 25.07 |   9.59   |  13.30  |  17.90   |
| fir rust   |  0.16   | 13.61 |  13.48   |  14.79  |  22.38   |
| fir sse4.1 |  0.16   | 5.83  |   5.30   |  6.05   |   8.52   |
| fir avx2   |  0.16   | 6.60  |   6.24   |  4.58   |   7.83   |
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
| libvips    |  6.43   | 72.86 |  117.83  | 205.40  |  294.15  |
| fir rust   |  0.18   | 26.02 |  25.19   |  30.37  |  42.94   |
| fir sse4.1 |  0.18   | 11.90 |  12.52   |  14.58  |  18.16   |
| fir avx2   |  0.18   | 8.09  |   8.61   |  10.27  |  12.51   |
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
| image      |  29.16  |   -   |  83.62   | 139.48  |  186.90  |
| resize     |    -    | 26.96 |  49.81   |  95.97  |  141.46  |
| libvips    |  16.00  | 62.87 |  54.24   | 101.91  |  125.80  |
| fir rust   |  0.36   | 26.93 |  43.67   |  78.12  |  113.94  |
| fir sse4.1 |  0.36   | 16.07 |  22.95   |  36.78  |  52.08   |
| fir avx2   |  0.36   | 14.25 |  19.79   |  30.81  |  38.21   |
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
| resize     |    -    | 43.09  |  84.11   | 144.84  |  207.43  |
| libvips    |  22.91  | 129.09 |  205.81  | 365.72  |  538.03  |
| fir rust   |  0.37   | 62.01  |  80.56   | 118.95  |  158.09  |
| fir sse4.1 |  0.37   | 31.89  |  42.41   |  64.19  |  86.30   |
| fir avx2   |  0.37   | 20.46  |  26.01   |  36.93  |  48.50   |
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
| image      |  26.43  |   -   |  58.51   |  86.90  |  116.18  |
| resize     |    -    | 9.95  |  16.07   |  33.47  |  58.32   |
| libvips    |  7.63   | 26.42 |  21.70   |  37.33  |  46.08   |
| fir rust   |  0.17   | 14.37 |  21.34   |  30.01  |  40.55   |
| fir sse4.1 |  0.17   | 5.61  |   7.75   |  13.11  |  19.06   |
| fir avx2   |  0.17   | 5.68  |   6.46   |  8.81   |  13.84   |
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
| libvips    |  12.52  | 79.90 |  134.06  | 232.13  |  328.53  |
| fir rust   |  0.20   | 27.21 |  36.24   |  58.69  |  76.85   |
| fir sse4.1 |  0.20   | 15.36 |  21.61   |  33.88  |  46.37   |
| fir avx2   |  0.20   | 11.81 |  14.87   |  22.01  |  29.22   |
<!-- bench_compare_la16 end -->
