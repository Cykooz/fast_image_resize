<!-- introduction start -->

## Benchmarks of fast_image_resize crate for Wasm32 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 4000 MHz
- Ubuntu 22.04 (linux 6.5.0)
- Rust 1.79
- criterion = "0.5.1"
- fast_image_resize = "4.1.0"
- wasmtime = "22.0.0"

Other libraries used to compare of resizing speed:

- image = "0.25.1" (<https://crates.io/crates/image>)
- resize = "0.8.4" (<https://crates.io/crates/resize>, single-threaded mode)

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
| image       |  26.19  |   -   |  103.03  | 180.66  |  258.39  |
| resize      |  11.71  | 33.22 |  60.30   | 114.09  |  167.14  |
| fir rust    |  0.39   | 45.14 |  79.53   | 150.42  |  223.19  |
| fir simd128 |  0.39   | 6.41  |   8.71   |  14.33  |  21.60   |

<!-- bench_compare_rgb end -->

<!-- bench_compare_rgba start -->

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

|             | Nearest |  Box   | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:------:|:--------:|:-------:|:--------:|
| resize      |  12.41  | 38.94  |  73.52   | 138.90  |  211.97  |
| fir rust    |  0.27   | 101.41 |  147.99  | 244.04  |  341.24  |
| fir simd128 |  0.27   | 16.30  |  19.02   |  25.52  |  33.58   |

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
| image       |  24.25  |   -   |  87.19   | 149.64  |  210.43  |
| resize      |  7.87   | 17.80 |  28.64   |  53.29  |  77.57   |
| fir rust    |  0.21   | 16.50 |  28.23   |  52.74  |  78.04   |
| fir simd128 |  0.21   | 3.02  |   3.32   |  4.66   |   7.59   |

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
| fir rust    |  0.20   | 50.44 |  73.54   | 120.60  |  168.31  |
| fir simd128 |  0.20   | 8.49  |   9.75   |  12.51  |  17.40   |

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
| image       |  27.31  |   -   |  106.09  | 188.35  |  270.09  |
| resize      |  12.10  | 33.83 |  60.20   | 113.85  |  168.18  |
| fir rust    |  0.41   | 39.02 |  62.29   | 107.96  |  154.74  |
| fir simd128 |  0.41   | 32.47 |  51.34   |  89.26  |  129.12  |

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
| resize      |  12.69  | 39.35 |  74.01   | 139.45  |  213.46  |
| fir rust    |  0.34   | 87.33 |  110.98  | 158.35  |  207.47  |
| fir simd128 |  0.34   | 51.84 |  75.53   | 123.10  |  172.10  |

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
| image       |  23.98  |   -   |  86.94   | 149.67  |  211.09  |
| resize      |  7.88   | 17.12 |  27.73   |  52.04  |  75.31   |
| fir rust    |  0.19   | 21.90 |  30.69   |  49.16  |  69.15   |
| fir simd128 |  0.19   | 12.02 |  17.84   |  29.47  |  42.84   |

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
| fir rust    |  0.23   | 49.69 |  66.52   |  98.05  |  131.53  |
| fir simd128 |  0.23   | 28.41 |  40.62   |  65.56  |  92.21   |

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
| image    |  12.80  |   -   |  55.61   | 101.29  |  145.08  |
| resize   |  7.49   | 16.02 |  22.29   |  43.93  |  64.19   |
| fir rust |  0.24   | 15.47 |  25.58   |  48.98  |  72.55   |

<!-- bench_compare_l32f end -->

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
| fir rust |  0.36   | 33.44 |  46.92   |  75.38  |  105.46  |

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
| image    |  16.25  |   -   |  66.81   | 122.03  |  177.05  |
| resize   |  10.81  | 21.69 |  36.33   |  66.61  |  97.10   |
| fir rust |  1.06   | 32.97 |  56.36   | 104.53  |  155.20  |

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
| fir rust |  1.25   | 59.62 |  84.12   | 137.68  |  192.68  |

<!-- bench_compare_rgba32f end -->
