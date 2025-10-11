{ pkgs ? import <nixpkgs> {
  overlays = [
      (import (builtins.fetchTarball
        "https://github.com/oxalica/rust-overlay/archive/refs/heads/master.tar.gz"))
    ];
  }
}:

let
  rust = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
in

pkgs.mkShell {
  buildInputs = [
    # pkgs.rustup
    rust
    pkgs.qemu            # optional, if you want to emulate
    pkgs.gdb             # optional, for debugging
    pkgs.cargo-binutils  # for objdump, nm, etc.
    pkgs.llvmPackages.bintools
    pkgs.pkgsCross.aarch64-embedded.buildPackages.gcc

    # For ESP32 (or other Arduino) UART forwarding.
    # Arduino for simplicity.
    pkgs.arduino-ide
    pkgs.python3
    pkgs.picocom
  ];

  RUST_TARGET_PATH = ".";

  shellHook = ''
    echo "Welcome to the bare-metal Raspberry Pi Rust dev shell!"
    export CARGO_TARGET_AARCH64_UNKNOWN_NONE_GNU_LINKER=aarch64-none-elf-gcc
  '';
}
