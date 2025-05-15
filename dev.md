# Preparation

Install system libraries:

- libvips-dev (used in benchmarks)

Install additional toolchains:

- Arm64:
  ```shell
  rustup target add aarch64-unknown-linux-gnu
  ```
- Wasm32:
  ```shell
  rustup target add wasm32-wasip2
  ```
  Install [Wasmtime](https://wasmtime.dev/).

# Tests

Run tests with saving result images as files in `./data` directory:

```shell
SAVE_RESULT=1 cargo test
```

# Benchmarks

Run benchmarks to compare with other crates for image resizing and write results into
report files, such as `./benchmarks-x86_64.md`:

```shell
WRITE_COMPARE_RESULT=1 cargo bench -- Compare
```

If you want to use old benchmark results for other crates, you must add
an env variable with the number of days as a result lifetime:

```shell
WRITE_COMPARE_RESULT=1 RESULTS_LIFETIME=5 cargo bench -- Compare
```

# Wasm32

Specify build target and runner in `.cargo/config.toml` file.

```toml
[build]
target = "wasm32-wasip2"

[target.wasm32-wasip2]
runner = "wasmtime --dir=. --"
```

Run tests:

```shell
cargo test
```

Run tests with saving result images as files in `./data` directory:

```shell
CARGO_TARGET_WASM32_WASIP2_RUNNER="wasmtime --dir=. --env SAVE_RESULT=1 --" cargo test
```

Run a specific benchmark in `quick` mode:

```shell
cargo bench --bench bench_resize -- --color=always --quick
```

Run benchmarks to compare with other crates for image resizing and write results into
report files, such as `./benchmarks-wasm32.md`:

```shell
CARGO_TARGET_WASM32_WASIP2_RUNNER="wasmtime --dir=. --env WRITE_COMPARE_RESULT=1 --" cargo bench --no-fail-fast -- --color=always Compare
```
