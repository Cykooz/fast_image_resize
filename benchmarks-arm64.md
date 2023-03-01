## Benchmarks of fast_image_resize crate for arm64 architecture

Environment:

- CPU: Neoverse-N1 2GHz (Oracle Cloud Compute, VM.Standard.A1.Flex)
- Ubuntu 22.04 (linux 5.19.0)
- Rust 1.67.1
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
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| image    |  82.14  |  168.74  |   320.90   |  450.42  |
| resize   |    -    |  89.75   |   177.52   |  265.89  |
| fir rust |  0.88   |  71.87   |   86.61    |  112.92  |
| fir neon |    -    |  42.62   |   57.11    |  81.55   |
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
| resize   |    -    |  115.05  |   196.68   |  291.51  |
| fir rust |  0.93   |  93.79   |   107.13   |  155.10  |
| fir neon |    -    |  44.79   |   62.46    |  83.87   |
<!-- bench_compare_rgba end -->

### Resize L8 image (U8) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with one byte per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

<!-- bench_compare_l start -->
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| image    |  75.25  |  104.16  |   174.73   |  242.71  |
| resize   |    -    |  37.83   |   83.66    |  128.53  |
| fir rust |  0.48   |  29.92   |   38.04    |  45.57   |
| fir neon |    -    |  15.03   |   20.11    |  28.18   |
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
| fir rust |  0.67   |  60.13   |   75.12    |  71.88   |
| fir neon |    -    |  34.48   |   39.38    |  54.70   |
<!-- bench_compare_la end -->

### Resize RGB16 image (U16x3) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into RGB16 image.
- Numbers in table is mean duration of image resizing in milliseconds.

<!-- bench_compare_rgb16 start -->
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| image    |  84.64  |  163.62  |   324.75   |  470.06  |
| resize   |    -    |  90.04   |   179.15   |  266.92  |
| fir rust |  1.37   |  148.80  |   275.50   |  392.99  |
| fir neon |    -    |  70.09   |   92.15    |  128.53  |
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
| resize   |    -    |  119.04  |   202.65   |  300.32  |
| fir rust |  1.50   |  202.83  |   366.23   |  521.08  |
| fir neon |    -    |  77.24   |   117.97   |  159.74  |
<!-- bench_compare_rgba16 end -->

### Resize L16 image (U16) 4928x3279 => 852x567

Pipeline:

`src_image => resize => dst_image`

- Source image [nasa-4928x3279.png](https://github.com/Cykooz/fast_image_resize/blob/main/data/nasa-4928x3279.png)
  has converted into grayscale image with two bytes per pixel.
- Numbers in table is mean duration of image resizing in milliseconds.

<!-- bench_compare_l16 start -->
|          | Nearest | Bilinear | CatmullRom | Lanczos3 |
|----------|:-------:|:--------:|:----------:|:--------:|
| image    |  75.81  |  109.53  |   186.26   |  258.33  |
| resize   |    -    |  41.47   |   64.49    |  88.17   |
| fir rust |  0.63   |  57.12   |   95.16    |  135.02  |
| fir neon |    -    |  17.47   |   26.64    |  37.81   |
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
| fir rust |  0.92   |  106.64  |   192.31   |  271.17  |
| fir neon |    -    |  35.71   |   53.88    |  73.40   |
<!-- bench_compare_la16 end -->
