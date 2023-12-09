FROM rust:1.71.1-alpine3.18

RUN apk --update-cache add curl-dev pkgconfig musl-dev npm openssl-dev zip zlib-dev
RUN cargo install wasm-bindgen-cli@0.2.87
RUN rustup component add clippy rustfmt
RUN rustup target add wasm32-unknown-unknown

WORKDIR /packages/node
RUN npm init --yes && npm install eslint eslint-config-google

WORKDIR /packages/cargo
COPY cubicle/Cargo.toml ./Cargo.toml
RUN sed -i 's/^\(name = "\)cubicle"/\1cache"/' ./Cargo.toml &&\
    sed -i 's/^\(crate-type = \["\)cdylib"\]/\1lib"]/' ./Cargo.toml
RUN mkdir ./src && echo 'fn main() {}' > ./src/lib.rs
RUN CARGO_HOME=. cargo build
RUN CARGO_HOME=. cargo build --target wasm32-unknown-unknown
