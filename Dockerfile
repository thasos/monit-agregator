FROM rust:alpine3.16 AS builder


WORKDIR /opt/monit-agregator
COPY . .
RUN cargo build --release


FROM alpine:3.16

RUN groupadd -g 1000 monagr  \
 && useradd -s /bin/bash --create-home -u 1000 -g 1000 monagr

COPY --from=builder /opt/monit-agregator/target/release/monit-agregator /opt/

USER monagr
CMD ["/opt/monit-agregator", "-l", "info"]
