## Benchmarks of fast_image_resize crate

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.15.0)
- Rust 1.65
- fast_image_resize = "2.3.0"
- glassbench = "0.3.3"

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

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  21.19  |  83.67   |   142.85   |  200.63  |
| resize     |    -    |  48.86   |   97.63    |  145.82  |
| fir rust   |  0.28   |  38.49   |   75.80    |  106.99  |
| fir sse4.1 |  0.28   |   9.45   |   13.59    |  19.39   |
| fir avx2   |  0.28   |   7.53   |    9.61    |  13.78   |

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel. 

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  65.57   |   129.76   |  193.57  |
| fir rust   |  0.18   |  37.33   |   53.84    |  76.67   |
| fir sse4.1 |  0.18   |  13.01   |   17.21    |  22.41   |
| fir avx2   |  0.18   |   9.58   |   12.03    |  16.28   |

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  17.79  |  49.78   |   78.10    |  106.32  |
| resize     |    -    |  17.11   |   35.43    |  60.81   |
| fir rust   |  0.15   |  13.48   |   15.50    |  23.58   |
| fir sse4.1 |  0.15   |   4.53   |    5.08    |   7.61   |
| fir avx2   |  0.15   |   6.00   |    4.75    |   7.46   |

### Resize LA8 image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (two bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| fir rust   |  0.17   |  25.72   |   30.47    |  42.66   |
| fir sse4.1 |  0.17   |  12.46   |   14.29    |  17.68   |
| fir avx2   |  0.17   |   8.40   |    9.54    |  12.11   |

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  21.41  |  77.21   |   129.91   |  180.91  |
| resize     |    -    |  45.29   |   89.53    |  133.11  |
| fir rust   |  0.33   |  44.13   |   79.81    |  114.73  |
| fir sse4.1 |  0.33   |  23.98   |   39.08    |  55.20   |
| fir avx2   |  0.33   |  19.41   |   29.18    |  36.06   |

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  63.77   |   125.95   |  187.88  |
| fir rust   |  0.37   |  82.06   |   120.76   |  160.84  |
| fir sse4.1 |  0.37   |  42.40   |   63.79    |  85.78   |
| fir avx2   |  0.37   |  23.90   |   35.07    |  46.52   |

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  17.75  |  50.70   |   79.91    |  110.49  |
| resize     |    -    |  15.29   |   31.49    |  56.96   |
| fir rust   |  0.17   |  19.19   |   27.03    |  38.05   |
| fir sse4.1 |  0.17   |   7.77   |   13.31    |  19.07   |
| fir avx2   |  0.17   |   6.40   |    8.64    |  13.76   |

### Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (four bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| fir rust   |  0.18   |  33.83   |   53.37    |  72.47   |
| fir sse4.1 |  0.18   |  21.96   |   34.14    |  46.69   |
| fir avx2   |  0.18   |  15.22   |   21.90    |  29.04   |
