## Benchmarks of fast_image_resize crate

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.15.0)
- Rust 1.62.0
- fast_image_resize = "0.9.7"
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
| image      |  18.75  |  83.16   |   148.86   |  205.11  |
| resize     |    -    |  52.27   |   103.40   |  154.10  |
| fir rust   |  0.28   |  41.93   |   76.81    |  114.10  |
| fir sse4.1 |  0.28   |  27.98   |   43.16    |  58.43   |
| fir avx2   |  0.28   |   7.42   |    9.71    |  13.91   |

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel. 

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  61.37   |   122.00   |  181.84  |
| fir rust   |  0.18   |  37.42   |   53.36    |  75.87   |
| fir sse4.1 |  0.18   |  13.33   |   17.40    |  22.71   |
| fir avx2   |  0.18   |   9.69   |   12.09    |  16.35   |

### Resize L8 (luma) image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  15.48  |  46.20   |   73.56    |  101.74  |
| resize     |    -    |  17.29   |   35.60    |  61.27   |
| fir rust   |  0.16   |  13.79   |   15.31    |  22.76   |
| fir sse4.1 |  0.16   |  12.01   |   12.12    |  18.10   |
| fir avx2   |  0.16   |   6.29   |    4.81    |   7.71   |

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
| fir rust   |  0.17   |  25.97   |   31.30    |  43.20   |
| fir sse4.1 |  0.17   |  12.73   |   14.69    |  18.16   |
| fir avx2   |  0.17   |  11.24   |   12.51    |  15.45   |

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  18.34  |  75.56   |   136.94   |  186.09  |
| resize     |    -    |  55.09   |   107.12   |  158.71  |
| fir rust   |  0.33   |  43.65   |   79.55    |  117.51  |
| fir sse4.1 |  0.33   |  24.39   |   39.47    |  55.81   |
| fir avx2   |  0.33   |  20.79   |   30.79    |  36.76   |

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  63.90   |   127.59   |  191.40  |
| fir rust   |  0.37   |  87.77   |   125.78   |  165.53  |
| fir sse4.1 |  0.37   |  42.54   |   63.71    |  85.59   |
| fir avx2   |  0.37   |  23.43   |   34.04    |  45.37   |

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  15.70  |  45.79   |   73.63    |  103.38  |
| resize     |    -    |  15.52   |   32.13    |  57.22   |
| fir rust   |  0.17   |  18.51   |   27.75    |  38.42   |
| fir sse4.1 |  0.17   |   7.77   |   13.21    |  19.16   |
| fir avx2   |  0.17   |   7.07   |    9.54    |  14.81   |

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
| fir rust   |  0.19   |  33.85   |   53.43    |  72.17   |
| fir sse4.1 |  0.19   |  21.92   |   34.05    |  46.44   |
| fir avx2   |  0.19   |  15.16   |   21.93    |  28.98   |
