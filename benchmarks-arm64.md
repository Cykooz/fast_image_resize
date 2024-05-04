<!-- introduction start -->

## Benchmarks of fast_image_resize crate for arm64 architecture

Environment:

- CPU: Neoverse-N1 2GHz (Oracle Cloud Compute, VM.Standard.A1.Flex)
- Ubuntu 22.04 (linux 6.5.0)
- Rust 1.78
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

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| image    |  90.69  |   -    |  168.18  | 299.23  |  427.40  |
| resize   |  18.16  | 58.87  |  101.28  | 188.75  |  277.33  |
| libvips  |  27.10  | 128.31 |  106.99  | 210.32  |  284.40  |
| fir rust |  0.90   | 29.31  |  44.65   | 107.87  |  142.49  |
| fir neon |  0.90   | 19.68  |  29.98   |  50.44  |  74.62   |

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
| resize   |  24.31  | 85.76  |  124.73  | 213.75  |  308.93  |
| libvips  |  35.14  | 239.46 |  356.80  | 703.89  |  947.88  |
| fir rust |  0.96   | 53.44  |  69.96   | 157.47  |  205.40  |
| fir neon |  0.96   | 37.38  |  51.67   |  79.36  |  108.03  |

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
| image    |  81.40  |   -   |  106.89  | 178.47  |  248.19  |
| resize   |  10.70  | 28.04 |  39.95   |  70.96  |  95.71   |
| libvips  |  12.34  | 50.17 |  39.13   |  70.50  |  96.28   |
| fir rust |  0.47   | 11.48 |  15.68   |  24.28  |  35.24   |
| fir neon |  0.47   | 6.09  |   9.34   |  16.20  |  24.99   |

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

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| libvips  |  19.70  | 146.65 |  228.38  | 402.65  |  555.39  |
| fir rust |  0.66   | 41.60  |  51.73   |  92.90  |  106.33  |
| fir neon |  0.66   | 18.86  |  25.14   |  37.43  |  52.43   |

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
| image    |  94.20  |   -    |  170.28  | 338.10  |  487.16  |
| resize   |  19.54  | 57.93  |  97.88   | 186.57  |  274.03  |
| libvips  |  27.45  | 141.20 |  116.02  | 241.02  |  312.91  |
| fir rust |  1.37   | 61.30  |  91.44   | 149.30  |  188.70  |
| fir neon |  1.37   | 61.06  |  70.69   |  93.07  |  134.12  |

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
| resize   |  28.13  | 89.26  |  130.60  | 226.11  |  321.18  |
| libvips  |  36.84  | 247.88 |  377.52  | 730.92  |  997.26  |
| fir rust |  1.53   | 105.18 |  137.04  | 226.61  |  286.05  |
| fir neon |  1.53   | 55.58  |  78.28   | 119.44  |  164.93  |

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
| image    |  83.21  |   -   |  114.62  | 190.60  |  264.48  |
| resize   |  11.44  | 28.56 |  44.56   |  77.73  |  101.13  |
| libvips  |  12.35  | 55.45 |  42.78   |  80.09  |  104.97  |
| fir rust |  0.66   | 27.61 |  37.81   |  56.81  |  79.79   |
| fir neon |  0.66   | 13.18 |  17.72   |  27.13  |  38.72   |

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

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| libvips  |  21.02  | 153.62 |  246.05  | 435.33  |  585.66  |
| fir rust |  0.93   | 58.74  |  72.66   | 110.12  |  145.87  |
| fir neon |  0.93   | 27.97  |  37.06   |  55.73  |  75.52   |

<!-- bench_compare_la16 end -->
