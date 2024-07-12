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
- resize = "0.8.4" (<https://crates.io/crates/resize>, single-threaded mode)
- libvips = "8.12.1" (single-threaded mode)

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
| image      |  32.17  |   -   |  94.60   | 153.14  |  211.14  |
| resize     |  9.18   | 26.68 |  49.76   |  96.06  |  141.84  |
| libvips    |  7.75   | 59.58 |  19.81   |  30.46  |  39.96   |
| fir rust   |  0.29   | 11.99 |  16.56   |  25.93  |  37.85   |
| fir sse4.1 |  0.29   | 4.13  |   5.67   |  9.77   |  15.52   |
| fir avx2   |  0.29   | 3.13  |   3.98   |  6.88   |  13.18   |

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
| resize     |  11.98  | 43.60  |  86.90   | 147.95  |  211.64  |
| libvips    |  10.06  | 122.00 |  188.57  | 336.42  |  499.80  |
| fir rust   |  0.19   | 22.02  |  26.95   |  38.41  |  51.70   |
| fir sse4.1 |  0.19   | 10.32  |  12.59   |  18.10  |  24.95   |
| fir avx2   |  0.19   |  7.75  |   8.88   |  13.78  |  22.12   |

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
| image      |  28.76  |   -   |  60.68   |  89.41  |  117.41  |
| resize     |  6.40   | 11.24 |  20.84   |  42.92  |  68.93   |
| libvips    |  4.66   | 25.06 |   9.67   |  13.27  |  17.99   |
| fir rust   |  0.15   | 4.74  |   6.02   |  8.41   |  12.62   |
| fir sse4.1 |  0.15   | 1.67  |   2.14   |  3.31   |   5.61   |
| fir avx2   |  0.15   | 1.74  |   1.91   |  2.31   |   4.16   |

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
| libvips    |  6.42   | 73.21 |  118.10  | 205.49  |  292.37  |
| fir rust   |  0.18   | 18.67 |  21.14   |  25.86  |  32.98   |
| fir sse4.1 |  0.17   | 6.18  |   7.21   |  9.60   |  13.48   |
| fir avx2   |  0.17   | 4.35  |   4.92   |  6.48   |   9.60   |

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
| image      |  31.17  |   -   |  86.80   | 138.46  |  191.10  |
| resize     |  8.16   | 26.79 |  50.87   |  97.41  |  144.44  |
| libvips    |  16.00  | 63.08 |  54.33   | 102.53  |  125.07  |
| fir rust   |  0.33   | 30.74 |  47.64   |  81.28  |  115.50  |
| fir sse4.1 |  0.33   | 16.24 |  23.65   |  38.31  |  54.69   |
| fir avx2   |  0.33   | 13.88 |  19.29   |  29.86  |  36.75   |

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
| resize     |  13.57  | 44.09  |  84.32   | 145.34  |  207.68  |
| libvips    |  22.77  | 128.72 |  205.37  | 366.05  |  537.03  |
| fir rust   |  0.40   | 63.43  |  84.58   | 127.16  |  171.87  |
| fir sse4.1 |  0.40   | 32.01  |  42.48   |  63.86  |  86.06   |
| fir avx2   |  0.40   | 20.60  |  26.04   |  36.66  |  47.97   |

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
| image      |  29.50  |   -   |  61.96   |  91.03  |  119.53  |
| resize     |  6.40   | 11.10 |  20.93   |  43.22  |  68.96   |
| libvips    |  7.45   | 26.23 |  21.80   |  36.47  |  46.00   |
| fir rust   |  0.17   | 14.84 |  21.62   |  30.25  |  42.89   |
| fir sse4.1 |  0.17   | 5.40  |   7.49   |  12.91  |  18.84   |
| fir avx2   |  0.17   | 5.49  |   6.42   |  8.54   |  13.70   |

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
| libvips    |  12.51  | 79.48 |  133.81  | 232.12  |  326.55  |
| fir rust   |  0.19   | 32.88 |  40.97   |  62.61  |  84.70   |
| fir sse4.1 |  0.19   | 15.01 |  21.19   |  33.27  |  45.87   |
| fir avx2   |  0.19   | 11.84 |  15.10   |  22.03  |  29.10   |

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
| image      |  24.21  |   -   |  48.61   |  79.04  |  104.73  |
| resize     |  5.26   | 8.83  |  13.71   |  30.19  |  45.85   |
| libvips    |  6.21   | 27.94 |  19.81   |  43.10  |  69.32   |
| fir rust   |  0.19   | 8.32  |  12.65   |  26.87  |  41.19   |
| fir sse4.1 |  0.19   | 5.28  |   7.28   |  11.67  |  17.07   |
| fir avx2   |  0.19   | 4.68  |   5.46   |  7.11   |  10.73   |

<!-- bench_compare_l32f end -->

<!-- bench_compare_la32f start -->

### Resize LA32F (luma with alpha channel) image (F32x2) 4928x3279 => 852x567

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
| libvips    |  13.72  | 72.74 |  100.78  | 176.30  |  252.88  |
| fir rust   |  0.35   | 21.99 |  29.09   |  47.76  |  70.52   |
| fir sse4.1 |  0.35   | 16.25 |  20.94   |  30.33  |  40.11   |
| fir avx2   |  0.35   | 15.73 |  17.36   |  22.92  |  27.79   |

<!-- bench_compare_la32f end -->

<!-- bench_compare_rgb32f start -->

### Resize RGB32F image (F32x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB32F image.
- Numbers in the table mean a duration of image resizing in milliseconds.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image      |  26.67  |   -   |  64.09   | 108.10  |  155.20  |
| resize     |  9.30   | 16.45 |  24.68   |  48.28  |  72.52   |
| libvips    |  13.46  | 62.91 |  52.79   | 114.29  |  199.08  |
| fir rust   |  0.86   | 16.49 |  27.10   |  51.23  |  76.20   |
| fir sse4.1 |  0.86   | 12.38 |  19.56   |  32.72  |  47.73   |
| fir avx2   |  0.86   | 11.23 |  14.21   |  20.39  |  29.32   |

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
| libvips    |  24.44  | 113.81 |  138.06  | 249.74  |  379.76  |
| fir rust   |  1.01   | 36.20  |  46.05   |  71.26  |  93.95   |
| fir sse4.1 |  1.01   | 31.81  |  39.68   |  58.07  |  77.59   |
| fir avx2   |  1.01   | 28.64  |  30.94   |  40.70  |  49.71   |

<!-- bench_compare_rgba32f end -->
