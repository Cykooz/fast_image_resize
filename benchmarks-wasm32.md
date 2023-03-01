## Benchmarks of fast_image_resize crate for Wasm32 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.19.0)
- Rust 1.67.1
- wasmtime = "6.0.0"
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
| image       |  28.76  |  173.68  |   317.43   |  471.25  |
| resize      |    -    |  60.02   |   112.12   |  164.45  |
| fir rust    |  0.39   |  64.55   |   112.09   |  160.57  |
| fir simd128 |    -    |  19.24   |   26.81    |  37.65   |
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
| resize   |    -    |  113.48  |   220.71   |  329.72  |
| fir rust |  0.28   |  108.67  |   165.42   |  225.15  |
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
| image       |  25.92  |  150.91  |   282.91   |  414.63  |
| resize      |    -    |  30.59   |   62.37    |  92.67   |
| fir rust    |  0.24   |  31.30   |   51.98    |  74.33   |
| fir simd128 |    -    |   8.68   |    9.01    |  13.46   |
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
| fir rust |  0.28   |  60.44   |   92.25    |  123.84  |
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
| image       |  28.94  |  173.64  |   317.43   |  471.87  |
| resize      |    -    |  60.21   |   167.97   |  248.82  |
| fir rust    |  0.42   |  72.90   |   125.37   |  177.67  |
| fir simd128 |    -    |  61.07   |   105.06   |  151.93  |
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
| resize   |    -    |  106.03  |   206.32   |  311.74  |
| fir rust |  0.45   |  117.19  |   172.46   |  230.54  |
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
| image       |  25.78  |  156.37  |   284.77   |  423.77  |
| resize      |    -    |  30.57   |   62.63    |  93.16   |
| fir rust    |  0.26   |  36.22   |   58.49    |  82.46   |
| fir simd128 |    -    |  22.11   |   35.98    |  51.49   |
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
| fir rust |  0.28   |  64.02   |   98.11    |  132.66  |
<!-- bench_compare_la16 end -->