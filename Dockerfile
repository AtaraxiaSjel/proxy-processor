FROM rust:1.91-alpine as builder
WORKDIR /usr/src/proxy-processor
COPY . .
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static
RUN cargo install --path proxy-filter-cli

FROM alpine:3.22
COPY --from=builder /usr/local/cargo/bin/proxy-filter-cli /bin/proxy-filter-cli
