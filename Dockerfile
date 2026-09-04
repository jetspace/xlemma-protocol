# syntax=docker/dockerfile:1.7
FROM rust:1.82-bookworm AS builder
WORKDIR /src
COPY . .
RUN cargo build --release -p xlemma-api -p xlemma-cli

FROM debian:bookworm-slim AS runtime
RUN useradd --create-home --uid 10001 xlemma \
    && apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/xlemma-api /usr/local/bin/xlemma-api
COPY --from=builder /src/target/release/xlemma-cli /usr/local/bin/xlemma
USER xlemma
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/xlemma-api"]
