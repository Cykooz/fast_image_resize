## Benchmarks of fast_image_resize crate for Wasm32 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.19.0)
- Rust 1.69.0
- wasmtime = "8.0.0"
- criterion = "0.4"
- fast_image_resize = "2.7.1"

Other Rust libraries used to compare of resizing speed:

- image = "0.24.6" (<https://crates.io/crates/image>)
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
| image       |  23.16  |  99.13   |   173.28   |  247.24  |
| resize      |    -    |  53.26   |   100.84   |  148.86  |
| fir rust    |  0.39   |  56.20   |   96.83    |  137.63  |
| fir simd128 |    -    |  15.76   |   21.15    |  29.93   |
<!-- bench_compare_rgb end -->

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

<!-- bench_compare_rgba start -->
|             | Nearest | Bilinear | CatmullRom | Lanczos3 |
|-------------|:-------:|:--------:|:----------:|:--------:|
| resize      |    -    |  70.26   |   135.86   |  201.70  |
| fir rust    |  0.29   |  109.08  |   161.29   |  211.15  |
| fir simd128 |    -    |  18.99   |   24.47    |  32.11   |
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
| image       |  19.97  |  76.69   |   130.46   |  183.81  |
| resize      |    -    |  24.89   |   47.08    |  69.85   |
| fir rust    |  0.23   |  31.20   |   49.05    |  69.19   |
| fir simd128 |    -    |   8.62   |    8.73    |  13.31   |
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
|             | Nearest | Bilinear | CatmullRom | Lanczos3 |
|-------------|:-------:|:--------:|:----------:|:--------:|
| fir rust    |  0.22   |  61.12   |   94.63    |  129.43  |
| fir simd128 |    -    |  20.28   |   21.32    |  27.25   |
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
| image       |  23.09  |  104.82  |   196.11   |  285.95  |
| resize      |    -    |  54.00   |   104.82   |  155.54  |
| fir rust    |  0.45   |  67.27   |   111.68   |  157.72  |
| fir simd128 |    -    |  53.94   |   91.66    |  131.08  |
<!-- bench_compare_rgb16 end -->

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

<!-- bench_compare_rgba16 start -->
|             | Nearest | Bilinear | CatmullRom | Lanczos3 |
|-------------|:-------:|:--------:|:----------:|:--------:|
| resize      |    -    |  73.08   |   139.06   |  205.89  |
| fir rust    |  0.40   |  131.46  |   188.91   |  246.28  |
| fir simd128 |    -    |  76.39   |   124.16   |  173.10  |
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
| image       |  21.17  |  77.63   |   131.44   |  185.26  |
| resize      |    -    |  30.48   |   61.50    |  91.86   |
| fir rust    |  0.23   |  32.81   |   49.09    |  69.09   |
| fir simd128 |    -    |  19.42   |   30.96    |  44.40   |
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
|             | Nearest | Bilinear | CatmullRom | Lanczos3 |
|-------------|:-------:|:--------:|:----------:|:--------:|
| fir rust    |  0.29   |  77.94   |   118.99   |  159.79  |
| fir simd128 |    -    |  40.72   |   65.98    |  92.33   |
<!-- bench_compare_la16 end -->