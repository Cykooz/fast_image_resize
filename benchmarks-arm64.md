## Benchmarks of fast_image_resize crate for arm64 architecture

Environment:

- CPU: Neoverse-N1 2GHz (Oracle Cloud Compute, VM.Standard.A1.Flex)
- Ubuntu 22.04 (linux 5.15.0)
- Rust 1.67
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
| image    |  82.04  |  170.39  |   323.54   |  455.59  |
| resize   |    -    |  89.54   |   178.04   |  265.99  |
| fir rust |  0.89   |  72.03   |   87.92    |  112.94  |
| fir neon |    -    |  43.24   |   57.50    |  82.20   |
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
| resize   |    -    |  114.42  |   197.39   |  290.99  |
| fir rust |  0.95   |  94.07   |   108.51   |  155.94  |
| fir neon |    -    |  44.72   |   61.65    |  84.19   |
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
| image    |  75.34  |  104.26  |   174.57   |  243.13  |
| resize   |    -    |  37.87   |   83.74    |  128.46  |
| fir rust |  0.50   |  29.85   |   37.35    |  45.96   |
| fir neon |    -    |  15.09   |   20.09    |  28.27   |
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
| fir rust |  0.70   |  60.50   |   75.23    |  72.11   |
| fir neon |    -    |  34.26   |   39.25    |  54.73   |
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
| image    |  85.61  |  167.17  |   331.31   |  479.45  |
| resize   |    -    |  90.45   |   179.51   |  267.39  |
| fir rust |  1.37   |  148.73  |   275.24   |  393.32  |
| fir neon |    -    |  70.44   |   92.62    |  128.94  |
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
| resize   |    -    |  119.14  |   202.43   |  301.05  |
| fir rust |  1.49   |  203.02  |   366.89   |  521.69  |
| fir neon |    -    |  77.88   |   118.26   |  159.35  |
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
| image    |  73.89  |  110.24  |   185.96   |  260.32  |
| resize   |    -    |  41.82   |   64.05    |  88.11   |
| fir rust |  0.62   |  57.53   |   95.11    |  134.94  |
| fir neon |    -    |  17.47   |   26.87    |  37.82   |
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
| fir rust |  0.94   |  106.88  |   192.68   |  271.72  |
| fir neon |    -    |  35.87   |   53.97    |  74.00   |
<!-- bench_compare_la16 end -->
