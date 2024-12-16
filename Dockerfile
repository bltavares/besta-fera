FROM clux/muslrust:1.83.0-stable AS chef
USER root
RUN cargo install cargo-chef
# https://github.com/briansmith/ring/issues/2127
ENV CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=aarch64-linux-gnu-gcc
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target $(uname -m)-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target $(uname -m)-unknown-linux-musl --bin besta-fera \
    && mv target/$(uname -m)-unknown-linux-musl/release/besta-fera /app/target/besta-fera

FROM alpine AS runtime
RUN addgroup -S besta && adduser -S besta -G besta
COPY --from=builder /app/target/besta-fera /usr/local/bin/
USER besta
CMD ["/usr/local/bin/besta-fera"]