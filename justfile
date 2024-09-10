# print receipes
default:
    just -l

# build debug for dev
build:
    # ya toujours des infos sur thasos quand on fait un `strings` 🙁
    cargo build

# release using using musl target, for alpine, with size optimizations
release_musl:
    cargo +nightly build --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-musl

# release with size optimizations
release:
    # ya toujours des infos sur thasos quand on fait un `strings` 🙁
    cargo +nightly build --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-gnu

test:
    cargo test

clean:
    cargo clean

# local installation
install: test
    cargo install -f --path {{ justfile_directory() }}

podman_build:
    podman pull docker.io/rustlang/rust:nightly-alpine docker.io/alpine:3.20
    podman build -t ghcr.io/thasos/monit-agregator:latest .
# fake, use podman
docker_build:
    @just podman_build

nixshell shell='zsh':
    nix develop --command {{shell}}
