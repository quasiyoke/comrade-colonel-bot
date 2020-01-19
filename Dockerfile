FROM rust:1.40.0
WORKDIR /usr/src/comrade-colonel-bot
COPY . .
RUN cargo build --release
CMD ["./target/release/comrade-colonel-bot"]
