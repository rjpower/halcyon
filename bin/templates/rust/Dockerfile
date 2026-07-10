# ---- build ----------------------------------------------------------------
FROM rust:1-bookworm AS build
WORKDIR /src

# Compile the dependency graph against a stub main first, so that editing our
# own source doesn't re-compile every crate we depend on.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs && cargo build --release && rm -rf src

COPY src ./src
RUN touch src/main.rs && cargo build --release --locked

# ---- runtime --------------------------------------------------------------
FROM debian:bookworm-slim
WORKDIR /app
ENV {{env_prefix}}_STATIC=/app/static \
    {{env_prefix}}_PORT={{port}}

# ca-certificates for outbound TLS; curl only so HEALTHCHECK has something to
# call with. Both are a few megabytes and save reaching for a shell.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

COPY --from=build /src/target/release/{{name}} /usr/local/bin/{{name}}
COPY static ./static

RUN useradd --system --uid 1000 app && mkdir -p /data && chown -R app:app /data /app
USER app

EXPOSE {{port}}
VOLUME ["/data"]

HEALTHCHECK --interval=30s --timeout=10s --start-period=10s --retries=3 \
    CMD curl -fsS http://127.0.0.1:{{port}}/healthz || exit 1

CMD ["{{name}}"]
