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
| image    |  81.35  |   -    |  155.47  | 292.99  |  420.30  |
| resize   |  17.88  | 58.72  |  99.72   | 191.53  |  283.22  |
| libvips  |  26.19  | 127.94 |  108.26  | 211.62  |  281.96  |
| fir rust |  0.90   | 25.61  |  40.74   | 105.03  |  140.08  |
| fir neon |  0.90   | 19.74  |  29.20   |  49.33  |  73.89   |

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
| resize   |  24.78  | 87.91  |  123.06  | 214.90  |  304.80  |
| libvips  |  35.85  | 237.77 |  359.87  | 708.06  |  956.23  |
| fir rust |  0.94   | 49.35  |  66.77   | 154.14  |  202.65  |
| fir neon |  0.94   | 36.60  |  50.20   |  76.66  |  106.89  |

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
| image    |  77.04  |   -   |  106.26  | 173.16  |  243.71  |
| resize   |  11.02  | 29.25 |  41.11   |  67.09  |  96.79   |
| libvips  |  12.02  | 49.78 |  39.45   |  72.24  |  95.72   |
| fir rust |  0.50   | 10.14 |  14.42   |  34.24  |  45.02   |
| fir neon |  0.50   | 6.14  |   9.33   |  16.24  |  24.50   |

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
| libvips  |  19.49  | 147.59 |  223.85  | 401.32  |  549.79  |
| fir rust |  0.67   | 25.55  |  36.12   |  66.30  |  74.14   |
| fir neon |  0.67   | 18.32  |  24.74   |  37.13  |  53.14   |

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
| image    |  82.63  |   -    |  156.54  | 313.14  |  458.26  |
| resize   |  19.42  | 58.83  |  98.96   | 187.79  |  273.43  |
| libvips  |  27.75  | 142.96 |  116.71  | 240.22  |  310.89  |
| fir rust |  1.46   | 56.42  |  78.80   | 138.76  |  180.98  |
| fir neon |  1.46   | 62.65  |  72.34   |  95.01  |  134.46  |

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
| resize   |  28.30  | 92.30  |  133.03  | 231.50  |  331.17  |
| libvips  |  37.22  | 244.27 |  374.70  | 734.02  | 1002.61  |
| fir rust |  1.54   | 102.16 |  136.90  | 226.23  |  287.39  |
| fir neon |  1.54   | 56.83  |  79.00   | 120.50  |  166.11  |

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
| image    |  77.69  |   -   |  107.96  | 183.76  |  257.47  |
| resize   |  11.67  | 28.88 |  47.17   |  76.66  |  103.51  |
| libvips  |  12.48  | 55.66 |  41.58   |  82.70  |  106.61  |
| fir rust |  0.66   | 26.14 |  36.65   |  55.52  |  76.01   |
| fir neon |  0.66   | 13.37 |  17.58   |  27.39  |  39.01   |

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
| libvips  |  20.62  | 152.38 |  238.83  | 433.39  |  584.20  |
| fir rust |  0.95   | 43.78  |  60.65   |  95.25  |  132.69  |
| fir neon |  0.95   | 27.28  |  36.73   |  55.64  |  76.28   |

<!-- bench_compare_la16 end -->
