
# 1. Start with a lightweight version of Python (Linux based)
FROM rust:1.92-trixie

# 2. Create a folder inside the container to hold your app
WORKDIR /matrix-pinger-rs

# create intermediate image with the libraries compiled
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && \
    echo "fn main() {" > src/main.rs && \
    echo "println!(\"if you see this, the build broke\")}" >> src/main.rs && \
    cargo build

# coppy all to the container
COPY . .
RUN touch src/main.rs

RUN cargo build

CMD ["cargo", "run"]
