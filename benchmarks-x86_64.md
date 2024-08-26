<!-- introduction start -->

## Benchmarks of fast_image_resize crate for x86_64 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 4000 MHz
- Ubuntu 24.04 (linux 6.8.0)
- Rust 1.81.0
- criterion = "0.5.1"
- fast_image_resize = "4.3.0"

Other libraries used to compare of resizing speed:

- image = "0.25.2" (<https://crates.io/crates/image>)
- resize = "0.8.7" (<https://crates.io/crates/resize>, single-threaded mode)
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
| image      |  34.13  |   -   |  88.09   | 142.99  |  191.80  |
| resize     |  8.87   | 26.97 |  53.08   |  98.26  |  145.74  |
| libvips    |  2.38   | 61.57 |   5.66   |  9.70   |  16.03   |
| fir rust   |  0.28   | 10.93 |  15.35   |  25.77  |  37.09   |
| fir sse4.1 |  0.28   | 3.43  |   5.39   |  9.82   |  15.34   |
| fir avx2   |  0.28   | 2.62  |   3.80   |  6.89   |  13.22   |

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
| resize     |  9.90   | 37.85  |  74.20   | 133.72  |  201.29  |
| libvips    |  4.17   | 169.03 |  141.52  | 232.30  |  330.89  |
| fir rust   |  0.19   | 20.66  |  26.02   |  37.27  |  50.21   |
| fir sse4.1 |  0.19   |  9.59  |  11.99   |  17.79  |  24.83   |
| fir avx2   |  0.19   |  7.21  |   8.61   |  13.22  |  22.41   |

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
| image      |  29.07  |   -   |  60.25   |  89.15  |  117.51  |
| resize     |  6.42   | 11.26 |  20.87   |  42.87  |  69.50   |
| libvips    |  2.57   | 25.05 |   6.82   |  9.85   |  12.68   |
| fir rust   |  0.15   | 4.45  |   5.57   |  9.02   |  12.31   |
| fir sse4.1 |  0.15   | 1.52  |   2.09   |  3.52   |   5.65   |
| fir avx2   |  0.15   | 1.54  |   1.76   |  2.80   |   4.03   |

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
| libvips    |  3.70   | 94.30 |  79.22   | 123.08  |  165.17  |
| fir rust   |  0.17   | 17.42 |  19.66   |  25.76  |  31.87   |
| fir sse4.1 |  0.17   | 5.89  |   7.11   |  9.83   |  13.51   |
| fir avx2   |  0.17   | 4.09  |   4.85   |  6.70   |   9.68   |

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
| image      |  31.57  |   -   |  87.50   | 140.87  |  192.50  |
| resize     |  8.27   | 26.82 |  50.82   |  97.83  |  144.89  |
| libvips    |  14.14  | 95.67 |  67.26   | 130.91  |  175.04  |
| fir rust   |  0.34   | 27.89 |  45.79   |  79.04  |  113.41  |
| fir sse4.1 |  0.34   | 14.54 |  22.50   |  38.58  |  54.63   |
| fir avx2   |  0.34   | 12.26 |  17.80   |  28.55  |  37.15   |

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
| resize     |  11.11  | 39.37  |  71.59   | 135.94  |  204.58  |
| libvips    |  21.16  | 181.24 |  153.70  | 241.95  |  342.56  |
| fir rust   |  0.38   | 60.31  |  79.71   | 122.36  |  166.54  |
| fir sse4.1 |  0.38   | 30.83  |  41.64   |  63.39  |  85.77   |
| fir avx2   |  0.38   | 20.80  |  26.25   |  37.26  |  49.02   |

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
| image      |  28.99  |   -   |  61.22   |  91.67  |  119.54  |
| resize     |  7.02   | 11.23 |  21.15   |  44.75  |  69.40   |
| libvips    |  5.68   | 34.67 |  23.78   |  43.76  |  59.42   |
| fir rust   |  0.17   | 13.53 |  21.22   |  32.94  |  45.88   |
| fir sse4.1 |  0.17   | 4.95  |   7.20   |  12.59  |  18.74   |
| fir avx2   |  0.17   | 4.94  |   6.14   |  9.09   |  13.75   |

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
| libvips    |  11.27  | 104.93 |  87.91   | 132.52  |  175.67  |
| fir rust   |  0.19   | 30.06  |  38.42   |  59.01  |  81.08   |
| fir sse4.1 |  0.19   | 14.45  |  20.42   |  31.81  |  45.02   |
| fir avx2   |  0.19   | 11.27  |  14.54   |  21.66  |  28.87   |

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
| image      |  25.15  |   -   |  53.26   |  84.55  |  112.61  |
| resize     |  5.18   | 8.87  |  13.79   |  30.25  |  46.06   |
| libvips    |  4.65   | 33.97 |  23.90   |  46.23  |  64.98   |
| fir rust   |  0.19   | 7.12  |  11.76   |  26.13  |  39.24   |
| fir sse4.1 |  0.19   | 4.36  |   6.75   |  11.60  |  16.87   |
| fir avx2   |  0.19   | 4.06  |   5.29   |  7.93   |  11.18   |

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
| libvips    |  10.75  | 91.80 |  77.32   | 118.49  |  161.60  |
| fir rust   |  0.38   | 22.66 |  29.18   |  47.22  |  71.11   |
| fir sse4.1 |  0.38   | 17.36 |  22.09   |  30.88  |  40.93   |
| fir avx2   |  0.38   | 16.35 |  18.25   |  24.06  |  28.64   |

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
| image      |  27.62  |   -   |  64.22   | 108.90  |  148.53  |
| resize     |  9.17   | 16.34 |  24.61   |  48.55  |  72.74   |
| libvips    |  10.72  | 92.12 |  69.06   | 136.43  |  189.79  |
| fir rust   |  0.87   | 14.52 |  24.96   |  48.83  |  73.53   |
| fir sse4.1 |  0.87   | 11.17 |  18.18   |  31.78  |  47.45   |
| fir avx2   |  0.87   | 9.32  |  12.96   |  21.67  |  29.82   |

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
| libvips    |  20.19  | 153.58 |  125.81  | 213.36  |  312.24  |
| fir rust   |  1.04   | 35.17  |  44.55   |  68.83  |  92.90   |
| fir sse4.1 |  1.04   | 30.67  |  38.72   |  57.04  |  76.49   |
| fir avx2   |  1.04   | 28.76  |  29.99   |  39.20  |  49.14   |

<!-- bench_compare_rgba32f end -->
