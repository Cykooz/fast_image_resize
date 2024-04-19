<!-- introduction start -->

## Benchmarks of fast_image_resize crate for arm64 architecture

Environment:

- CPU: Neoverse-N1 2GHz (Oracle Cloud Compute, VM.Standard.A1.Flex)
- Ubuntu 22.04 (linux 6.5.0)
- Rust 1.77.2
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
| image    |  90.20  |   -    |  166.08  | 303.20  |  430.75  |
| resize   |  15.68  | 54.30  |  101.43  | 199.72  |  303.02  |
| libvips  |  26.78  | 128.18 |  105.36  | 212.25  |  284.64  |
| fir rust |  0.89   | 21.91  |  35.02   |  79.65  |  107.28  |
| fir neon |  0.89   | 20.05  |  30.20   |  49.85  |  73.68   |

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
| resize   |  25.67  | 85.67  |  124.05  | 214.50  |  307.61  |
| libvips  |  35.85  | 235.69 |  358.87  | 707.52  |  951.53  |
| fir rust |  0.95   | 45.27  |  58.06   | 121.95  |  159.06  |
| fir neon |  0.95   | 36.69  |  49.59   |  77.06  |  107.21  |

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
| image    |  82.62  |   -   |  107.17  | 176.69  |  244.89  |
| resize   |  10.55  | 29.08 |  42.17   |  70.09  |  104.17  |
| libvips  |  12.31  | 49.87 |  38.62   |  71.87  |  94.15   |
| fir rust |  0.50   | 8.93  |  12.49   |  18.52  |  27.88   |
| fir neon |  0.50   | 6.18  |   9.78   |  16.88  |  25.27   |

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
| libvips  |  19.64  | 146.86 |  222.28  | 406.17  |  551.26  |
| fir rust |  0.66   | 23.51  |  31.74   |  61.53  |  70.75   |
| fir neon |  0.66   | 19.56  |  25.77   |  38.22  |  54.01   |

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
| image    |  92.23  |   -    |  168.25  | 322.07  |  478.89  |
| resize   |  18.81  | 58.74  |  110.13  | 226.83  |  328.52  |
| libvips  |  27.46  | 140.79 |  115.85  | 240.04  |  313.65  |
| fir rust |  1.38   | 59.23  |  86.66   | 156.94  |  194.54  |
| fir neon |  1.38   | 63.84  |  70.34   |  89.05  |  126.25  |

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
| resize   |  27.79  | 86.11  |  146.72  | 276.74  |  396.76  |
| libvips  |  37.78  | 245.44 |  377.70  | 733.81  |  990.11  |
| fir rust |  1.53   | 76.85  |  109.19  | 185.33  |  247.13  |
| fir neon |  1.53   | 57.80  |  77.53   | 117.35  |  160.90  |

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
| image    |  83.01  |   -   |  114.49  | 188.01  |  252.44  |
| resize   |  11.17  | 28.98 |  46.08   |  79.95  |  103.28  |
| libvips  |  12.27  | 55.24 |  42.38   |  79.91  |  106.25  |
| fir rust |  0.65   | 27.74 |  37.38   |  56.50  |  77.09   |
| fir neon |  0.65   | 13.23 |  17.10   |  27.05  |  38.15   |

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
| libvips  |  20.91  | 150.82 |  239.37  | 430.56  |  585.25  |
| fir rust |  0.95   | 44.09  |  60.50   |  99.71  |  133.58  |
| fir neon |  0.95   | 26.79  |  35.08   |  51.66  |  68.65   |

<!-- bench_compare_la16 end -->
