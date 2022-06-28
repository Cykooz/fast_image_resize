## Benchmarks of fast_image_resize crate

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.15.0)
- Rust 1.61.0
- fast_image_resize = "0.9.3"
- glassbench = "0.3.3"

Other Rust libraries used to compare of resizing speed:

- image = "0.24.2" (<https://crates.io/crates/image>)
- resize = "0.7.3" (<https://crates.io/crates/resize>)

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
| image      |  19.24  |  82.52   |   152.17   |  207.63  |
| resize     |    -    |  52.19   |   103.40   |  154.15  |
| fir rust   |  0.28   |  40.88   |   69.39    |  101.53  |
| fir sse4.1 |  0.28   |  28.21   |   43.03    |  59.46   |
| fir avx2   |  0.28   |   7.33   |    9.47    |  13.59   |

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel. 

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  61.93   |   122.10   |  182.55  |
| fir rust   |  0.18   |  36.57   |   52.28    |  74.14   |
| fir sse4.1 |  0.18   |  13.14   |   17.21    |  22.44   |
| fir avx2   |  0.18   |   9.69   |   11.99    |  16.23   |

### Resize L8 (luma) image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  15.86  |  47.17   |   74.46    |  102.53  |
| resize     |    -    |  17.30   |   35.92    |  61.52   |
| fir rust   |  0.15   |  14.10   |   16.20    |  24.12   |
| fir sse4.1 |  0.15   |  11.93   |   12.13    |  18.20   |
| fir avx2   |  0.15   |   6.30   |    4.71    |   7.62   |

### Resize LA8 (luma with alpha channel) image (U8x2) 4928x3279 => 852x567

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
| fir rust   |  0.17   |  25.73   |   30.75    |  42.34   |
| fir sse4.1 |  0.17   |  12.81   |   14.64    |  18.06   |
| fir avx2   |  0.17   |  11.26   |   12.42    |  15.46   |

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  18.58  |  76.20   |   138.35   |  193.78  |
| resize     |    -    |  54.72   |   106.58   |  158.30  |
| fir rust   |  0.33   |  43.80   |   80.11    |  116.95  |
| fir sse4.1 |  0.33   |  24.40   |   39.44    |  55.86   |
| fir avx2   |  0.33   |  20.51   |   30.34    |  35.88   |

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  63.81   |   127.53   |  191.08  |
| fir rust   |  0.37   |  80.36   |   118.89   |  159.05  |
| fir sse4.1 |  0.37   |  42.70   |   63.96    |  86.08   |
| fir avx2   |  0.37   |  25.40   |   36.62    |  47.99   |

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  16.37  |  47.13   |   74.89    |  104.83  |
| resize     |    -    |  15.35   |   31.91    |  57.04   |
| fir rust   |  0.17   |  19.05   |   28.02    |  37.48   |
| fir sse4.1 |  0.17   |   7.80   |   13.16    |  19.13   |
| fir avx2   |  0.17   |   7.07   |    9.48    |  14.80   |

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
| fir rust   |  0.19   |  33.44   |   53.17    |  72.06   |
| fir sse4.1 |  0.19   |  21.89   |   33.99    |  46.56   |
| fir avx2   |  0.19   |  15.22   |   21.95    |  28.99   |
