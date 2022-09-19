default: test build

debug:
  cargo build

build:
  # ya toujours des infos sur thasos 🙁
  cargo +nightly build --release -Z build-std=std,panic_abort -Z build-std-features=panic_immediate_abort --target x86_64-unknown-linux-gnu

test:
  cargo test

install_target := if os_family() != "unix" {
  "/usr/local/bin"
} else {
  "not unix ?"
}
install: test build
  echo sudo cp {{justfile_directory()}}/target/release/monit-agregator {{install_target}}
