FROM rust:1.73 as builder
WORKDIR /app
RUN apt-get update && apt-get install -y libclang-dev llvm-dev
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libclang1 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/your-app-name /usr/local/bin/
CMD ["your-app-name"]
