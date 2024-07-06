<!-- introduction start -->

## Benchmarks of fast_image_resize crate for arm64 architecture

Environment:

- CPU: Neoverse-N1 2GHz (Oracle Cloud Compute, VM.Standard.A1.Flex)
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

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| image    |  88.57  |   -    |  161.64  | 302.78  |  427.58  |
| resize   |  18.23  | 57.41  |  100.21  | 182.06  |  269.21  |
| libvips  |  26.80  | 127.55 |  106.28  | 207.19  |  278.34  |
| fir rust |  0.90   | 21.87  |  35.17   |  83.46  |  112.76  |
| fir neon |  0.90   | 19.73  |  30.06   |  50.35  |  72.19   |

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
| resize   |  25.54  | 88.18  |  123.40  | 200.42  |  293.80  |
| libvips  |  33.41  | 238.14 |  352.40  | 716.10  |  973.96  |
| fir rust |  0.97   | 49.57  |  63.97   | 125.33  |  169.58  |
| fir neon |  0.97   | 37.39  |  50.31   |  78.54  |  107.03  |

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
| image    |  81.61  |   -   |  108.99  | 175.51  |  247.57  |
| resize   |  10.85  | 27.94 |  40.56   |  64.81  |  96.27   |
| libvips  |  12.29  | 50.07 |  38.45   |  70.87  |  94.21   |
| fir rust |  0.47   | 8.81  |  12.35   |  18.06  |  27.73   |
| fir neon |  0.47   | 6.13  |   9.11   |  16.15  |  24.81   |

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
| libvips  |  19.29  | 144.90 |  220.76  | 403.94  |  550.70  |
| fir rust |  0.68   | 37.46  |  46.04   |  61.22  |  71.23   |
| fir neon |  0.68   | 18.38  |  24.45   |  36.89  |  52.69   |

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
| image    |  91.90  |   -    |  170.26  | 331.19  |  470.90  |
| resize   |  19.59  | 56.26  |  96.88   | 179.84  |  268.43  |
| libvips  |  27.43  | 141.41 |  115.52  | 234.66  |  312.04  |
| fir rust |  1.42   | 61.01  |  90.50   | 148.67  |  205.49  |
| fir neon |  1.42   | 63.82  |  71.03   |  93.53  |  132.92  |

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
| resize   |  31.53  | 87.13  |  129.24  | 219.52  |  318.78  |
| libvips  |  37.19  | 245.54 |  368.92  | 742.45  | 1004.81  |
| fir rust |  1.59   | 97.25  |  133.51  | 225.08  |  296.06  |
| fir neon |  1.59   | 56.57  |  78.24   | 119.19  |  162.73  |

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
| image    |  81.22  |   -   |  110.96  | 185.44  |  253.11  |
| resize   |  11.12  | 27.58 |  43.97   |  72.33  |  98.01   |
| libvips  |  12.39  | 55.35 |  41.49   |  80.00  |  104.92  |
| fir rust |  0.63   | 28.46 |  39.16   |  58.51  |  83.71   |
| fir neon |  0.63   | 12.96 |  17.41   |  26.79  |  38.46   |

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
| libvips  |  20.97  | 150.55 |  241.54  | 428.63  |  583.38  |
| fir rust |  1.00   | 61.63  |  80.23   | 115.45  |  153.73  |
| fir neon |  1.00   | 26.84  |  36.52   |  55.71  |  75.56   |

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
| image    |  24.55  |   -   |  49.54   |  78.44  |  104.51  |
| resize   |  5.03   | 8.64  |  13.85   |  30.14  |  45.81   |
| libvips  |  5.96   | 25.74 |  21.52   |  42.24  |  71.71   |
| fir rust |  0.20   | 9.87  |  14.93   |  29.66  |  51.71   |

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

|          | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:-----:|:--------:|:-------:|:--------:|
| libvips  |  11.58  | 70.05 |  101.70  | 176.48  |  252.74  |
| fir rust |  0.38   | 22.75 |  30.35   |  49.41  |  71.81   |

<!-- bench_compare_la32f end -->

<!-- bench_compare_rgb32f start -->

### Resize RGB16F image (F32x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB32F image.
- Numbers in the table mean a duration of image resizing in milliseconds.

|          | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image    |  26.32  |   -   |  62.99   | 104.89  |  147.02  |
| resize   |  8.97   | 16.30 |  24.52   |  48.25  |  72.16   |
| libvips  |  11.75  | 60.36 |  52.63   | 113.20  |  197.67  |
| fir rust |  0.83   | 16.39 |  26.95   |  50.36  |  75.58   |

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
| libvips  |  23.22  | 111.26 |  140.16  | 249.97  |  381.64  |
| fir rust |  1.01   | 35.88  |  45.64   |  70.56  |  93.29   |

<!-- bench_compare_rgba32f end -->
