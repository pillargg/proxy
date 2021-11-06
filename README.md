<div align="center">
  <h1><code>aws-lambda-relay</code></h1>
  <p><strong>HTTP relay for AWS Lambda.</strong></p>
</div>

## Build

Target `x86_64-unknown-linux-gnu` for Amazon Linux 2 x86_64 or `aarch64-unknown-linux-gnu` for AWS Graviton2 processors (ARM64).

- Build on x86_64 GNU/Linux (`glibc`)

  ```sh
  cargo build --release --target x86_64-unknown-linux-gnu
  ```

- Build on other OS with Docker

  - Install [`cross`](https://github.com/rust-embedded/cross)

    ```sh
    cargo install cross --version 0.2.1
    ```

  - Build (uses [`tedbyron/relay:0.2.1`](https://hub.docker.com/repository/docker/tedbyron/relay) Docker image which is [`rustembedded/cross:x86_64-unknown-linux-gnu-0.2.1`](https://hub.docker.com/layers/rustembedded/cross/x86_64-unknown-linux-gnu-0.2.1/images/sha256-9f368a726a8ba08559451cd64160f7d2b47f6180ad024a46e31d29cc85dd81ff) built with `libssl-dev`)

    ```sh
    cross build --release --target x86_64-unknown-linux-gnu
    ```

## Dependency explanation

- [`lambda_http`](https://lib.rs/crates/lambda_http) Library for AWS API Gateway proxy event focused AWS Lambda functions

  - [`lambda_runtime`](https://lib.rs/crates/lambda_runtime) AWS Lambda runtime

- [`reqwest`](https://lib.rs/crates/reqwest) Async HTTP Client with connection pooling

- [`tokio`](https://lib.rs/crates/tokio) Async runtime

- [`tracing`](https://lib.rs/crates/tracing) Collect event-based diagnostics

- [`tracing-subscriber`](https://lib.rs/crates/tracing-subscriber) Log trace events to the console
