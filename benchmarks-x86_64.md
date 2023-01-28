## Benchmarks of fast_image_resize crate for x86_64 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.15.0)
- Rust 1.67
- criterion = "0.4"
- fast_image_resize = "2.5.0"

Other Rust libraries used to compare of resizing speed:

- image = "0.24.5" (<https://crates.io/crates/image>)
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
| image      |  18.53  |  80.66   |   137.96   |  196.04  |
| resize     |    -    |  46.50   |   91.78    |  136.27  |
| fir rust   |  0.28   |  38.94   |   78.10    |  111.35  |
| fir sse4.1 |    -    |   9.88   |   14.22    |  20.09   |
| fir avx2   |    -    |   7.79   |    9.90    |  14.47   |
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
| resize     |    -    |  74.49   |   137.62   |  202.75  |
| fir rust   |  0.19   |  37.75   |   54.07    |  76.86   |
| fir sse4.1 |    -    |  13.13   |   17.33    |  22.57   |
| fir avx2   |    -    |   9.55   |   12.07    |  16.47   |
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
| image      |  15.77  |  47.29   |   74.58    |  102.36  |
| resize     |    -    |  17.05   |   35.51    |  60.73   |
| fir rust   |  0.15   |  13.61   |   15.48    |  23.84   |
| fir sse4.1 |    -    |   4.81   |    5.23    |   7.84   |
| fir avx2   |    -    |   6.66   |    4.99    |   8.17   |
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
| fir rust   |  0.17   |  25.78   |   30.41    |  43.01   |
| fir sse4.1 |    -    |  12.63   |   14.26    |  17.86   |
| fir avx2   |    -    |   8.73   |    9.68    |  12.41   |
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
| image      |  18.84  |  74.16   |   124.67   |  176.08  |
| resize     |    -    |  45.79   |   90.08    |  133.74  |
| fir rust   |  0.34   |  43.47   |   79.63    |  114.34  |
| fir sse4.1 |    -    |  24.51   |   39.97    |  56.39   |
| fir avx2   |    -    |  19.89   |   29.93    |  36.10   |
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
| resize     |    -    |  71.61   |   133.83   |  196.60  |
| fir rust   |  0.33   |  80.52   |   118.32   |  159.08  |
| fir sse4.1 |    -    |  43.59   |   65.40    |  87.52   |
| fir avx2   |    -    |  25.80   |   37.04    |  48.59   |
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
| image      |  16.13  |  47.80   |   75.63    |  104.82  |
| resize     |    -    |  15.38   |   31.74    |  57.08   |
| fir rust   |  0.17   |  19.26   |   29.34    |  41.26   |
| fir sse4.1 |    -    |   8.14   |   13.57    |  19.33   |
| fir avx2   |    -    |   6.64   |    8.80    |  13.96   |
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
| fir rust   |  0.19   |  34.52   |   53.05    |  71.72   |
| fir sse4.1 |    -    |  22.05   |   34.18    |  46.79   |
| fir avx2   |    -    |  15.28   |   22.01    |  29.28   |
<!-- bench_compare_la16 end -->