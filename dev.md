# Preparation

Install additional toolchains.
- Arm64:
  ```shell
  rustup target add aarch64-unknown-linux-gnu
  ```
- Wasm32:  
  ```shell
  rustup target add wasm32-wasi
  ```
  Install [Wasmtime](https://wasmtime.dev/).

# Tests

Run tests without saving result images as files in `./data` directory:
```shell
DONT_SAVE_RESULT=1 cargo test
```

# Benchmarks

Run benchmarks to compare with other crates for image resizing and write results into
report files, such as `./benchmarks-x86_64.md`:
```shell
WRITE_COMPARE_RESULT=1 cargo bench -- Compare
```


# Wasm32

Specify build target in `.cargo/config.toml` file.
```toml
[build]
target = "wasm32-wasi"
```

Run tests:
```shell
CARGO_TARGET_WASM32_WASI_RUNNER="wasmtime --dir=. --" cargo test
```

Run tests without saving result images as files in `./data` directory:
```shell
CARGO_TARGET_WASM32_WASI_RUNNER="wasmtime --dir=. --env DONT_SAVE_RESULT=1 --" cargo test
```

Run a specific benchmark in `quick` mode:
```shell
CARGO_TARGET_WASM32_WASI_RUNNER="wasmtime --dir=. --" cargo bench --bench bench_resize -- --color=always --quick
```

Run benchmarks to compare with other crates for image resizing and write results into
report files, such as `./benchmarks-x86_64.md`:
```shell
CARGO_TARGET_WASM32_WASI_RUNNER="wasmtime --dir=. --env WRITE_COMPARE_RESULT=1 --" cargo bench -- --color=always Compare
```
