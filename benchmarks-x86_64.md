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
| image      |  19.37  |  81.71   |   149.63   |  205.67  |
| resize     |    -    |  48.65   |   97.50    |  145.90  |
| fir rust   |  0.28   |  38.21   |   65.73    |  96.99   |
| fir sse4.1 |    -    |   9.78   |   14.26    |  20.00   |
| fir avx2   |    -    |   7.85   |    9.96    |  14.75   |
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
| resize     |    -    |  74.59   |   138.44   |  202.18  |
| fir rust   |  0.19   |  36.23   |   52.20    |  75.37   |
| fir sse4.1 |    -    |  13.31   |   17.32    |  22.66   |
| fir avx2   |    -    |   9.77   |   12.20    |  16.56   |
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
| image      |  16.55  |  48.08   |   75.52    |  103.29  |
| resize     |    -    |  17.02   |   35.08    |  60.26   |
| fir rust   |  0.16   |  13.59   |   15.51    |  23.58   |
| fir sse4.1 |    -    |   4.74   |    5.15    |   7.81   |
| fir avx2   |    -    |   6.64   |    4.95    |   8.04   |
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
| fir rust   |  0.17   |  25.01   |   30.30    |  42.12   |
| fir sse4.1 |    -    |  12.68   |   14.27    |  17.89   |
| fir avx2   |    -    |   8.79   |    9.67    |  12.48   |
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
| image      |  19.55  |  75.48   |   126.27   |  177.42  |
| resize     |    -    |  45.91   |   90.34    |  134.11  |
| fir rust   |  0.34   |  44.01   |   78.28    |  113.66  |
| fir sse4.1 |    -    |  24.31   |   39.35    |  55.89   |
| fir avx2   |    -    |  19.81   |   30.10    |  36.11   |
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
| resize     |    -    |  72.18   |   134.34   |  195.65  |
| fir rust   |  0.39   |  79.69   |   117.99   |  158.31  |
| fir sse4.1 |    -    |  43.46   |   65.22    |  87.31   |
| fir avx2   |    -    |  25.93   |   36.60    |  48.02   |
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
| image      |  17.01  |  48.77   |   76.80    |  105.87  |
| resize     |    -    |  15.43   |   32.01    |  56.97   |
| fir rust   |  0.17   |  18.66   |   27.70    |  38.56   |
| fir sse4.1 |    -    |   8.19   |   13.57    |  19.33   |
| fir avx2   |    -    |   6.69   |    8.85    |  13.76   |
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
| fir rust   |  0.19   |  34.11   |   53.60    |  72.66   |
| fir sse4.1 |    -    |  22.03   |   34.05    |  46.60   |
| fir avx2   |    -    |  15.38   |   22.13    |  29.33   |
<!-- bench_compare_la16 end -->