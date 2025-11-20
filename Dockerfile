########################################################################################################################
# build stage
########################################################################################################################

FROM rust:1.90-slim AS build

# RUN rustup target add x86_64-unknown-linux-musl && \
#     apt update && \
#     apt install -y musl-tools musl-dev adduser && \
#     update-ca-certificates
RUN apt update && \
    apt install -y adduser && \
    update-ca-certificates

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    "minecraft-discord-bot"

# Create a dummy project and build dependencies first
WORKDIR /app
RUN cargo init --name minecraft-discord-bot
COPY ./Cargo.lock .
COPY ./Cargo.toml .
# RUN cargo build --target x86_64-unknown-linux-musl --release
RUN cargo build --release

# Now copy the actual source code and rebuild if needed
COPY ./src ./src
# RUN cargo build --target x86_64-unknown-linux-musl --release
RUN cargo build --release

########################################################################################################################
# image
########################################################################################################################

FROM debian:stable

COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group

COPY --from=build --chown=minecraft-discord-bot:minecraft-discord-bot /app/target/release/minecraft-discord-bot /app/minecraft-discord-bot

USER minecraft-discord-bot:minecraft-discord-bot

ENTRYPOINT ["./app/minecraft-discord-bot"]
