FROM docker.io/library/rust:alpine3.18 AS builder

WORKDIR /opt/monit-agregator
COPY . .

# hadolint ignore=DL3018
RUN apk add --no-cache pkgconfig openssl-dev libc-dev just musl clang perl make upx \
 && rustup toolchain install nightly \
 && rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-musl
# hadolint ignore=DL3059
RUN just release_musl \
 && upx target/x86_64-unknown-linux-musl/release/monit-agregator

#--------------------------------
FROM alpine:3.18
RUN addgroup -S monagr \
 && adduser -S monagr -G monagr
# hadolint ignore=DL3018
RUN apk add --no-cache libssl3 # runtime prereqs

COPY --from=builder /opt/monit-agregator/target/x86_64-unknown-linux-musl/release/monit-agregator /opt/
COPY Settings.yaml /opt/

USER monagr
CMD ["/opt/monit-agregator", "-l", "info", "-c", "/opt/Settings.yaml"]
# still segfault 🙁
