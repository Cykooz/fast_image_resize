<!-- introduction start -->
## Benchmarks of fast_image_resize crate for Wasm32 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 6.2.0)
- Rust 1.74.0
- criterion = "0.5.1"
- fast_image_resize = "2.7.3"
- wasmtime = "15.0.0"


Other libraries used to compare of resizing speed:

- image = "0.24.7" (<https://crates.io/crates/image>)
- resize = "0.8.2" (<https://crates.io/crates/resize>)


Resize algorithms:

- Nearest
- Box - convolution with minimal kernel size 1x1 px
- Bilinear - convolution with minimal kernel size 2x2 px
- Bicubic (CatmullRom) - convolution with minimal kernel size 4x4 px
- Lanczos3 -convolution with minimal kernel size 6x6 px
<!-- introduction end -->

<!-- bench_compare_rgb start -->
### Resize RGB8 image (U8x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image       |  18.64  |   -   |  102.37  | 187.12  |  271.19  |
| resize      |    -    | 34.45 |  60.83   | 112.43  |  163.63  |
| fir rust    |  0.38   | 35.45 |  52.91   |  89.97  |  127.45  |
| fir simd128 |  0.38   | 14.51 |  15.56   |  20.86  |  29.45   |
<!-- bench_compare_rgb end -->

<!-- bench_compare_rgba start -->
### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|             | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize      |    -    | 40.11  |  72.74   | 138.09  |  202.86  |
| fir rust    |  0.26   | 107.99 |  159.85  | 280.66  |  405.81  |
| fir simd128 |  0.26   | 15.62  |  17.45   |  23.01  |  30.34   |
<!-- bench_compare_rgba end -->

<!-- bench_compare_l start -->
### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image       |  17.15  |   -   |  80.67   | 142.93  |  204.28  |
| resize      |    -    | 18.09 |  29.02   |  52.82  |  76.89   |
| fir rust    |  0.21   | 18.73 |  27.08   |  44.55  |  63.67   |
| fir simd128 |  0.21   | 8.88  |   7.82   |  8.74   |  12.57   |
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

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| fir rust    |  0.19   | 56.80 |  92.73   | 161.42  |  254.34  |
| fir simd128 |  0.19   | 16.29 |  16.33   |  18.41  |  24.30   |
<!-- bench_compare_la end -->

<!-- bench_compare_rgb16 start -->
### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image       |  19.04  |   -   |  96.74   | 174.32  |  252.87  |
| resize      |    -    | 32.56 |  58.90   | 111.59  |  160.42  |
| fir rust    |  0.43   | 41.31 |  62.32   | 105.37  |  148.85  |
| fir simd128 |  0.43   | 33.22 |  52.13   |  89.63  |  128.82  |
<!-- bench_compare_rgb16 end -->

<!-- bench_compare_rgba16 start -->
### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| resize      |    -    | 40.16 |  74.19   | 140.44  |  207.71  |
| fir rust    |  0.37   | 93.26 |  121.24  | 177.01  |  235.33  |
| fir simd128 |  0.37   | 54.35 |  77.83   | 125.41  |  174.12  |
<!-- bench_compare_rgba16 end -->

<!-- bench_compare_l16 start -->
### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image       |  18.53  |   -   |  81.21   | 143.82  |  205.26  |
| resize      |    -    | 19.85 |  31.37   |  54.48  |  78.40   |
| fir rust    |  0.19   | 20.78 |  29.80   |  46.92  |  65.02   |
| fir simd128 |  0.19   | 12.12 |  17.97   |  30.17  |  43.23   |
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

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| fir rust    |  0.27   | 50.38 |  67.82   | 103.23  |  138.77  |
| fir simd128 |  0.27   | 30.33 |  42.07   |  67.68  |  93.42   |
<!-- bench_compare_la16 end -->
