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
| image      |  28.98  |   -   |  85.03   | 146.36  |  191.18  |
| resize     |    -    | 26.83 |  53.33   |  97.71  |  144.57  |
| libvips    |  7.73   | 59.65 |  19.86   |  30.40  |  39.81   |
| fir rust   |  0.28   | 23.63 |  39.82   |  74.52  |  107.93  |
| fir sse4.1 |  0.28   | 7.89  |   9.46   |  14.01  |  19.64   |
| fir avx2   |  0.28   | 6.67  |   7.44   |  9.66   |  13.92   |
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
| resize     |    -    | 42.84  |  85.25   | 147.44  |  211.69  |
| libvips    |  9.87   | 122.96 |  190.30  | 339.61  |  501.24  |
| fir rust   |  0.19   | 60.54  |  100.92  | 189.20  |  287.01  |
| fir sse4.1 |  0.19   | 11.09  |  12.94   |  17.08  |  22.32   |
| fir avx2   |  0.19   |  8.69  |   9.40   |  11.84  |  15.86   |
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
| image      |  26.15  |   -   |  57.74   |  85.31  |  113.69  |
| resize     |    -    | 10.86 |  18.68   |  37.91  |  65.64   |
| libvips    |  4.66   | 25.00 |   9.58   |  13.10  |  17.94   |
| fir rust   |  0.15   | 12.35 |  13.42   |  15.05  |  22.06   |
| fir sse4.1 |  0.15   | 5.67  |   5.23   |  5.91   |   8.44   |
| fir avx2   |  0.15   | 6.63  |   6.20   |  4.52   |   7.81   |
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
| libvips    |  6.45   | 73.14 |  118.34  | 205.71  |  293.96  |
| fir rust   |  0.17   | 26.02 |  24.81   |  28.73  |  40.06   |
| fir sse4.1 |  0.17   | 11.84 |  12.46   |  14.53  |  17.99   |
| fir avx2   |  0.17   | 8.09  |   8.58   |  9.80   |  12.38   |
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
| image      |  28.82  |   -   |  91.33   | 160.30  |  214.84  |
| resize     |    -    | 27.10 |  49.64   |  95.91  |  141.28  |
| libvips    |  16.00  | 62.78 |  54.24   | 102.54  |  125.42  |
| fir rust   |  0.35   | 25.55 |  42.63   |  76.47  |  108.68  |
| fir sse4.1 |  0.35   | 16.07 |  23.03   |  36.73  |  51.97   |
| fir avx2   |  0.34   | 13.98 |  19.57   |  30.88  |  38.30   |
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
| resize     |    -    | 43.49  |  83.75   | 144.57  |  206.78  |
| libvips    |  22.61  | 129.33 |  205.19  | 364.93  |  535.68  |
| fir rust   |  0.38   | 61.06  |  79.63   | 118.03  |  159.90  |
| fir sse4.1 |  0.38   | 31.79  |  42.33   |  64.26  |  86.26   |
| fir avx2   |  0.38   | 20.43  |  25.94   |  36.59  |  47.92   |
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
| image      |  26.23  |   -   |  58.57   |  86.74  |  115.76  |
| resize     |    -    | 9.94  |  16.08   |  33.55  |  58.73   |
| libvips    |  7.89   | 26.07 |  21.67   |  36.36  |  46.28   |
| fir rust   |  0.17   | 14.22 |  19.82   |  29.13  |  41.53   |
| fir sse4.1 |  0.17   | 5.57  |   7.73   |  13.10  |  19.02   |
| fir avx2   |  0.17   | 5.67  |   6.47   |  8.78   |  13.78   |
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
| libvips    |  12.51  | 79.89 |  133.82  | 231.42  |  327.75  |
| fir rust   |  0.19   | 26.81 |  35.29   |  53.20  |  73.16   |
| fir sse4.1 |  0.19   | 15.29 |  21.43   |  33.53  |  46.13   |
| fir avx2   |  0.19   | 11.59 |  14.76   |  21.73  |  29.05   |
<!-- bench_compare_la16 end -->
