FROM rust:1.75 as builder
WORKDIR /usr/src/autokuma
COPY . .
RUN cargo install --path ./autokuma
 
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/autokuma /usr/local/bin/autokuma

CMD ["autokuma"]