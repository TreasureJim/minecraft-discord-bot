########################################################################################################################
# build stage
########################################################################################################################

FROM rust:1.90-slim AS build

RUN rustup target add x86_64-unknown-linux-musl && \
    apt update && \
    apt install -y musl-tools musl-dev adduser && \
    update-ca-certificates

COPY ./src ./src
COPY ./Cargo.lock .
COPY ./Cargo.toml .

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid 10001 \
    "minecraft-discord-bot"

RUN cargo build --target x86_64-unknown-linux-musl --release

########################################################################################################################
# image
########################################################################################################################

FROM scratch

COPY --from=build /etc/passwd /etc/passwd
COPY --from=build /etc/group /etc/group

COPY --from=build --chown=minecraft-discord-bot:minecraft-discord-bot ./target/x86_64-unknown-linux-musl/release/minecraft-discord-bot /app/minecraft-discord-bot

USER minecraft-discord-bot:minecraft-discord-bot

ENTRYPOINT ["./app/minecraft-discord-bot"]
