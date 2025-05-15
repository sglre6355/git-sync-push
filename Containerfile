FROM rust AS build-env
WORKDIR /app
RUN --mount=type=bind,source=src,target=src \
    --mount=type=bind,source=Cargo.toml,target=Cargo.toml \
    --mount=type=bind,source=Cargo.lock,target=Cargo.lock \
    --mount=type=cache,target=/app/target/ \
    --mount=type=cache,target=/usr/local/cargo/registry/ \
    cargo build --locked --release && \
    cp /app/target/release/git-sync-push /app/git-sync-push

FROM gcr.io/distroless/cc
COPY --from=build-env /app/git-sync-push /
CMD ["./git-sync-push"]
