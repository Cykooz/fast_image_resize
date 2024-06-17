<!-- introduction start -->
## Benchmarks of fast_image_resize crate for x86_64 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 4000 MHz
- Ubuntu 22.04 (linux 6.5.0)
- Rust 1.79
- criterion = "0.5.1"
- fast_image_resize = "4.0.0"


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
| image      |  28.28  |   -   |  82.31   | 134.61  |  186.22  |
| resize     |  7.83   | 26.85 |  53.70   |  97.59  |  144.79  |
| libvips    |  7.76   | 59.65 |  19.83   |  30.73  |  39.80   |
| fir rust   |  0.28   | 12.77 |  17.94   |  27.94  |  40.05   |
| fir sse4.1 |  0.28   | 3.99  |   5.66   |  9.86   |  15.42   |
| fir avx2   |  0.28   | 3.06  |   3.89   |  6.85   |  13.25   |

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
| resize     |  11.34  | 42.76  |  85.43   | 147.41  |  211.31  |
| libvips    |  8.97   | 120.82 |  190.82  | 341.07  |  500.87  |
| fir rust   |  0.19   | 21.79  |  28.34   |  42.24  |  56.72   |
| fir sse4.1 |  0.19   | 10.14  |  12.26   |  18.43  |  24.22   |
| fir avx2   |  0.19   |  7.44  |   8.48   |  14.10  |  21.31   |

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
| image      |  25.53  |   -   |  56.37   |  84.03  |  111.89  |
| resize     |  5.60   | 10.85 |  18.72   |  37.97  |  65.65   |
| libvips    |  4.67   | 25.02 |   9.80   |  13.73  |  18.17   |
| fir rust   |  0.15   | 4.94  |   6.37   |  8.97   |  13.36   |
| fir sse4.1 |  0.15   | 1.66  |   2.10   |  3.28   |   5.57   |
| fir avx2   |  0.15   | 1.76  |   1.96   |  2.37   |   4.41   |

<!-- bench_compare_l end -->

<!-- bench_compare_la start -->

### Resize LA8 image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (two bytes per pixel).
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| libvips    |  6.45   | 73.39 |  117.87  | 205.86  |  293.32  |
| fir rust   |  0.17   | 16.83 |  18.97   |  24.14  |  31.45   |
| fir sse4.1 |  0.17   | 6.15  |   7.17   |  9.64   |  13.40   |
| fir avx2   |  0.17   | 4.30  |   4.89   |  6.50   |   9.73   |

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
| image      |  28.09  |   -   |  82.63   | 134.66  |  185.48  |
| resize     |  8.06   | 26.77 |  51.13   |  97.40  |  144.26  |
| libvips    |  16.00  | 63.19 |  54.32   | 103.03  |  125.58  |
| fir rust   |  0.33   | 27.24 |  42.15   |  72.69  |  104.45  |
| fir sse4.1 |  0.33   | 15.98 |  23.43   |  38.31  |  54.65   |
| fir avx2   |  0.33   | 13.90 |  19.49   |  29.92  |  36.67   |

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
| resize     |  12.17  | 42.81  |  83.83   | 144.13  |  206.21  |
| libvips    |  22.58  | 130.14 |  209.14  | 367.87  |  536.49  |
| fir rust   |  0.37   | 59.65  |  78.35   | 116.41  |  155.49  |
| fir sse4.1 |  0.37   | 31.77  |  42.27   |  63.73  |  85.90   |
| fir avx2   |  0.37   | 20.39  |  26.03   |  36.49  |  47.81   |

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
| image      |  25.68  |   -   |  56.85   |  85.61  |  114.66  |
| resize     |  6.30   | 9.95  |  16.04   |  33.38  |  58.32   |
| libvips    |  7.42   | 26.00 |  21.69   |  36.26  |  46.04   |
| fir rust   |  0.17   | 13.56 |  19.27   |  28.71  |  40.87   |
| fir sse4.1 |  0.17   | 5.26  |   7.44   |  12.86  |  18.78   |
| fir avx2   |  0.17   | 5.44  |   6.42   |  8.56   |  13.65   |

<!-- bench_compare_l16 end -->

<!-- bench_compare_la16 start -->

### Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (four bytes per pixel).
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| libvips    |  12.53  | 80.02 |  134.11  | 231.61  |  328.41  |
| fir rust   |  0.19   | 25.16 |  32.76   |  51.72  |  70.89   |
| fir sse4.1 |  0.19   | 14.92 |  21.27   |  33.49  |  45.88   |
| fir avx2   |  0.19   | 11.72 |  15.02   |  21.87  |  29.07   |

<!-- bench_compare_la16 end -->

<!-- bench_compare_la_f32 start -->
### Resize LA-F32 (luma with alpha channel) image (F32x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (two `f32` values per pixel).
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| libvips    |  11.85  | 70.31 |  101.80  | 177.59  |  254.22  |
| fir rust   |  0.38   | 21.26 |  28.82   |  47.50  |  70.34   |
| fir sse4.1 |  0.38   | 16.23 |  20.91   |  30.35  |  40.12   |
| fir avx2   |  0.39   | 15.05 |  17.18   |  22.47  |  27.89   |
<!-- bench_compare_la_f32 end -->
