
# 1. Start with a lightweight version of Python (Linux based)
FROM rust:1.92-trixie

# 2. Create a folder inside the container to hold your app
WORKDIR /matrix-pinger-rs

# coppy all to the container
COPY . .

# 6. The command to run your bot when the container starts
CMD ["cargo", "run"]
