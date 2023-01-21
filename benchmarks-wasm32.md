## Benchmarks of fast_image_resize crate for Wasm32 architecture

Environment:

- CPU: AMD Ryzen 9 5950X
- RAM: DDR4 3800 MHz
- Ubuntu 22.04 (linux 5.15.0)
- Rust 1.66.1
- wasmtime = "5.0.0"
- criterion = "0.4"
- fast_image_resize = "2.4.0"

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

[//]: # (bench_compare_rgb start)
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| image    |  25.52  |  146.82  |   268.35   |  391.29  |
| resize   |    -    |  84.92   |   166.80   |  247.48  |
| fir rust |  0.37   |  50.56   |   88.08    |  126.30  |
[//]: # (bench_compare_rgb end)

### Resize RGBA8 image (U8x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

[//]: # (bench_compare_rgba start)
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| resize   |    -    |  109.02  |   216.79   |  325.27  |
| fir rust |  0.25   |  112.21  |   174.85   |  239.34  |
[//]: # (bench_compare_rgba end)

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

[//]: # (bench_compare_l start)
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| image    |  20.67  |  113.49  |   204.72   |  296.08  |
| resize   |    -    |  30.10   |   60.87    |  90.92   |
| fir rust |  0.21   |  24.54   |   40.10    |  57.36   |
[//]: # (bench_compare_l end)

### Resize LA8 image (U8x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (two bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

[//]: # (bench_compare_la start)
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| fir rust |  0.24   |  58.38   |   87.21    |  116.68  |
[//]: # (bench_compare_la end)

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

[//]: # (bench_compare_rgb16 start)
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| image    |  25.65  |  155.69  |   284.54   |  413.21  |
| resize   |    -    |  88.44   |   172.07   |  258.84  |
| fir rust |  0.40   |  57.66   |   97.19    |  138.74  |
[//]: # (bench_compare_rgb16 end)

### Resize RGBA16 image (U16x4) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.

[//]: # (bench_compare_rgba16 start)
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| resize   |    -    |  111.25  |   219.73   |  328.47  |
| fir rust |  0.42   |  112.28  |   169.04   |  228.52  |
[//]: # (bench_compare_rgba16 end)

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

[//]: # (bench_compare_l16 start)
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| image    |  22.20  |  129.91  |   236.08   |  342.27  |
| resize   |    -    |  30.63   |   60.99    |  91.22   |
| fir rust |  0.25   |  27.74   |   43.73    |  61.72   |
[//]: # (bench_compare_l16 end)

### Resize LA16 (luma with alpha channel) image (U16x2) 4928x3279 => 852x567

Pipeline:

`src_image => multiply by alpha => resize => divide by alpha => dst_image`

- Source image
  [nasa-4928x3279-rgba.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279-rgba.png)
  has converted into grayscale image with alpha channel (four bytes per pixel).
- Numbers in table is mean duration of image resizing in milliseconds.
- The `image` crate does not support multiplying and dividing by alpha channel.
- The `resize` crate does not support this pixel format.

[//]: # (bench_compare_la16 start)
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| fir rust |  0.25   |  59.93   |   95.26    |  131.62  |
[//]: # (bench_compare_la16 end)