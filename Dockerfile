# select build image
FROM ekidd/rust-musl-builder:stable as builder

# create a new empty shell project
RUN USER=root cargo new --bin maxblog-api

WORKDIR ./maxblog-api

# copy over your manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# this build step will cache your dependencies
RUN cargo build --release
RUN rm src/*.rs

# copy your source tree
ADD . ./
# build for release
# RUN rm ./target/x86_64-unknown-linux-musl/release/deps/maxblog-api*
RUN cargo build --release

# our final base
FROM alpine:latest

ARG APP=/usr/src/app

EXPOSE 8000

ENV TZ=Etc/UTC \
    APP_USER=appuser

RUN addgroup -S $APP_USER \
    && adduser -S -g $APP_USER $APP_USER

RUN apk update \
    && apk add --no-cache ca-certificates tzdata \
    && rm -rf /var/cache/apk/*

# copy the build artifact from the build stage
COPY --from=builder /home/rust/src/maxblog-api/target/x86_64-unknown-linux-musl/release/maxblog-api ${APP}/maxblog-api

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER

WORKDIR ${APP}

# set the startup command to run your binary
CMD ["./maxblog-api"]

