FROM rust:1.75-slim-bullseye as builder

RUN apt update \
    && apt install lld clang postgresql-client -y \
    && rm -rf /var/lib/apt/lists/*;

WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bullseye-slim AS runtime
WORKDIR /app
# Install OpenSSL - it is dynamically linked by some of our dependencies
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    # Clean up
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/zero2prod zero2prod
COPY configuration* .
ENV APP_ENVIRONMENT="production"
ENTRYPOINT [ "./zero2prod" ]
