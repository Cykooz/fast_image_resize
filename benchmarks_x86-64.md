## Benchmarks of fast_image_resize crate

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.15.0)
- Rust 1.65
- fast_image_resize = "2.4.0"
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
| image      |  20.66  |  82.74   |   141.56   |  199.96  |
| resize     |    -    |  48.75   |   97.42    |  145.70  |
| fir rust   |  0.28   |  39.57   |   67.00    |  98.50   |
| fir sse4.1 |  0.28   |   9.63   |   14.13    |  19.84   |
| fir avx2   |  0.28   |   7.73   |    9.67    |  14.66   |

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel. 

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  65.74   |   130.18   |  194.13  |
| fir rust   |  0.19   |  35.61   |   52.50    |  75.76   |
| fir sse4.1 |  0.19   |  13.19   |   17.25    |  22.62   |
| fir avx2   |  0.19   |   9.57   |   11.97    |  16.37   |

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  16.97  |  48.47   |   77.55    |  107.48  |
| resize     |    -    |  17.10   |   35.46    |  60.94   |
| fir rust   |  0.15   |  13.44   |   14.73    |  22.58   |
| fir sse4.1 |  0.15   |   4.99   |    5.28    |   8.05   |
| fir avx2   |  0.15   |   7.09   |    5.18    |   8.60   |

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
| fir rust   |  0.17   |  25.43   |   28.72    |  39.33   |
| fir sse4.1 |  0.17   |  12.66   |   14.10    |  17.66   |
| fir avx2   |  0.17   |   8.76   |    9.70    |  12.28   |

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  20.82  |  75.42   |   127.53   |  179.75  |
| resize     |    -    |  45.50   |   89.58    |  132.75  |
| fir rust   |  0.33   |  43.32   |   78.99    |  113.80  |
| fir sse4.1 |  0.33   |  24.42   |   39.46    |  55.70   |
| fir avx2   |  0.33   |  20.17   |   30.73    |  36.83   |

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  63.82   |   126.13   |  187.90  |
| fir rust   |  0.30   |  79.65   |   117.15   |  157.65  |
| fir sse4.1 |  0.30   |  43.40   |   64.83    |  86.98   |
| fir avx2   |  0.30   |  25.63   |   36.85    |  48.28   |

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  17.41  |  49.38   |   78.97    |  109.71  |
| resize     |    -    |  15.30   |   31.51    |  56.97   |
| fir rust   |  0.16   |  19.20   |   27.82    |  38.54   |
| fir sse4.1 |  0.16   |   8.07   |   13.39    |  19.30   |
| fir avx2   |  0.16   |   6.61   |    8.60    |  13.67   |

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
| fir rust   |  0.19   |  34.47   |   52.74    |  71.32   |
| fir sse4.1 |  0.19   |  22.10   |   34.18    |  46.62   |
| fir avx2   |  0.19   |  15.17   |   21.85    |  29.09   |
