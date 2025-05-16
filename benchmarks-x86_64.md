<!-- introduction start -->

## Benchmarks of fast_image_resize crate for x86_64 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 4000 MHz
- Ubuntu 24.04 (linux 6.11.0)
- Rust 1.87.0
- criterion = "0.5.1"
- fast_image_resize = "5.1.4"

Other libraries used to compare of resizing speed:

- image = "0.25.6" (<https://crates.io/crates/image>)
- resize = "0.8.8" (<https://crates.io/crates/resize>, single-threaded mode)
- libvips = "8.15.1" (single-threaded mode)

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
| image      |  29.28  |   -   |  83.28   | 136.97  |  189.93  |
| resize     |  7.42   | 26.82 |  49.29   |  93.22  |  140.26  |
| libvips    |  2.42   | 61.73 |   5.66   |  9.81   |  15.78   |
| fir rust   |  0.28   | 10.87 |  16.12   |  26.63  |  38.08   |
| fir sse4.1 |  0.28   | 3.37  |   5.34   |  9.89   |  15.30   |
| fir avx2   |  0.28   | 2.52  |   3.67   |  6.80   |  13.21   |

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
| resize     |  9.59   | 34.02  |  64.61   | 126.43  |  187.18  |
| libvips    |  4.19   | 169.02 |  142.22  | 228.64  |  330.24  |
| fir rust   |  0.19   | 20.30  |  25.25   |  36.57  |  49.69   |
| fir sse4.1 |  0.19   |  9.51  |  11.90   |  17.78  |  24.49   |
| fir avx2   |  0.19   |  7.11  |   8.39   |  13.68  |  21.72   |

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
| image      |  26.90  |   -   |  56.49   |  85.11  |  112.72  |
| resize     |  6.57   | 11.06 |  18.83   |  38.44  |  63.98   |
| libvips    |  2.62   | 24.92 |   6.81   |  9.84   |  12.73   |
| fir rust   |  0.16   | 4.42  |   5.45   |  8.69   |  12.04   |
| fir sse4.1 |  0.16   | 1.45  |   2.02   |  3.37   |   5.44   |
| fir avx2   |  0.16   | 1.51  |   1.73   |  2.74   |   4.11   |

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
| libvips    |  3.66   | 94.55 |  79.09   | 122.64  |  165.17  |
| fir rust   |  0.17   | 14.54 |  16.90   |  23.02  |  29.07   |
| fir sse4.1 |  0.17   | 5.82  |   7.03   |  9.72   |  13.47   |
| fir avx2   |  0.17   | 4.05  |   4.78   |  6.49   |   8.91   |

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
| image      |  29.20  |   -   |  82.74   | 136.66  |  191.22  |
| resize     |  8.31   | 24.85 |  48.08   |  92.32  |  138.14  |
| libvips    |  14.15  | 95.89 |  67.43   | 131.07  |  175.11  |
| fir rust   |  0.35   | 26.30 |  45.04   |  78.18  |  113.16  |
| fir sse4.1 |  0.35   | 14.65 |  22.51   |  38.61  |  55.30   |
| fir avx2   |  0.35   | 12.59 |  17.97   |  28.09  |  36.83   |

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
| resize     |  10.41  | 34.13  |  63.85   | 126.67  |  190.86  |
| libvips    |  21.24  | 181.32 |  152.83  | 241.51  |  344.01  |
| fir rust   |  0.39   | 62.48  |  84.00   | 127.42  |  174.28  |
| fir sse4.1 |  0.39   | 31.00  |  41.70   |  63.55  |  85.98   |
| fir avx2   |  0.39   | 21.07  |  26.54   |  37.35  |  48.88   |

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
| image      |  27.15  |   -   |  57.20   |  86.11  |  114.59  |
| resize     |  6.94   | 10.38 |  16.53   |  34.22  |  59.67   |
| libvips    |  5.74   | 34.76 |  23.81   |  43.92  |  59.54   |
| fir rust   |  0.17   | 13.12 |  20.60   |  32.93  |  45.22   |
| fir sse4.1 |  0.17   | 5.01  |   7.17   |  12.71  |  18.59   |
| fir avx2   |  0.17   | 5.02  |   6.15   |  9.08   |  13.87   |

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

|            | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:------:|:--------:|:-------:|:--------:|
| libvips    |  11.30  | 104.96 |  87.94   | 133.05  |  177.05  |
| fir rust   |  0.19   | 26.01  |  34.51   |  57.11  |  79.20   |
| fir sse4.1 |  0.19   | 14.57  |  20.95   |  32.23  |  45.14   |
| fir avx2   |  0.19   | 11.33  |  14.61   |  21.68  |  29.04   |

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
| image      |  2.25   |   -   |  17.43   |  34.22  |  52.11   |
| resize     |  5.37   | 8.92  |  13.48   |  30.89  |  45.56   |
| libvips    |  4.61   | 34.03 |  23.89   |  45.80  |  64.98   |
| fir rust   |  0.19   | 7.11  |  11.74   |  25.79  |  39.27   |
| fir sse4.1 |  0.19   | 4.33  |   6.75   |  11.59  |  16.93   |
| fir avx2   |  0.19   | 3.87  |   5.17   |  7.81   |  11.16   |

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
| libvips    |  10.74  | 91.69 |  77.46   | 119.06  |  161.51  |
| fir rust   |  0.40   | 20.41 |  27.92   |  46.32  |  68.22   |
| fir sse4.1 |  0.40   | 16.80 |  21.28   |  30.42  |  40.64   |
| fir avx2   |  0.40   | 15.46 |  17.60   |  23.08  |  28.89   |

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
| image      |  2.94   |   -   |  26.03   |  50.30  |  75.89   |
| resize     |  8.77   | 14.26 |  24.16   |  48.92  |  71.69   |
| libvips    |  10.81  | 91.85 |  69.09   | 136.77  |  190.14  |
| fir rust   |  0.87   | 13.98 |  24.68   |  48.56  |  73.59   |
| fir sse4.1 |  0.87   | 11.18 |  18.11   |  31.87  |  47.17   |
| fir avx2   |  0.87   | 9.24  |  13.08   |  21.25  |  29.01   |

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
| libvips    |  20.21  | 153.92 |  125.00  | 214.27  |  313.49  |
| fir rust   |  1.03   | 35.41  |  44.55   |  68.91  |  92.62   |
| fir sse4.1 |  1.03   | 30.65  |  38.50   |  57.21  |  76.65   |
| fir avx2   |  1.03   | 28.66  |  30.10   |  38.79  |  48.73   |

<!-- bench_compare_rgba32f end -->
