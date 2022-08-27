FROM rust:bullseye AS builder

WORKDIR /opt/monit-agregator
COPY . .
RUN apt-get update  \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y librust-openssl-sys-dev  \
 && apt-get clean  \
 && rm -rf /var/lib/apt/lists/*
RUN cargo build --release

#------------------------
FROM debian:sid
RUN groupadd -g 1000 monagr  \
 && useradd -s /bin/bash --create-home -u 1000 -g 1000 monagr

# runtime prereqs
RUN apt-get update  \
 && DEBIAN_FRONTEND=noninteractive apt-get install -y libssl3  \
 && apt-get clean  \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /opt/monit-agregator/target/release/monit-agregator /opt/
#COPY target/release/monit-agregator /opt/
COPY Settings.yaml /opt/

USER monagr
CMD ["/opt/monit-agregator", "-l", "info", "-c", "/opt/Settings.yaml"]
