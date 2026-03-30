FROM rust:1.85-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && cargo build --release && rm -rf src
COPY src/ src/
COPY templates/ templates/
RUN touch src/main.rs && cargo build --release

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
RUN useradd -r -s /bin/false vega
COPY --from=builder /app/target/release/vega /usr/local/bin/vega
COPY --from=builder /app/templates /opt/vega/templates
WORKDIR /opt/vega
USER vega
EXPOSE 3000
CMD ["vega"]
