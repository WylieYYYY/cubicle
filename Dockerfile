#!/usr/bin/env --split-string=docker build --tag cubicle . --file
FROM rust:1.68-bullseye

RUN cargo install wasm-bindgen-cli
RUN rustup target add wasm32-unknown-unknown
