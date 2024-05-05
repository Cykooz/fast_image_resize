<!-- introduction start -->

## Benchmarks of fast_image_resize crate for Wasm32 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 4000 MHz
- Ubuntu 22.04 (linux 6.5.0)
- Rust 1.78
- criterion = "0.5.1"
- fast_image_resize = "4.0.0"
- wasmtime = "20.0.0"

Other libraries used to compare of resizing speed:

- image = "0.25.1" (<https://crates.io/crates/image>)
- resize = "0.8.4" (<https://crates.io/crates/resize>)

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
| image       |  17.34  |   -   |  101.44  | 186.62  |  270.94  |
| resize      |  10.98  | 33.34 |  61.39   | 116.05  |  182.75  |
| fir rust    |  0.38   | 44.44 |  77.71   | 145.67  |  213.76  |
| fir simd128 |  0.38   | 6.22  |   8.33   |  13.44  |  20.59   |

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
| resize      |  11.88  | 38.43  |  71.56   | 137.42  |  208.90  |
| fir rust    |  0.26   | 108.03 |  161.33  | 272.43  |  384.54  |
| fir simd128 |  0.26   | 12.73  |  15.23   |  21.37  |  29.25   |

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
| image       |  16.36  |   -   |  79.55   | 142.10  |  203.30  |
| resize      |  6.73   | 17.18 |  30.31   |  57.66  |  83.45   |
| fir rust    |  0.21   | 16.02 |  27.48   |  50.59  |  74.72   |
| fir simd128 |  0.21   | 2.93  |   3.30   |  4.73   |   7.66   |

<!-- bench_compare_l end -->

<!-- bench_compare_la start -->

### Resize LA8 image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (two bytes per pixel).
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| fir rust    |  0.20   | 51.14 |  75.07   | 123.35  |  171.89  |
| fir simd128 |  0.20   | 7.59  |   8.80   |  11.44  |  16.33   |

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
| image       |  18.13  |   -   |  95.31   | 173.33  |  251.98  |
| resize      |  11.03  | 30.34 |  55.99   | 107.76  |  152.73  |
| fir rust    |  0.42   | 42.11 |  64.42   | 109.79  |  157.11  |
| fir simd128 |  0.42   | 32.56 |  51.45   |  89.52  |  129.08  |

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
| resize      |  12.49  | 39.73 |  74.62   | 140.65  |  211.84  |
| fir rust    |  0.41   | 94.07 |  126.42  | 190.18  |  256.18  |
| fir simd128 |  0.41   | 51.49 |  75.52   | 123.99  |  173.44  |

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
| image       |  17.78  |   -   |  81.33   | 143.51  |  206.46  |
| resize      |  8.06   | 18.72 |  29.77   |  54.94  |  79.38   |
| fir rust    |  0.19   | 21.71 |  31.40   |  48.02  |  65.33   |
| fir simd128 |  0.19   | 12.15 |  18.24   |  30.03  |  43.44   |

<!-- bench_compare_l16 end -->

<!-- bench_compare_la16 start -->

### Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (four bytes per pixel).
- Numbers in the table mean a duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

|             | Nearest |  Box  | Bilinear | Bicubic | Lanczos3 |
|-------------|:-------:|:-----:|:--------:|:-------:|:--------:|
| fir rust    |  0.25   | 49.12 |  64.81   |  94.16  |  123.91  |
| fir simd128 |  0.25   | 28.50 |  40.57   |  65.66  |  92.68   |

<!-- bench_compare_la16 end -->
