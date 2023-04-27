## Benchmarks of fast_image_resize crate for x86_64 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.19.0)
- Rust 1.69.0
- criterion = "0.4"
- fast_image_resize = "2.7.1"

Other Rust libraries used to compare of resizing speed:

- image = "0.24.6" (<https://crates.io/crates/image>)
- resize = "0.7.4" (<https://crates.io/crates/resize>)

Resize algorithms:

- Nearest
- Convolution with Bilinear filter
- Convolution with CatmullRom filter
- Convolution with Lanczos3 filter

### Resize RGB8 image (U8x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is mean duration of image resizing in milliseconds.

<!-- bench_compare_rgb start -->
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  19.39  |  81.40   |   138.60   |  196.45  |
| resize     |    -    |  46.45   |   91.99    |  136.59  |
| fir rust   |  0.29   |  38.13   |   65.61    |  97.16   |
| fir sse4.1 |    -    |   9.81   |   14.29    |  20.06   |
| fir avx2   |    -    |   7.91   |    9.89    |  14.84   |
<!-- bench_compare_rgb end -->

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

<!-- bench_compare_rgba start -->
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  73.12   |   137.57   |  202.70  |
| fir rust   |  0.19   |  37.29   |   53.75    |  76.40   |
| fir sse4.1 |    -    |  13.13   |   17.45    |  22.67   |
| fir avx2   |    -    |   9.53   |   12.06    |  16.49   |
<!-- bench_compare_rgba end -->

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

<!-- bench_compare_l start -->
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  16.29  |  47.78   |   75.65    |  103.63  |
| resize     |    -    |  17.09   |   35.60    |  60.28   |
| fir rust   |  0.16   |  13.60   |   15.51    |  23.92   |
| fir sse4.1 |    -    |   4.84   |    5.15    |   7.80   |
| fir avx2   |    -    |   6.66   |    4.97    |   8.14   |
<!-- bench_compare_l end -->

### Resize LA8 image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (two bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

<!-- bench_compare_la start -->
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| fir rust   |  0.17   |  24.81   |   30.32    |  42.56   |
| fir sse4.1 |    -    |  12.65   |   14.27    |  17.91   |
| fir avx2   |    -    |   8.73   |    9.59    |  12.41   |
<!-- bench_compare_la end -->

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

<!-- bench_compare_rgb16 start -->
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  19.01  |  73.92   |   133.43   |  183.58  |
| resize     |    -    |  46.18   |   90.68    |  134.20  |
| fir rust   |  0.35   |  44.34   |   79.10    |  114.15  |
| fir sse4.1 |    -    |  24.57   |   39.91    |  56.27   |
| fir avx2   |    -    |  19.87   |   29.82    |  36.09   |
<!-- bench_compare_rgb16 end -->

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

<!-- bench_compare_rgba16 start -->
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  71.67   |   134.02   |  195.58  |
| fir rust   |  0.39   |  81.01   |   119.14   |  159.04  |
| fir sse4.1 |    -    |  43.61   |   65.36    |  87.56   |
| fir avx2   |    -    |  25.64   |   36.42    |  47.76   |
<!-- bench_compare_rgba16 end -->

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

<!-- bench_compare_l16 start -->
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  16.34  |  48.68   |   76.27    |  105.09  |
| resize     |    -    |  15.41   |   31.72    |  57.02   |
| fir rust   |  0.18   |  18.72   |   27.96    |  38.71   |
| fir sse4.1 |    -    |   8.18   |   13.55    |  19.38   |
| fir avx2   |    -    |   6.66   |    8.71    |  13.97   |
<!-- bench_compare_l16 end -->

### Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (four bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

<!-- bench_compare_la16 start -->
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| fir rust   |  0.20   |  33.72   |   53.68    |  72.66   |
| fir sse4.1 |    -    |  21.98   |   34.05    |  46.60   |
| fir avx2   |    -    |  15.26   |   22.01    |  29.30   |
<!-- bench_compare_la16 end -->