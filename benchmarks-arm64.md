<!-- introduction start -->
## Benchmarks of fast_image_resize crate for arm64 architecture

Environment:

- CPU: Neoverse-N1 2GHz (Oracle Cloud Compute, VM.Standard.A1.Flex)
- Ubuntu 22.04 (linux 6.2.0)
- Rust 1.75.0
- criterion = "0.5.1"
- fast_image_resize = "3.0.0"


Other libraries used to compare of resizing speed:

- image = "0.24.7" (<https://crates.io/crates/image>)
- resize = "0.8.3" (<https://crates.io/crates/resize>)
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
| image    |  84.23  |   -    |  158.27  | 300.25  |  424.18  |
| resize   |    -    | 54.79  |  101.83  | 214.95  |  312.68  |
| libvips  |  26.96  | 127.39 |  108.26  | 211.88  |  285.14  |
| fir rust |  0.90   | 21.34  |  33.56   |  79.22  |  101.92  |
| fir neon |  0.90   | 20.30  |  30.95   |  54.08  |  79.14   |
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
| resize   |    -    | 90.34  |  126.89  | 214.26  |  306.18  |
| libvips  |  36.27  | 236.63 |  362.93  | 706.31  |  959.75  |
| fir rust |  1.00   | 47.82  |  62.33   | 125.25  |  160.17  |
| fir neon |  1.00   | 38.19  |  52.98   |  81.18  |  113.44  |
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
| image    |  80.44  |   -   |  104.41  | 177.41  |  245.26  |
| resize   |    -    | 29.58 |  40.81   |  69.88  |  99.03   |
| libvips  |  12.42  | 50.65 |  39.70   |  71.96  |  96.63   |
| fir rust |  0.49   | 8.89  |  12.50   |  18.65  |  28.38   |
| fir neon |  0.49   | 6.65  |  10.48   |  17.71  |  26.15   |
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
| libvips  |  19.61  | 146.02 |  227.87  | 406.12  |  552.30  |
| fir rust |  0.68   | 21.96  |  30.28   |  49.89  |  57.94   |
| fir neon |  0.68   | 19.20  |  25.86   |  38.59  |  54.96   |
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
| image    |  85.49  |   -    |  158.64  | 325.95  |  476.04  |
| resize   |    -    | 59.51  |  109.85  | 221.47  |  331.32  |
| libvips  |  27.84  | 142.44 |  118.54  | 244.13  |  315.47  |
| fir rust |  1.46   | 54.02  |  76.02   | 133.93  |  165.37  |
| fir neon |  1.46   | 63.23  |  71.72   |  90.48  |  127.29  |
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
| resize   |    -    | 87.97  |  148.55  | 281.23  |  396.41  |
| libvips  |  38.35  | 247.96 |  373.37  | 739.72  | 1010.18  |
| fir rust |  1.58   | 76.35  |  108.01  | 185.41  |  245.74  |
| fir neon |  1.58   | 58.50  |  78.71   | 118.26  |  160.98  |
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
| image    |  80.61  |   -   |  110.48  | 186.24  |  257.99  |
| resize   |    -    | 28.96 |  45.77   |  82.03  |  99.50   |
| libvips  |  12.46  | 54.61 |  43.46   |  82.52  |  106.08  |
| fir rust |  0.66   | 26.66 |  36.91   |  54.87  |  77.55   |
| fir neon |  0.66   | 13.21 |  17.64   |  26.98  |  37.92   |
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
| libvips  |  21.68  | 152.92 |  242.95  | 430.93  |  587.43  |
| fir rust |  1.02   | 44.28  |  60.84   |  97.75  |  131.67  |
| fir neon |  1.02   | 27.06  |  35.12   |  53.16  |  72.03   |
<!-- bench_compare_la16 end -->
