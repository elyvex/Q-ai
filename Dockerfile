FROM rust:1.97.1-bookworm AS build
WORKDIR /build
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY crates ./crates
COPY xtask ./xtask
COPY migrations ./migrations
RUN cargo build --locked --release -p cli --bin qai
RUN mkdir -p /runtime/data && chown 65532:65532 /runtime/data

FROM gcr.io/distroless/cc-debian12:nonroot
WORKDIR /app
COPY --from=build /build/target/release/qai /usr/local/bin/qai
COPY --from=build /build/migrations ./migrations
COPY --from=build --chown=65532:65532 /runtime/data /data
ENV QAI_DATA_DIR=/data
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/qai"]
CMD ["version"]
