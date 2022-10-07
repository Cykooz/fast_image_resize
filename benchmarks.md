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
| image      |  19.37  |  82.24   |   141.62   |  200.90  |
| resize     |    -    |  51.49   |   102.74   |  153.21  |
| fir rust   |  0.28   |  44.07   |   81.14    |  119.43  | ?
| fir sse4.1 |  0.28   |  28.22   |   43.29    |  57.77   |
| fir avx2   |  0.28   |   7.47   |    9.65    |  13.86   |

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel. 

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  62.30   |   122.50   |  182.64  |
| fir rust   |  0.19   |  37.27   |   53.06    |  75.83   |
| fir sse4.1 |  0.19   |  13.20   |   17.29    |  22.74   |
| fir avx2   |  0.19   |   9.58   |   12.09    |  16.24   |

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  15.80  |  46.71   |   74.50    |  102.36  |
| resize     |    -    |  17.33   |   35.75    |  61.46   |
| fir rust   |  0.16   |  14.63   |   16.86    |  24.73   | ?
| fir sse4.1 |  0.16   |  12.32   |   12.37    |  18.32   |
| fir avx2   |  0.16   |   6.29   |    4.69    |   7.94   |

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
| fir rust   |  0.17   |  25.97   |   31.36    |  43.22   |
| fir sse4.1 |  0.17   |  12.77   |   14.64    |  18.07   |
| fir avx2   |  0.17   |  11.26   |   12.45    |  15.45   |

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  19.08  |  75.99   |   138.66   |  191.07  |
| resize     |    -    |  54.82   |   106.80   |  158.63  |
| fir rust   |  0.33   |  42.57   |   71.01    |  100.55  |
| fir sse4.1 |  0.33   |  24.41   |   39.39    |  55.82   |
| fir avx2   |  0.33   |  20.59   |   30.53    |  36.28   |

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| resize     |    -    |  63.87   |   127.55   |  191.89  |
| fir rust   |  0.36   |  85.57   |   124.24   |  163.89  |
| fir sse4.1 |  0.36   |  42.25   |   63.57    |  85.65   |
| fir avx2   |  0.36   |  23.51   |   34.20    |  45.55   |

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

|            | Nearest | Bilinear | CatmullRom | Lanczos3 |
|------------|:-------:|:--------:|:----------:|:--------:|
| image      |  15.81  |  48.15   |   75.73    |  104.58  |
| resize     |    -    |  15.46   |   32.02    |  57.12   |
| fir rust   |  0.17   |  18.98   |   29.12    |  41.17   | ?
| fir sse4.1 |  0.17   |   7.79   |   13.24    |  19.14   |
| fir avx2   |  0.17   |   7.09   |    9.49    |  14.83   |

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
| fir rust   |  0.19   |  33.72   |   52.63    |  71.65   |
| fir sse4.1 |  0.19   |  21.80   |   33.99    |  46.62   |
| fir avx2   |  0.19   |  15.10   |   21.85    |  29.03   |
