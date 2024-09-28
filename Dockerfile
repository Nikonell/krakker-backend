# Stage 1: Prepare the build environment using cargo-chef
FROM rust:1.81 as chef
WORKDIR /usr/src/krakker-backend
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Cache dependencies using cargo-chef
FROM rust:1.81 as cacher
WORKDIR /usr/src/krakker-backend
COPY --from=chef /usr/local/cargo/bin/cargo-chef /usr/local/cargo/bin/cargo-chef
COPY --from=chef /usr/src/krakker-backend/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 3: Build the project
FROM rust:1.81 as builder
WORKDIR /usr/src/krakker-backend
COPY . .
COPY --from=cacher /usr/src/krakker-backend/target target
RUN cargo build --release

# Stage 4: Create the final image
FROM debian:bookworm
RUN apt-get update && apt-get install -y libssl3 openssl && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/src/krakker-backend
COPY --from=builder /usr/src/krakker-backend/target/release/krakker-backend .
ENV LD_LIBRARY_PATH=/usr/lib:/usr/local/lib
EXPOSE 1488
CMD ["./krakker-backend"]
