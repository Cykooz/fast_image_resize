## Benchmarks of fast_image_resize crate for Wasm32 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.15.0)
- Rust 1.67
- wasmtime = "5.0.0"
- criterion = "0.4"
- fast_image_resize = "2.5.0"

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

<!-- bench_compare_rgb start -->
|             | Nearest | Bilinear | CatmullRom | Lanczos3 |
|-------------|:-------:|:--------:|:----------:|:--------:|
| image       |  30.54  |  182.53  |   333.69   |  485.57  |
| resize      |    -    |  85.65   |   166.46   |  248.04  |
| fir rust    |  0.37   |  56.17   |   95.14    |  136.15  |
| fir simd128 |    -    |  16.32   |   23.87    |  33.97   |
<!-- bench_compare_rgb end -->

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

<!-- bench_compare_rgba start -->
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| resize   |    -    |  106.79  |   204.81   |  309.52  |
| fir rust |  0.27   |  107.92  |   162.37   |  218.41  |
<!-- bench_compare_rgba end -->

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

<!-- bench_compare_l start -->
|             | Nearest | Bilinear | CatmullRom | Lanczos3 |
|-------------|:-------:|:--------:|:----------:|:--------:|
| image       |  26.91  |  149.62  |   270.65   |  402.29  |
| resize      |    -    |  24.86   |   46.99    |  69.49   |
| fir rust    |  0.21   |  25.26   |   40.43    |  56.80   |
| fir simd128 |    -    |   7.66   |    8.62    |  12.87   |
<!-- bench_compare_l end -->

### Resize LA8 image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (two bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

<!-- bench_compare_la start -->
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| fir rust |  0.24   |  58.78   |   88.65    |  117.81  |
<!-- bench_compare_la end -->

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

<!-- bench_compare_rgb16 start -->
|             | Nearest | Bilinear | CatmullRom | Lanczos3 |
|-------------|:-------:|:--------:|:----------:|:--------:|
| image       |  30.21  |  183.61  |   334.87   |  485.75  |
| resize      |    -    |  54.97   |   106.92   |  158.51  |
| fir rust    |  0.42   |  63.58   |   105.10   |  146.28  |
| fir simd128 |    -    |  59.84   |   104.52   |  151.31  |
<!-- bench_compare_rgb16 end -->

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

<!-- bench_compare_rgba16 start -->
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| resize   |    -    |  80.98   |   158.27   |  230.59  |
| fir rust |  0.42   |  117.66  |   171.42   |  227.46  |
<!-- bench_compare_rgba16 end -->

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

<!-- bench_compare_l16 start -->
|             | Nearest | Bilinear | CatmullRom | Lanczos3 |
|-------------|:-------:|:--------:|:----------:|:--------:|
| image       |  27.74  |  174.53  |   315.94   |  471.17  |
| resize      |    -    |  30.87   |   61.00    |  91.44   |
| fir rust    |  0.25   |  28.48   |   44.72    |  62.67   |
| fir simd128 |    -    |  20.61   |   33.51    |  48.61   |
<!-- bench_compare_l16 end -->

### Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (four bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

<!-- bench_compare_la16 start -->
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| fir rust |  0.26   |  58.50   |   87.86    |  117.28  |
<!-- bench_compare_la16 end -->