# Preparation

Install additional toolchains.
- Arm64:
  ```shell
  rustup target add aarch64-unknown-linux-gnu
  ```
- Wasm32:  
  ```shell
  rustup target add wasm32-wasi
  cargo install cargo-wasi
  ```
  Install [Wasmtime](https://wasmtime.dev/).

# Tests

Run tests without saving result images as files in `./data` directory:
```shell
DONT_SAVE_RESULT=1 cargo test
```

# Wasm32

Specify build target in `.cargo/config.toml` file.
```toml
[build]
target = "wasm32-wasi"
```

Template of command to run `cargo` commands with using `Wasmtime`:
```
CARGO_TARGET_WASM32_WASI_RUNNER="wasmtime --dir=." cargo wasi <any cargo command>
```

Run tests:
```shell
CARGO_TARGET_WASM32_WASI_RUNNER="wasmtime --dir=." cargo wasi test
```

Run tests without saving result images as files in `./data` directory:
```shell
CARGO_TARGET_WASM32_WASI_RUNNER="wasmtime --dir=. --env DONT_SAVE_RESULT=1" cargo wasi test
```

Run a specific benchmark in `quick` mode:
```shell
CARGO_TARGET_WASM32_WASI_RUNNER="wasmtime --dir=." cargo wasi bench --bench bench_resize -- --quick
```

Run benchmarks to compare with other image resize crates and write results into
report files, such as `./benchmarks-x86_64.md`:
```shell
CARGO_TARGET_WASM32_WASI_RUNNER="wasmtime --dir=. --env WRITE_COMPARE_RESULT=1" cargo wasi bench -- Compare
```
