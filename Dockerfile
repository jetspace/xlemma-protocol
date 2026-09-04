# syntax=docker/dockerfile:1.7
FROM rust:1.82.0-bookworm@sha256:d9c3c6f1264a547d84560e06ffd79ed7a799ce0bff0980b26cf10d29af888377 AS builder
WORKDIR /src
COPY . .
RUN cargo build --locked --release -p xlemma-api -p xlemma-cli

FROM debian:12.12-slim@sha256:d5d3f9c23164ea16f31852f95bd5959aad1c5e854332fe00f7b3a20fcc9f635c AS runtime
RUN useradd --create-home --uid 10001 xlemma \
    && mkdir -p /var/lib/xlemma \
    && chown xlemma:xlemma /var/lib/xlemma \
    && apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /src/target/release/xlemma-api /usr/local/bin/xlemma-api
COPY --from=builder /src/target/release/xlemma-cli /usr/local/bin/xlemma
USER xlemma
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --retries=3 CMD ["curl", "--fail", "--silent", "--show-error", "http://127.0.0.1:8080/health"]
ENTRYPOINT ["/usr/local/bin/xlemma-api"]
