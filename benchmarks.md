## Benchmarks of fast_image_resize crate

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.15.0)
- Rust 1.65
- fast_image_resize = "2.1.0"
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
| image      |  21.19  |  83.63   |   143.36   |  204.99  |
| resize     |    -    |  45.90   |   90.65    |  134.75  |
| fir rust   |  0.28   |  38.73   |   75.43    |  111.71  |
| fir sse4.1 |  0.28   |   9.35   |   13.63    |  19.20   |
| fir avx2   |  0.28   |   7.36   |    9.52    |  13.66   |

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel. 

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  65.62   |   129.84   |  193.56  |
| fir rust   |  0.19   |  37.38   |   53.83    |  76.69   |
| fir sse4.1 |  0.19   |  13.02   |   17.18    |  22.33   |
| fir avx2   |  0.19   |   9.56   |   12.04    |  16.27   |

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  17.56  |  49.20   |   77.55    |  106.07  |
| resize     |    -    |  17.71   |   36.82    |  62.15   |
| fir rust   |  0.15   |  13.36   |   14.81    |  22.22   |
| fir sse4.1 |  0.15   |   4.51   |    5.08    |   7.61   |
| fir avx2   |  0.15   |   6.00   |    4.77    |   7.46   |

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
| fir rust   |  0.17   |  25.72   |   30.48    |  42.69   |
| fir sse4.1 |  0.17   |  12.44   |   14.26    |  17.69   |
| fir avx2   |  0.17   |   8.40   |    9.53    |  12.09   |

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  21.20  |  76.55   |   129.51   |  180.79  |
| resize     |    -    |  45.34   |   89.33    |  132.50  |
| fir rust   |  0.33   |  43.41   |   80.01    |  114.53  |
| fir sse4.1 |  0.33   |  24.01   |   39.11    |  55.22   |
| fir avx2   |  0.33   |  19.32   |   29.30    |  35.98   |

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  63.72   |   126.13   |  188.03  |
| fir rust   |  0.36   |  83.15   |   120.73   |  160.78  |
| fir sse4.1 |  0.36   |  42.33   |   63.74    |  85.79   |
| fir avx2   |  0.36   |  23.74   |   34.94    |  46.48   |

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  17.82  |  49.90   |   78.51    |  107.97  |
| resize     |    -    |  15.37   |   31.78    |  57.42   |
| fir rust   |  0.17   |  19.02   |   28.93    |  41.06   |
| fir sse4.1 |  0.17   |   7.81   |   13.20    |  19.09   |
| fir avx2   |  0.17   |   6.37   |    8.58    |  13.58   |

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
| fir rust   |  0.18   |  33.83   |   53.19    |  72.56   |
| fir sse4.1 |  0.18   |  21.88   |   33.94    |  46.75   |
| fir avx2   |  0.18   |  15.20   |   21.89    |  29.11   |
