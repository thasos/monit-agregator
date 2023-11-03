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

# just cargo test
test:
    cargo test

# local installation
install: test
    cargo install -f --path {{ justfile_directory() }}
