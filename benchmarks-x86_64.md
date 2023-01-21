## Benchmarks of fast_image_resize crate for x86_64 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.15.0)
- Rust 1.66.1
- criterion = "0.4"
- fast_image_resize = "2.4.0"

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

[//]: # (bench_compare_rgb start)
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  18.74  |  80.85   |   138.40   |  196.43  |
| resize     |    -    |  48.75   |   97.45    |  145.80  |
| fir rust   |  0.28   |  38.90   |   78.31    |  111.49  |
| fir sse4.1 |    -    |   9.92   |   14.35    |  20.07   |
| fir avx2   |    -    |   7.82   |    9.90    |  14.48   |
[//]: # (bench_compare_rgb end)

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

[//]: # (bench_compare_rgba start)
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  73.08   |   137.79   |  201.39  |
| fir rust   |  0.19   |  34.74   |   47.92    |  68.65   |
| fir sse4.1 |    -    |  13.03   |   17.35    |  22.69   |
| fir avx2   |    -    |   9.43   |   11.88    |  16.38   |
[//]: # (bench_compare_rgba end)

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

[//]: # (bench_compare_l start)
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  16.07  |  47.59   |   74.81    |  102.84  |
| resize     |    -    |  17.09   |   35.49    |  60.93   |
| fir rust   |  0.15   |  13.59   |   15.52    |  23.87   |
| fir sse4.1 |    -    |   5.00   |    5.31    |   8.07   |
| fir avx2   |    -    |   7.11   |    5.19    |   8.58   |
[//]: # (bench_compare_l end)

### Resize LA8 image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (two bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

[//]: # (bench_compare_la start)
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| fir rust   |  0.17   |  24.66   |   28.86    |  39.83   |
| fir sse4.1 |    -    |  12.72   |   14.20    |  17.78   |
| fir avx2   |    -    |   8.74   |    9.76    |  12.42   |
[//]: # (bench_compare_la end)

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

[//]: # (bench_compare_rgb16 start)
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  18.97  |  74.21   |   138.04   |  186.05  |
| resize     |    -    |  45.81   |   89.88    |  132.87  |
| fir rust   |  0.33   |  42.44   |   71.39    |  99.56   |
| fir sse4.1 |    -    |  24.64   |   39.98    |  56.36   |
| fir avx2   |    -    |  20.22   |   30.88    |  37.19   |
[//]: # (bench_compare_rgb16 end)

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

[//]: # (bench_compare_rgba16 start)
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  71.04   |   133.27   |  194.32  |
| fir rust   |  0.36   |  80.55   |   118.26   |  158.83  |
| fir sse4.1 |    -    |  43.41   |   65.24    |  87.44   |
| fir avx2   |    -    |  25.79   |   36.47    |  47.91   |
[//]: # (bench_compare_rgba16 end)

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

[//]: # (bench_compare_l16 start)
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  15.89  |  47.75   |   75.79    |  104.00  |
| resize     |    -    |  15.42   |   31.89    |  57.14   |
| fir rust   |  0.17   |  18.66   |   27.70    |  38.69   |
| fir sse4.1 |    -    |   8.14   |   13.64    |  19.35   |
| fir avx2   |    -    |   6.65   |    8.84    |  13.98   |
[//]: # (bench_compare_l16 end)

### Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (four bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

[//]: # (bench_compare_la16 start)
|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| fir rust   |  0.19   |  33.55   |   52.91    |  72.29   |
| fir sse4.1 |    -    |  21.98   |   34.08    |  46.66   |
| fir avx2   |    -    |  15.28   |   21.92    |  29.29   |
[//]: # (bench_compare_la16 end)