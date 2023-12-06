<!-- introduction start -->
## Benchmarks of fast_image_resize crate for arm64 architecture

Environment:

- CPU: Neoverse-N1 2GHz (Oracle Cloud Compute, VM.Standard.A1.Flex)
- Ubuntu 22.04 (linux 6.2.0)
- Rust 1.74.0
- criterion = "0.5.1"
- fast_image_resize = "2.7.3"


Other libraries used to compare of resizing speed:

- image = "0.24.7" (<https://crates.io/crates/image>)
- resize = "0.8.2" (<https://crates.io/crates/resize>)
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
- Numbers in table is mean duration of image resizing in milliseconds.

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| image    |  85.70  |   -    |  165.53  | 306.98  |  434.12  |
| resize   |    -    | 54.54  |  104.14  | 216.28  |  315.33  |
| libvips  |  27.32  | 128.45 |  108.65  | 212.03  |  286.66  |
| fir rust |  0.89   | 44.47  |  64.55   |  86.93  |  106.47  |
| fir neon |  0.89   | 36.01  |  43.77   |  59.54  |  82.73   |
<!-- bench_compare_rgb end -->

<!-- bench_compare_rgba start -->
### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize   |    -    | 88.20  |  124.25  | 212.49  |  310.58  |
| libvips  |  36.34  | 241.01 |  361.89  | 708.92  |  955.79  |
| fir rust |  1.00   | 84.57  |  126.75  | 225.98  |  311.13  |
| fir neon |  1.00   | 38.06  |  46.57   |  63.90  |  88.55   |
<!-- bench_compare_rgba end -->

<!-- bench_compare_l start -->
### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|          | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image    |  75.45  |   -   |  106.36  | 178.09  |  241.14  |
| resize   |    -    | 29.27 |  42.14   |  70.37  |  103.28  |
| libvips  |  12.05  | 50.26 |  38.89   |  72.16  |  96.37   |
| fir rust |  0.50   | 25.21 |  30.45   |  30.65  |  42.67   |
| fir neon |  0.50   | 11.97 |  14.26   |  20.71  |  27.35   |
<!-- bench_compare_l end -->

<!-- bench_compare_la start -->
### Resize LA8 image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (two bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| libvips  |  19.81  | 146.93 |  225.07  | 406.79  |  553.61  |
| fir rust |  0.63   | 43.35  |  60.97   |  69.54  |  67.24   |
| fir neon |  0.63   | 34.19  |  36.30   |  40.20  |  55.14   |
<!-- bench_compare_la end -->

<!-- bench_compare_rgb16 start -->
### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| image    |  87.54  |   -    |  166.89  | 333.04  |  477.76  |
| resize   |    -    | 59.54  |  110.42  | 225.94  |  338.53  |
| libvips  |  27.95  | 141.49 |  115.13  | 245.45  |  319.30  |
| fir rust |  1.44   | 56.06  |  76.68   | 135.12  |  186.88  |
| fir neon |  1.44   | 63.61  |  70.51   |  91.09  |  128.26  |
<!-- bench_compare_rgb16 end -->

<!-- bench_compare_rgba16 start -->
### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize   |    -    | 86.98  |  147.89  | 278.84  |  396.78  |
| libvips  |  37.75  | 249.06 |  378.55  | 739.45  |  994.75  |
| fir rust |  1.53   | 78.32  |  107.65  | 181.44  |  240.45  |
| fir neon |  1.53   | 59.54  |  79.88   | 120.79  |  163.88  |
<!-- bench_compare_rgba16 end -->

<!-- bench_compare_l16 start -->
### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|          | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image    |  76.84  |   -   |  110.23  | 187.22  |  252.48  |
| resize   |    -    | 28.62 |  47.00   |  79.21  |  106.25  |
| libvips  |  12.70  | 54.92 |  42.32   |  84.12  |  108.10  |
| fir rust |  0.62   | 27.39 |  36.86   |  54.36  |  75.75   |
| fir neon |  0.62   | 13.00 |  18.26   |  27.83  |  38.69   |
<!-- bench_compare_l16 end -->

<!-- bench_compare_la16 start -->
### Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (four bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|          | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:------:|:--------:|:-------:|:--------:|
| libvips  |  21.44  | 151.27 |  246.13  | 439.24  |  591.20  |
| fir rust |  0.95   | 45.45  |  60.95   | 104.35  |  143.89  |
| fir neon |  0.95   | 27.29  |  36.32   |  53.71  |  70.16   |
<!-- bench_compare_la16 end -->
