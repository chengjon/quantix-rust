# P0.15b: Multi-stage build for quantix-openstock-import.
# Stage 1: musl builder (static binary, no glibc dependency).
FROM rust:1.83-alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /build
COPY . .

RUN cargo build --release --target x86_64-unknown-linux-musl --bin quantix

# Stage 2: minimal runtime.
FROM alpine:3.19

RUN apk add --no-cache ca-certificates tzdata

COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/quantix \
                    /usr/local/bin/quantix
RUN chmod +x /usr/local/bin/quantix

ENV TZ=Asia/Shanghai

ENTRYPOINT ["/usr/local/bin/quantix"]
