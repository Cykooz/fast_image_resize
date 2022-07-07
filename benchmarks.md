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
| image      |  19.50  |  83.55   |   142.66   |  202.49  |
| resize     |    -    |  52.12   |   102.98   |  153.42  |
| fir rust   |  0.28   |  40.94   |   69.96    |  100.86  |
| fir sse4.1 |  0.28   |  28.10   |   43.06    |  58.05   |
| fir avx2   |  0.28   |   7.24   |    9.49    |  13.61   |

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel. 

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  61.96   |   122.09   |  182.24  |
| fir rust   |  0.19   |  36.38   |   52.06    |  74.03   |
| fir sse4.1 |  0.19   |  13.37   |   17.44    |  22.63   |
| fir avx2   |  0.19   |   9.76   |   12.18    |  16.30   |

### Resize L8 (luma) image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  15.95  |  47.09   |   74.65    |  103.92  |
| resize     |    -    |  17.28   |   35.54    |  61.15   |
| fir rust   |  0.16   |  14.33   |   16.25    |  24.35   |
| fir sse4.1 |  0.16   |  12.17   |   12.18    |  18.41   |
| fir avx2   |  0.16   |   6.31   |    4.66    |   7.97   |

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
| fir rust   |  0.18   |  25.79   |   30.79    |  42.38   |
| fir sse4.1 |  0.17   |  12.71   |   14.68    |  18.16   |
| fir avx2   |  0.17   |  11.25   |   12.53    |  15.53   |

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  19.09  |  76.86   |   140.93   |  191.14  |
| resize     |    -    |  55.19   |   107.76   |  159.66  |
| fir rust   |  0.33   |  43.89   |   80.03    |  117.89  |
| fir sse4.1 |  0.33   |  24.46   |   39.45    |  55.91   |
| fir avx2   |  0.33   |  21.01   |   31.07    |  36.95   |

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  62.92   |   124.11   |  185.29  |
| fir rust   |  0.38   |  84.98   |   123.53   |  163.92  |
| fir sse4.1 |  0.38   |  42.35   |   63.57    |  85.62   |
| fir avx2   |  0.38   |  23.60   |   34.29    |  45.60   |

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  16.43  |  48.12   |   75.85    |  104.63  |
| resize     |    -    |  15.48   |   32.05    |  57.16   |
| fir rust   |  0.17   |  19.31   |   26.93    |  37.76   |
| fir sse4.1 |  0.18   |   7.87   |   13.22    |  19.26   |
| fir avx2   |  0.18   |   7.17   |    9.62    |  14.73   |

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
| fir rust   |  0.19   |  34.70   |   54.59    |  74.64   |
| fir sse4.1 |  0.19   |  22.71   |   34.82    |  47.46   |
| fir avx2   |  0.20   |  15.68   |   22.42    |  29.13   |
