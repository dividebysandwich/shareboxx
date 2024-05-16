# Get started with a build env with Rust nightly
FROM rustlang/rust:nightly-alpine as builder

RUN apk update && \
    apk add --no-cache bash curl npm libc-dev binaryen

RUN npm install -g sass

RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/latest/download/cargo-leptos-installer.sh | sh

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /src
COPY . .

RUN cargo leptos build --release -vv

#
FROM rustlang/rust:nightly-alpine AS runner
WORKDIR /app
RUN adduser -D shareboxx
RUN mkdir -p /shareboxx/files
RUN touch /shareboxx/chat.json
RUN chown shareboxx /shareboxx -R
COPY --from=builder /src/target/site /shareboxx/
COPY --from=builder /src/target/release/shareboxx /app/
COPY --from=builder /src/Cargo.toml /app/
# Set Environment variables
ENV LEPTOS_OUTPUT_NAME=shareboxx
ENV LEPTOS_SITE_ROOT=/shareboxx/site
ENV LEPTOS_SITE_PKG_DIR=pkg
ENV LEPTOS_SITE_ADDR=0.0.0.0:3000
ENV LEPTOS_RELOAD_PORT=3001
ENTRYPOINT ["/app/shareboxx"]
