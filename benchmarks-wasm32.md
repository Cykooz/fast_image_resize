<!-- introduction start -->

## Benchmarks of fast_image_resize crate for Wasm32 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 4000 MHz
- Ubuntu 26.04 (linux 7.0.0)
- Rust 1.99
- criterion = "0.7.0"
- fast_image_resize = "6.2.0"
- wasmtime = "50.0.0"

Other libraries used to compare of resizing speed:

- image = "0.25.6" (<https://crates.io/crates/image>)
- resize = "0.8.9" (<https://crates.io/crates/resize>, single-threaded mode)

Resize algorithms:

- Nearest
- Box - convolution with minimal kernel size 1x1 px
- Bilinear - convolution with minimal kernel size 2x2 px
- Bicubic (CatmullRom) - convolution with minimal kernel size 4x4 px
- Lanczos3 - convolution with minimal kernel size 6x6 px

<!-- introduction end -->

<!-- bench_compare_rgb start -->

### Resize RGB8 image (U8x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
- Numbers in the table mean the duration of the image resizing in milliseconds.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image       |  22.57  |   -   |  100.71  | 180.93  |  258.63  |
| resize      |  11.78  | 31.04 |  56.33   | 107.84  |  168.70  |
| fir rust    |  0.33   | 34.87 |  65.16   | 125.66  |  188.25  |
| fir simd128 |  0.33   | 5.61  |   8.40   |  14.91  |  21.96   |

<!-- bench_compare_rgb end -->

<!-- bench_compare_rgba start -->

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean the duration of the image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| resize      |  11.58  | 39.10 |  77.47   | 144.76  |  225.54  |
| fir rust    |  0.25   | 88.31 |  128.66  | 209.95  |  290.85  |
| fir simd128 |  0.25   | 13.08 |  16.20   |  23.81  |  32.39   |

<!-- bench_compare_rgba end -->

<!-- bench_compare_l start -->

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in the table mean the duration of the image resizing in milliseconds.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image       |  19.04  |   -   |  75.39   | 130.45  |  183.91  |
| resize      |  7.31   | 17.44 |  28.25   |  52.43  |  77.00   |
| fir rust    |  0.21   | 13.26 |  23.66   |  44.90  |  66.92   |
| fir simd128 |  0.21   | 2.55  |   3.40   |  5.40   |   8.03   |

<!-- bench_compare_l end -->

<!-- bench_compare_la start -->

### Resize LA8 image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with an alpha channel (two bytes per pixel).
- Numbers in the table mean the duration of the image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| fir rust    |  0.19   | 44.60 |  65.30   | 108.09  |  150.57  |
| fir simd128 |  0.19   | 8.06  |   9.64   |  13.76  |  18.20   |

<!-- bench_compare_la end -->

<!-- bench_compare_rgb16 start -->

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in the table mean the duration of the image resizing in milliseconds.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image       |  22.21  |   -   |  100.09  | 178.34  |  256.03  |
| resize      |  11.57  | 29.88 |  52.21   | 104.66  |  164.94  |
| fir rust    |  0.39   | 32.53 |  49.07   |  87.14  |  123.43  |
| fir simd128 |  0.39   | 38.35 |  68.16   | 128.54  |  188.49  |

<!-- bench_compare_rgb16 end -->

<!-- bench_compare_rgba16 start -->

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean the duration of the image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| resize      |  13.03  | 38.98 |  73.34   | 141.54  |  208.31  |
| fir rust    |  0.55   | 81.12 |  100.91  | 140.94  |  183.19  |
| fir simd128 |  0.55   | 60.83 |  98.30   | 172.97  |  248.10  |

<!-- bench_compare_rgba16 end -->

<!-- bench_compare_l16 start -->

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in the table mean the duration of the image resizing in milliseconds.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image       |  18.85  |   -   |  82.44   | 145.14  |  206.58  |
| resize      |  7.53   | 16.78 |  27.62   |  52.19  |  76.32   |
| fir rust    |  0.19   | 17.96 |  24.56   |  42.83  |  60.77   |
| fir simd128 |  0.19   | 16.31 |  25.07   |  43.84  |  63.59   |

<!-- bench_compare_l16 end -->

<!-- bench_compare_la16 start -->

### Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with an alpha channel (four bytes per pixel).
- Numbers in the table mean the duration of the image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| fir rust    |  0.27   | 43.60 |  57.43   |  87.19  |  116.41  |
| fir simd128 |  0.27   | 33.11 |  51.75   |  90.60  |  129.95  |

<!-- bench_compare_la16 end -->

<!-- bench_compare_l32f start -->

### Resize L32F image (F32) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in the table mean the duration of the image resizing in milliseconds.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image       |  14.14  |   -   |  48.08   |  87.26  |  121.74  |
| resize      |  6.81   | 15.59 |  23.78   |  46.79  |  71.51   |
| fir rust    |  0.25   | 10.67 |  18.62   |  36.07  |  58.93   |
| fir simd128 |  0.25   | 6.38  |   8.75   |  15.07  |  21.56   |

<!-- bench_compare_l32f end -->

Note:
The `resize` crate uses `f32` for intermediate calculations.
The `fast_image_resize` uses `f64`. This is a reason why `fast_image_resize`
is slower or equal in cases with `f32`-based pixels.

<!-- bench_compare_la32f start -->

### Resize LA32F (luma with alpha channel) image (F32x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with an alpha channel (two `f32` values per pixel).
- Numbers in the table mean the duration of the image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| fir rust    |  0.63   | 35.12 |  47.03   |  73.82  |  101.61  |
| fir simd128 |  0.63   | 26.00 |  32.89   |  46.08  |  60.63   |

<!-- bench_compare_la32f end -->

<!-- bench_compare_rgb32f start -->

### Resize RGB32F image (F32x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB32F image.
- Numbers in the table mean the duration of the image resizing in milliseconds.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image       |  15.84  |   -   |  63.90   | 114.99  |  169.86  |
| resize      |  12.68  | 20.78 |  34.82   |  63.87  |  93.00   |
| fir rust    |  1.28   | 26.75 |  48.15   |  92.28  |  138.60  |
| fir simd128 |  1.28   | 18.43 |  28.67   |  50.96  |  72.31   |

<!-- bench_compare_rgb32f end -->


<!-- bench_compare_rgba32f start -->

### Resize RGBA32F image (F32x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean the duration of the image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support multiplying and dividing by alpha channel
  for this pixel format.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| fir rust    |  1.74   | 59.58 |  82.52   | 127.75  |  178.55  |
| fir simd128 |  1.74   | 47.77 |  58.47   |  83.82  |  109.24  |

<!-- bench_compare_rgba32f end -->