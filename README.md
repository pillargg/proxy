<div align="center">
  <h1><code>aws-lambda-relay</code></h1>
  <p><strong>HTTP relay for AWS Lambda.</strong></p>
</div>

## Build

### Building on `x86_64-*-linux-gnu`

```sh
cargo build --release
```

### Building with Docker

1. Add toolchain target `x86_64-unknown-linux-gnu`.

   ```sh
   rustup target add x86_64-unknown-linux-gnu
   ```

2. Build using the official Rust image.

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
