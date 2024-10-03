<!-- introduction start -->

## Benchmarks of fast_image_resize crate for Wasm32 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 4000 MHz
- Ubuntu 24.04 (linux 6.8.0)
- Rust 1.81.0
- criterion = "0.5.1"
- fast_image_resize = "5.0.0"
- wasmtime = "25.0.1"

Other libraries used to compare of resizing speed:

- image = "0.25.2" (<https://crates.io/crates/image>)
- resize = "0.8.7" (<https://crates.io/crates/resize>, single-threaded mode)

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
- Numbers in the table mean a duration of image resizing in milliseconds.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image       |  25.61  |   -   |  111.45  | 199.95  |  281.24  |
| resize      |  11.32  | 33.48 |  60.90   | 113.34  |  166.20  |
| fir rust    |  0.36   | 40.56 |  76.04   | 148.68  |  223.70  |
| fir simd128 |  0.36   | 5.19  |   7.93   |  14.07  |  20.86   |

<!-- bench_compare_rgb end -->

<!-- bench_compare_rgba start -->

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| resize      |  12.36  | 39.11 |  73.57   | 140.10  |  216.32  |
| fir rust    |  0.23   | 96.26 |  143.11  | 240.79  |  338.59  |
| fir simd128 |  0.23   | 15.98 |  18.99   |  25.80  |  33.58   |

<!-- bench_compare_rgba end -->

<!-- bench_compare_l start -->

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in the table mean a duration of image resizing in milliseconds.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image       |  23.29  |   -   |  87.60   | 150.35  |  211.42  |
| resize      |  7.79   | 17.22 |  27.25   |  50.98  |  74.74   |
| fir rust    |  0.21   | 15.15 |  27.92   |  54.02  |  81.12   |
| fir simd128 |  0.21   | 2.65  |   3.32   |  5.36   |   7.97   |

<!-- bench_compare_l end -->

<!-- bench_compare_la start -->

### Resize LA8 image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with an alpha channel (two bytes per pixel).
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| fir rust    |  0.19   | 49.46 |  74.31   | 124.11  |  175.58  |
| fir simd128 |  0.19   | 8.02  |   9.41   |  12.85  |  17.58   |

<!-- bench_compare_la end -->

<!-- bench_compare_rgb16 start -->

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in the table mean a duration of image resizing in milliseconds.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image       |  26.75  |   -   |  107.99  | 188.76  |  270.38  |
| resize      |  11.39  | 30.97 |  53.58   | 106.09  |  157.82  |
| fir rust    |  0.42   | 34.58 |  58.12   | 107.33  |  157.86  |
| fir simd128 |  0.42   | 28.62 |  48.16   |  87.52  |  126.68  |

<!-- bench_compare_rgb16 end -->

<!-- bench_compare_rgba16 start -->

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| resize      |  12.73  | 39.77 |  75.15   | 140.39  |  214.27  |
| fir rust    |  0.39   | 89.75 |  123.03  | 185.52  |  250.31  |
| fir simd128 |  0.39   | 46.65 |  70.80   | 121.69  |  170.89  |

<!-- bench_compare_rgba16 end -->

<!-- bench_compare_l16 start -->

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in the table mean a duration of image resizing in milliseconds.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image       |  23.45  |   -   |  79.42   | 134.57  |  188.61  |
| resize      |  8.33   | 16.79 |  26.60   |  50.54  |  73.93   |
| fir rust    |  0.19   | 18.79 |  26.98   |  46.01  |  64.94   |
| fir simd128 |  0.19   | 10.14 |  17.11   |  29.49  |  42.87   |

<!-- bench_compare_l16 end -->

<!-- bench_compare_la16 start -->

### Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with an alpha channel (four bytes per pixel).
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| fir rust    |  0.25   | 44.15 |  59.50   |  91.27  |  125.41  |
| fir simd128 |  0.25   | 26.02 |  38.49   |  65.75  |  92.37   |

<!-- bench_compare_la16 end -->

<!-- bench_compare_l32f start -->

### Resize L32F image (F32) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in the table mean a duration of image resizing in milliseconds.

|          | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image    |  13.04  |   -   |  63.40   | 116.33  |  167.53  |
| resize   |  7.31   | 15.27 |  22.17   |  42.93  |  65.69   |
| fir rust |  0.25   | 10.20 |  18.49   |  35.77  |  59.88   |

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
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|          | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:-----:|:--------:|:-------:|:--------:|
| fir rust |  0.41   | 30.13 |  42.49   |  69.34  |  98.15   |

<!-- bench_compare_la32f end -->

<!-- bench_compare_rgb32f start -->

### Resize RGB32F image (F32x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB32F image.
- Numbers in the table mean a duration of image resizing in milliseconds.

|          | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:-----:|:--------:|:-------:|:--------:|
| image    |  16.34  |   -   |  67.78   | 123.04  |  180.03  |
| resize   |  10.39  | 21.49 |  35.88   |  65.69  |  95.80   |
| fir rust |  1.02   | 25.55 |  45.73   |  86.30  |  128.94  |

<!-- bench_compare_rgb32f end -->


<!-- bench_compare_rgba32f start -->

### Resize RGBA32F image (F32x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support multiplying and dividing by alpha channel
  for this pixel format.

|          | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|----------|:-------:|:-----:|:--------:|:-------:|:--------:|
| fir rust |  1.20   | 50.35 |  73.55   | 118.86  |  168.11  |

<!-- bench_compare_rgba32f end -->
