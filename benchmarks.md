## Benchmarks of fast_image_resize crate

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.15.0)
- Rust 1.62.1
- fast_image_resize = "1.0.0"
- glassbench = "0.3.3"

Other Rust libraries used to compare of resizing speed:

- image = "0.24.3" (<https://crates.io/crates/image>)
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
| image      |  19.01  |  84.06   |   148.84   |  205.25  |
| resize     |    -    |  52.21   |   104.32   |  154.11  |
| fir rust   |  0.28   |  41.75   |   76.85    |  113.11  |
| fir sse4.1 |  0.28   |  28.16   |   43.00    |  59.62   |
| fir avx2   |  0.28   |   7.41   |    9.61    |  13.84   |

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel. 

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  61.94   |   122.19   |  182.60  |
| fir rust   |  0.19   |  37.33   |   53.29    |  76.49   |
| fir sse4.1 |  0.19   |  13.14   |   17.25    |  22.59   |
| fir avx2   |  0.19   |   9.59   |   12.09    |  16.18   |

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  15.60  |  45.30   |   72.95    |  102.13  |
| resize     |    -    |  17.29   |   35.45    |  61.14   |
| fir rust   |  0.15   |  13.91   |   15.37    |  22.86   |
| fir sse4.1 |  0.15   |  12.16   |   12.16    |  18.19   |
| fir avx2   |  0.15   |   6.33   |    4.66    |   7.69   |

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
| fir rust   |  0.17   |  25.48   |   31.83    |  43.72   |
| fir sse4.1 |  0.17   |  12.81   |   14.69    |  18.12   |
| fir avx2   |  0.17   |  11.27   |   12.45    |  15.48   |

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  18.28  |  75.10   |   137.04   |  189.89  |
| resize     |    -    |  54.71   |   106.93   |  158.29  |
| fir rust   |  0.33   |  43.65   |   79.56    |  117.31  |
| fir sse4.1 |  0.33   |  24.34   |   39.55    |  55.89   |
| fir avx2   |  0.33   |  20.67   |   30.81    |  36.98   |

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  63.93   |   127.61   |  191.17  |
| fir rust   |  0.40   |  89.83   |   125.96   |  167.05  |
| fir sse4.1 |  0.40   |  42.24   |   63.53    |  85.53   |
| fir avx2   |  0.40   |  23.53   |   34.14    |  45.43   |

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  15.53  |  45.82   |   73.87    |  103.21  |
| resize     |    -    |  15.44   |   31.92    |  57.15   |
| fir rust   |  0.18   |  18.86   |   27.52    |  38.34   |
| fir sse4.1 |  0.18   |   7.93   |   13.28    |  19.24   |
| fir avx2   |  0.18   |   7.16   |    9.50    |  14.54   |

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
| fir rust   |  0.19   |  33.54   |   52.84    |  71.97   |
| fir sse4.1 |  0.19   |  21.87   |   34.08    |  46.61   |
| fir avx2   |  0.19   |  15.19   |   21.93    |  29.07   |
