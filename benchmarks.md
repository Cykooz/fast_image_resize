# Benchmarks of fast_image_resize crate

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

## Resize RGB8 image (U8x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  19.44  |  83.01   |   153.17   |  208.82  |
| resize     |    -    |  52.13   |   103.37   |  154.10  |
| fir rust   |  0.28   |  43.00   |   79.52    |  117.41  |
| fir sse4.1 |  0.28   |  27.79   |   42.97    |  58.16   |
| fir avx2   |  0.28   |   7.30   |    9.50    |  13.59   |

## Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  19.73  |  82.34   |   141.74   |  198.86  |
| resize     |    -    |  49.91   |   100.27   |  148.99  |
| fir rust   |  0.18   |  36.84   |   52.31    |  74.99   |
| fir sse4.1 |  0.18   |  13.21   |   17.26    |  22.42   |
| fir avx2   |  0.18   |   9.47   |   12.03    |  16.08   |

## Resize L8 (luma) image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  15.21  |  44.78   |   71.92    |  100.91  |
| resize     |    -    |  17.53   |   36.20    |  61.10   |
| fir rust   |  0.15   |  14.08   |   16.38    |  23.73   |
| fir sse4.1 |  0.16   |  11.92   |   12.28    |  17.79   |
| fir avx2   |  0.16   |   6.48   |    4.77    |   7.85   |

## Resize LA8 (luma with alpha channel) image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (two bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `resize` crate does not support this pixel format.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  17.03  |  63.71   |   119.64   |  160.06  |
| fir rust   |  0.17   |  25.20   |   31.18    |  42.83   |
| fir sse4.1 |  0.17   |  12.88   |   14.73    |  18.16   |
| fir avx2   |  0.17   |  11.26   |   12.40    |  15.40   |

## Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  17.79  |  74.65   |   133.43   |  185.33  |
| resize     |    -    |  54.94   |   107.17   |  159.07  |
| fir rust   |  0.32   |  43.85   |   80.21    |  117.04  |
| fir sse4.1 |  0.32   |  24.46   |   39.49    |  56.00   |
| fir avx2   |  0.32   |  20.56   |   30.36    |  36.07   |

## Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  15.38  |  46.00   |   74.00    |  102.80  |
| resize     |    -    |  15.48   |   32.07    |  57.20   |
| fir rust   |  0.17   |  19.20   |   28.51    |  37.54   |
| fir sse4.1 |  0.17   |   7.74   |   13.16    |  19.11   |
| fir avx2   |  0.17   |   7.06   |    9.55    |  14.82   |

## Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (four bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `resize` crate does not support this pixel format.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  17.05  |  64.88   |   117.68   |  159.22  |
| fir rust   |  0.19   |  33.85   |   52.56    |  72.33   |
| fir sse4.1 |  0.19   |  21.80   |   33.98    |  46.49   |
| fir avx2   |  0.19   |  15.11   |   21.76    |  28.95   |
