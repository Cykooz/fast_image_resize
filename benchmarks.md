## Benchmarks of fast_image_resize crate

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.15.0)
- Rust 1.64
- fast_image_resize = "2.0.0"
- glassbench = "0.3.3"

Other Rust libraries used to compare of resizing speed:

- image = "0.24.4" (<https://crates.io/crates/image>)
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
| image      |  19.67  |  82.74   |   142.14   |  201.32  |
| resize     |    -    |  51.53   |   102.81   |  153.19  |
| fir rust   |  0.27   |  44.33   |   81.01    |  119.84  |
| fir sse4.1 |  0.27   |   9.31   |   13.60    |  19.27   |
| fir avx2   |  0.27   |   7.38   |    9.75    |  13.88   |

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel. 

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  61.91   |   122.16   |  182.69  |
| fir rust   |  0.18   |  35.78   |   49.45    |  70.58   |
| fir sse4.1 |  0.18   |  13.47   |   17.38    |  22.56   |
| fir avx2   |  0.18   |   9.65   |   12.16    |  16.45   |

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  16.00  |  47.58   |   75.45    |  103.37  |
| resize     |    -    |  17.29   |   35.98    |  61.51   |
| fir rust   |  0.15   |  14.82   |   16.77    |  25.07   |
| fir sse4.1 |  0.15   |  12.51   |   12.39    |  18.69   |
| fir avx2   |  0.15   |   6.31   |    4.72    |   7.69   |

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
| fir rust   |  0.17   |  25.13   |   30.05    |  40.71   |
| fir sse4.1 |  0.17   |  12.84   |   14.64    |  18.08   |
| fir avx2   |  0.17   |  11.28   |   12.47    |  15.42   |

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  18.66  |  76.14   |   138.29   |  191.26  |
| resize     |    -    |  55.10   |   107.11   |  158.91  |
| fir rust   |  0.33   |  42.60   |   71.05    |  100.55  |
| fir sse4.1 |  0.33   |  24.46   |   39.47    |  55.92   |
| fir avx2   |  0.33   |  20.81   |   31.08    |  36.97   |

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  64.18   |   127.54   |  191.43  |
| fir rust   |  0.36   |  85.44   |   124.23   |  163.95  |
| fir sse4.1 |  0.36   |  42.26   |   63.52    |  85.61   |
| fir avx2   |  0.36   |  23.44   |   34.16    |  45.49   |

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  15.99  |  47.75   |   75.47    |  104.19  |
| resize     |    -    |  15.52   |   32.16    |  57.26   |
| fir rust   |  0.17   |  18.53   |   27.73    |  38.70   |
| fir sse4.1 |  0.17   |   7.85   |   13.25    |  19.24   |
| fir avx2   |  0.17   |   7.06   |    9.50    |  14.80   |

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
| fir rust   |  0.18   |  33.70   |   52.60    |  71.68   |
| fir sse4.1 |  0.18   |  21.82   |   33.96    |  46.57   |
| fir avx2   |  0.18   |  15.13   |   21.83    |  29.04   |
