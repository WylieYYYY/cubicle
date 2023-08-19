FROM rust:1.71.1-alpine3.18

RUN apk --update-cache add curl-dev pkgconfig musl-dev npm openssl-dev zip zlib-dev
RUN cargo install wasm-bindgen-cli
RUN rustup component add clippy rustfmt
RUN rustup target add wasm32-unknown-unknown
