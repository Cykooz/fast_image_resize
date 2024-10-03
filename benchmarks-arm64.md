<!-- introduction start -->

## Benchmarks of fast_image_resize crate for arm64 architecture

Environment:

- CPU: Neoverse-N1 2GHz (Oracle Cloud Compute, VM.Standard.A1.Flex)
- Ubuntu 24.04 (linux 6.8.0)
- Rust 1.81.0
- criterion = "0.5.1"
- fast_image_resize = "5.0.0"

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

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| image    |  87.29  |   -    |  161.50  | 300.01  |  422.03  |
| resize   |  18.40  | 58.24  |  103.05  | 187.60  |  276.63  |
| libvips  |  10.09  | 139.03 |  27.55   |  66.93  |  88.63   |
| fir rust |  0.87   | 20.90  |  34.27   |  84.19  |  109.98  |
| fir neon |  0.87   | 18.68  |  27.25   |  49.81  |  73.35   |

<!-- bench_compare_rgb end -->

<!-- bench_compare_rgba start -->

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize   |  21.06  | 74.75  |  116.24  | 197.20  |  294.64  |
| libvips  |  12.78  | 324.80 |  230.82  | 453.35  |  568.10  |
| fir rust |  0.92   | 47.50  |  59.95   | 129.03  |  166.45  |
| fir neon |  0.92   | 35.02  |  49.27   |  77.35  |  105.99  |

<!-- bench_compare_rgba end -->

<!-- bench_compare_l start -->

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in the table mean a duration of image resizing in milliseconds.

|          | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image    |  78.41  |   -   |  104.80  | 174.66  |  240.78  |
| resize   |  10.63  | 27.45 |  40.33   |  63.03  |  92.52   |
| libvips  |  5.70   | 51.73 |  14.50   |  24.28  |  30.84   |
| fir rust |  0.50   | 8.24  |  11.40   |  19.10  |  26.72   |
| fir neon |  0.50   | 5.49  |   8.92   |  16.09  |  24.49   |

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

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| libvips  |  8.59   | 185.59 |  132.47  | 229.07  |  288.92  |
| fir rust |  0.66   | 35.15  |  44.12   |  58.62  |  70.41   |
| fir neon |  0.66   | 17.61  |  23.13   |  37.96  |  52.88   |

<!-- bench_compare_la end -->

<!-- bench_compare_rgb16 start -->

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in the table mean a duration of image resizing in milliseconds.

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| image    |  89.59  |   -    |  161.49  | 329.96  |  478.48  |
| resize   |  20.19  | 58.37  |  99.98   | 185.89  |  272.58  |
| libvips  |  24.24  | 200.90 |  111.02  | 231.86  |  311.97  |
| fir rust |  1.30   | 54.29  |  84.54   | 143.06  |  204.83  |
| fir neon |  1.30   | 51.49  |  73.47   | 114.72  |  141.26  |

<!-- bench_compare_rgb16 end -->

<!-- bench_compare_rgba16 start -->

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize   |  25.53  | 78.22  |  117.14  | 211.16  |  308.86  |
| libvips  |  32.82  | 324.74 |  231.14  | 460.10  |  580.86  |
| fir rust |  1.57   | 91.95  |  126.75  | 208.18  |  285.42  |
| fir neon |  1.57   | 52.53  |  74.02   | 114.79  |  157.37  |

<!-- bench_compare_rgba16 end -->

<!-- bench_compare_l16 start -->

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in the table mean a duration of image resizing in milliseconds.

|          | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image    |  80.33  |   -   |  109.18  | 182.67  |  252.40  |
| resize   |  11.16  | 27.22 |  43.39   |  71.07  |  96.37   |
| libvips  |  9.20   | 69.16 |  39.31   |  77.20  |  100.41  |
| fir rust |  0.67   | 24.39 |  36.16   |  59.35  |  84.09   |
| fir neon |  0.67   | 11.97 |  16.87   |  26.77  |  38.06   |

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

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| libvips  |  17.31  | 200.28 |  144.61  | 241.53  |  298.13  |
| fir rust |  0.97   | 55.28  |  74.08   | 114.13  |  153.81  |
| fir neon |  0.97   | 24.97  |  34.56   |  53.88  |  75.30   |

<!-- bench_compare_la16 end -->

<!-- bench_compare_l32f start -->

### Resize L32F image (F32) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in the table mean a duration of image resizing in milliseconds.

|          | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image    |  39.38  |   -   |  94.31   | 185.23  |  241.78  |
| resize   |  11.82  | 22.48 |  31.48   |  54.70  |  81.74   |
| libvips  |  8.05   | 66.37 |  39.86   |  92.17  |  120.92  |
| fir rust |  0.96   | 18.62 |  30.71   |  53.79  |  77.15   |

<!-- bench_compare_l32f end -->

Note:
The `resize` crate uses `f32` for intermediate calculations.
The `fast_image_resize` uses `f64`. This is a reason why `fast_image_resize`
is slower or equal in cases with `f32`-based pixels.

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

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| libvips  |  16.45  | 182.62 |  127.58  | 220.52  |  278.16  |
| fir rust |  1.56   | 41.75  |  66.81   | 123.14  |  168.42  |

<!-- bench_compare_la32f end -->

<!-- bench_compare_rgb32f start -->

### Resize RGB32F image (F32x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB32F image.
- Numbers in the table mean a duration of image resizing in milliseconds.

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| image    |  50.67  |   -    |  129.41  | 322.93  |  441.29  |
| resize   |  21.65  | 36.44  |  67.62   | 135.27  |  190.02  |
| libvips  |  19.82  | 200.45 |  114.77  | 278.40  |  355.69  |
| fir rust |  2.24   | 38.69  |  73.20   | 152.46  |  225.81  |

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

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| libvips  |  30.58  | 318.17 |  223.24  | 410.56  |  530.71  |
| fir rust |  3.12   | 68.42  |  112.09  | 213.48  |  316.09  |

<!-- bench_compare_rgba32f end -->
