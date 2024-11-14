ARG FEATURES=

FROM rust:1.81 AS builder
ARG FEATURES
WORKDIR /usr/src/autokuma
COPY . .
RUN cargo install --features "${FEATURES}" --path ./autokuma
 
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/autokuma /usr/local/bin/autokuma

ENV AUTOKUMA_DOCKER=1
CMD ["autokuma"]