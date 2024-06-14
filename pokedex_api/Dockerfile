FROM rust:alpine3.20

# add support for macOS M1 chip
RUN apk add --no-cache musl-dev
# add support for libressl
RUN apk add libressl-dev

# Set the working directory
WORKDIR /code/pokedex_api