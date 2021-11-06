<div align="center">
  <h1><code>aws-lambda-relay</code></h1>
  <p><strong>HTTP relay for AWS Lambda.</strong></p>
</div>

## Build

Target `x86_64-unknown-linux-gnu` for Amazon Linux 2 x86_64 or `aarch64-unknown-linux-gnu` for AWS Graviton2 processors that use ARM64.

- 64-bit GNU/Linux

  ```sh
  # alias: cargo br
  cargo build --release --target x86_64-unknown-linux-gnu
  ```

- Docker

  ```sh
  docker run \
    --rm \
    --platform linux/arm64 \
    -v "${PWD}:/usr/src/relay" \
    -w /usr/src/relay \
    rust:latest \
    cargo build --release --target x86_64-unknown-linux-gnu
  ```

  - Windows

    ```ps
    docker run `
    --rm `
    --platform linux/arm64 `
    -v C:\Users\user\directory\relay\:/usr/src/relay `
    -w /usr/src/relay `
    rust:latest `
    cargo build --release --target x86_64-unknown-linux-gnu
    ```



## Dependency explanation

- [`lambda_http`](https://lib.rs/crates/lambda_http) Library for AWS API Gateway proxy event focused AWS Lambda functions

  - [`lambda_runtime`](https://lib.rs/crates/lambda_runtime) AWS Lambda runtime

- [`reqwest`](https://lib.rs/crates/reqwest) Async HTTP Client with connection pooling

- [`tokio`](https://lib.rs/crates/tokio) Async runtime

- [`tracing`](https://lib.rs/crates/tracing) Collect event-based diagnostics

- [`tracing-subscriber`](https://lib.rs/crates/tracing-subscriber) Log trace events to the console
