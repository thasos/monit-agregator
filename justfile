default: test build

build:
    # ya toujours des infos sur thasos quand on fait un `strings` 🙁
    cargo build

release_musl: test
    RUSTFLAGS='-C target-feature=-crt-static'
    cargo +nightly build --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-musl

release:
    # ya toujours des infos sur thasos quand on fait un `strings` 🙁
    cargo +nightly build --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-gnu

test:
    cargo test

install: test
    cargo install -f --path {{ justfile_directory() }}
