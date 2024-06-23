<!-- introduction start -->

## Benchmarks of fast_image_resize crate for x86_64 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 4000 MHz
- Ubuntu 22.04 (linux 6.5.0)
- Rust 1.79
- criterion = "0.5.1"
- fast_image_resize = "4.1.0"

Other libraries used to compare of resizing speed:

- image = "0.25.1" (<https://crates.io/crates/image>)
- resize = "0.8.4" (<https://crates.io/crates/resize>)
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
- Numbers in the table mean a duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  32.06  |   -   |  94.81   | 153.34  |  212.23  |
| resize     |  9.12   | 26.76 |  52.27   |  97.57  |  144.36  |
| libvips    |  7.75   | 59.45 |  19.98   |  30.79  |  39.94   |
| fir rust   |  0.29   | 11.91 |  16.56   |  26.11  |  37.89   |
| fir sse4.1 |  0.29   | 4.09  |   5.68   |  9.79   |  15.45   |
| fir avx2   |  0.29   | 3.10  |   3.98   |  6.82   |  13.30   |

<!-- bench_compare_rgb end -->

<!-- bench_compare_rgba start -->

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize     |  11.45  | 42.73  |  85.20   | 147.41  |  211.63  |
| libvips    |  9.77   | 120.67 |  189.16  | 336.71  |  500.29  |
| fir rust   |  0.19   | 21.84  |  26.36   |  36.95  |  50.38   |
| fir sse4.1 |  0.19   | 10.22  |  12.42   |  17.84  |  24.86   |
| fir avx2   |  0.19   |  7.85  |   8.87   |  13.82  |  22.10   |

<!-- bench_compare_rgba end -->

<!-- bench_compare_l start -->

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in the table mean a duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  28.75  |   -   |  60.74   |  89.30  |  117.34  |
| resize     |  6.83   | 11.03 |  20.67   |  43.74  |  67.86   |
| libvips    |  4.66   | 25.00 |   9.74   |  13.19  |  17.94   |
| fir rust   |  0.15   | 4.54  |   5.69   |  8.05   |  12.22   |
| fir sse4.1 |  0.15   | 1.70  |   2.16   |  3.34   |   5.64   |
| fir avx2   |  0.15   | 1.74  |   1.90   |  2.31   |   4.28   |

<!-- bench_compare_l end -->

<!-- bench_compare_la start -->

### Resize LA8 image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with an alpha channel (two bytes per pixel).
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| libvips    |  6.45   | 72.42 |  117.32  | 205.22  |  293.11  |
| fir rust   |  0.17   | 18.09 |  20.67   |  25.58  |  32.87   |
| fir sse4.1 |  0.17   | 6.18  |   7.20   |  9.61   |  13.48   |
| fir avx2   |  0.17   | 4.34  |   4.91   |  6.53   |   9.59   |

<!-- bench_compare_la end -->

<!-- bench_compare_rgb16 start -->

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in the table mean a duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  31.04  |   -   |  87.07   | 139.57  |  190.73  |
| resize     |  8.09   | 26.30 |  49.97   |  96.79  |  143.58  |
| libvips    |  16.00  | 62.91 |  54.53   | 102.92  |  125.23  |
| fir rust   |  0.35   | 30.77 |  47.74   |  81.36  |  115.72  |
| fir sse4.1 |  0.35   | 16.16 |  23.62   |  38.31  |  54.64   |
| fir avx2   |  0.35   | 13.85 |  19.15   |  29.74  |  36.50   |

<!-- bench_compare_rgb16 end -->

<!-- bench_compare_rgba16 start -->

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize     |  12.27  | 43.48  |  83.80   | 144.81  |  207.30  |
| libvips    |  22.68  | 128.29 |  205.54  | 364.11  |  533.24  |
| fir rust   |  0.39   | 62.43  |  82.36   | 122.70  |  164.59  |
| fir sse4.1 |  0.39   | 31.92  |  42.43   |  63.89  |  85.88   |
| fir avx2   |  0.39   | 20.45  |  26.17   |  36.72  |  48.24   |

<!-- bench_compare_rgba16 end -->

<!-- bench_compare_l16 start -->

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in the table mean a duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  28.43  |   -   |  61.77   |  90.52  |  119.06  |
| resize     |  6.37   | 11.09 |  20.38   |  42.28  |  67.60   |
| libvips    |  7.63   | 26.04 |  21.73   |  36.34  |  45.95   |
| fir rust   |  0.17   | 14.63 |  20.81   |  30.42  |  43.41   |
| fir sse4.1 |  0.17   | 5.31  |   7.48   |  12.85  |  18.79   |
| fir avx2   |  0.17   | 5.55  |   6.45   |  8.50   |  13.61   |

<!-- bench_compare_l16 end -->

<!-- bench_compare_la16 start -->

### Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with an alpha channel (four bytes per pixel).
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| libvips    |  12.15  | 79.49 |  133.83  | 232.22  |  327.14  |
| fir rust   |  0.19   | 31.57 |  40.69   |  62.09  |  84.65   |
| fir sse4.1 |  0.19   | 14.86 |  21.03   |  33.10  |  45.59   |
| fir avx2   |  0.19   | 11.57 |  14.84   |  21.66  |  28.89   |

<!-- bench_compare_la16 end -->

<!-- bench_compare_l32f start -->

### Resize L32F image (F32) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in the table mean a duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  24.45  |   -   |  49.71   |  78.69  |  105.12  |
| resize     |  5.02   | 8.82  |  13.42   |  29.90  |  45.04   |
| libvips    |  6.48   | 25.81 |  21.50   |  42.64  |  70.48   |
| fir rust   |  0.19   | 9.60  |  15.08   |  29.70  |  51.94   |
| fir sse4.1 |  0.19   | 5.05  |   7.28   |  11.54  |  16.87   |
| fir avx2   |  0.19   | 4.64  |   5.52   |  7.12   |  10.63   |

<!-- bench_compare_l32f end -->

<!-- bench_compare_la32f start -->

### Resize LA-F32 (luma with alpha channel) image (F32x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with an alpha channel (two `f32` values per pixel).
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| libvips    |  11.97  | 69.97 |  101.99  | 177.48  |  252.49  |
| fir rust   |  0.39   | 21.50 |  29.51   |  47.46  |  70.36   |
| fir sse4.1 |  0.39   | 16.46 |  21.12   |  30.52  |  40.27   |
| fir avx2   |  0.39   | 15.32 |  17.33   |  22.63  |  28.11   |

<!-- bench_compare_la32f end -->

<!-- bench_compare_rgb32f start -->

### Resize RGB16F image (F32x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB32F image.
- Numbers in the table mean a duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  26.87  |   -   |  64.40   | 106.65  |  147.88  |
| resize     |  8.89   | 14.23 |  23.78   |  47.82  |  70.56   |
| libvips    |  11.71  | 60.78 |  53.26   | 114.11  |  193.66  |
| fir rust   |  0.88   | 16.43 |  26.93   |  50.27  |  74.94   |
| fir sse4.1 |  0.88   | 12.63 |  19.38   |  32.27  |  46.71   |
| fir avx2   |  0.88   | 10.95 |  13.96   |  20.31  |  29.14   |

<!-- bench_compare_rgb32f end -->


<!-- bench_compare_rgba32f start -->

### Resize RGBA32F image (F32x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support multiplying and dividing by alpha channel
  for this pixel format.

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| libvips    |  23.22  | 111.20 |  140.22  | 250.45  |  386.72  |
| fir rust   |  1.06   | 36.13  |  45.93   |  70.53  |  93.30   |
| fir sse4.1 |  1.06   | 32.10  |  40.35   |  58.59  |  77.88   |
| fir avx2   |  1.06   | 29.64  |  31.75   |  41.44  |  51.22   |

<!-- bench_compare_rgba32f end -->
