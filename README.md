<div align="center">
  <h1><code>aws-lambda-relay</code></h1>
  <p><strong>HTTP relay for AWS Lambda.</strong></p>
</div>

## Build

The toolchain target `x86_64-unknown-linux-gnu` is required to build for AWS x86_64 processors.

```sh
rustup target add x86_64-unknown-linux-gnu
```

### Building on `x86_64-*-linux-gnu`

```sh
cargo build --release --target x86_64-unknown-linux-gnu
```

### Building with [`cross`](https://github.com/rust-embedded/cross) and Docker

  1. Install `cross`

     ```sh
     cargo install cross --version 0.2.1
     ```

  2. Build (uses [`rustembedded/cross:x86_64-unknown-linux-gnu-0.2.1`](https://hub.docker.com/layers/rustembedded/cross/x86_64-unknown-linux-gnu-0.2.1/images/sha256-9f368a726a8ba08559451cd64160f7d2b47f6180ad024a46e31d29cc85dd81ff) Docker image with `libssl-dev` installed)

     ```sh
     cross build --release --target x86_64-unknown-linux-gnu
     ```

### Building with Docker

```sh
docker run \
  --rm \
  --platform linux/amd64 \
  --user "$(id -u):$(id -g)" \
  --volume "${PWD}:/usr/src/relay" \
  --workdir '/usr/src/relay' \
  rust:latest \
  cargo build --release --target x86_64-unknown-linux-gnu
```

## Dependency explanation

- [`bytes`](https://lib.rs/crates/bytes) Bytes container used by `reqwest::Response::bytes`

- [`lambda_http`](https://lib.rs/crates/lambda_http) Library for AWS API Gateway proxy event focused AWS Lambda functions

  - [`lambda_runtime`](https://lib.rs/crates/lambda_runtime) AWS Lambda runtime

- [`reqwest`](https://lib.rs/crates/reqwest) Async HTTP Client

- [`tokio`](https://lib.rs/crates/tokio) Async runtime

- [`tracing`](https://lib.rs/crates/tracing) Collect event-based diagnostics

- [`tracing-subscriber`](https://lib.rs/crates/tracing-subscriber) Log trace events to the console
