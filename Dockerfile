FROM rust:1.95-slim-trixie
ARG geckodriver_version=0.33.0

RUN apt-get update && apt-get install --yes pkg-config npm libssl-dev zip wget firefox-esr

WORKDIR /packages/node
RUN npm init --yes && npm install \
    @eslint/js@^9.39.4 \
    eslint@^9.39.4 \
    eslint-config-google@git+https://github.com/google/eslint-config-google.git#30e07afe1cc4b105f9228b5c6300be79279503e1 \
    globals

WORKDIR /packages/cargo
RUN cargo install wasm-bindgen-cli@0.2.118
RUN rustup component add clippy rustfmt
RUN rustup target add wasm32-unknown-unknown
COPY cubicle/Cargo.toml ./Cargo.toml
RUN sed -i 's/^\(name = "\)cubicle"/\1cache"/' ./Cargo.toml &&\
    sed -i 's/^\(crate-type = \["\)cdylib"\]/\1lib"]/' ./Cargo.toml
RUN mkdir ./src && echo 'fn main() {}' > ./src/lib.rs
RUN CARGO_HOME=. cargo build --target wasm32-unknown-unknown

WORKDIR /packages/geckodriver
RUN wget https://github.com/mozilla/geckodriver/releases/download/\
v${geckodriver_version}/geckodriver-v${geckodriver_version}-linux64.tar.gz
RUN tar xvzf geckodriver-v${geckodriver_version}-linux64.tar.gz
