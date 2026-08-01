FROM rust:1.88-bullseye as builder

RUN apt update && apt upgrade -y && apt install clang -y

# Make an /app dir, which everything will eventually live in
RUN mkdir -p /app
WORKDIR /app
COPY . .

ENV SQLX_OFFLINE=true

# Build the app
RUN cargo build --release

FROM rust:1.88-bullseye as runner

# Copy the server binary to the /app directory
COPY --from=builder /app/target/release/omibot /app/
COPY --from=builder /app/Cargo.toml /app/Cargo.toml

WORKDIR /app

CMD ["/app/omibot"]
