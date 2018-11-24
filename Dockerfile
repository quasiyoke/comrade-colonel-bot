FROM rust:1.30.1
WORKDIR /usr/src/comrade-colonel-bot
COPY . .
RUN cargo build --release
CMD ["./target/release/comrade-colonel-bot"]
