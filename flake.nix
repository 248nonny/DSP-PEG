{
  description = "Rust 1.86.0 dev shell with components/targets";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      pkgs = import nixpkgs {
        system = "x86_64-linux";
        overlays = [ rust-overlay.overlays.default ];
      };
      rust = pkgs.rust-bin.stable."1.91.0".default.override {
        extensions = [ "rust-src" "rustfmt-preview" "clippy-preview" "llvm-tools-preview" ];
        targets = [
          "aarch64-unknown-none"
          "aarch64-unknown-linux-gnu"
          "x86_64-unknown-linux-gnu"
        ];
      };
    in {
      devShells.x86_64-linux.default = pkgs.mkShell {
        nativeBuildInputs = [
          rust
          pkgs.pkg-config

          # pkgs.llvmPackages.lld
          # pkgs.llvmPackages.bintools
          pkgs.pkgsCross.aarch64-embedded.buildPackages.gcc
          pkgs.pkgsCross.aarch64-multiplatform.buildPackages.gcc
        ];

        # buildInputs = [
          # pkgs.llvmPackages.bintools
        # ];

        shellHook = ''
          # Green color code
          GREEN="\[\033[0;32m\]"
          RESET="\[\033[0m\]"
          export PS1="$GREEN (DSP-dev)$RESET $PS1"
        '';
      };

      packages.x86_64-linux.default = pkgs.stdenv.mkDerivation {
        pname = "DSP-PEG";
        version = "0.1.0";

        src = ./.;

        nativeBuildInputs = [
          rust
          pkgs.pkg-config

          # pkgs.llvmPackages.lld
          # pkgs.llvmPackages.bintools
          pkgs.pkgsCross.aarch64-embedded.buildPackages.gcc
          pkgs.pkgsCross.aarch64-multiplatform.buildPackages.gcc
        ];

        configurePhase = "true";

        buildPhase = ''
          $src/scripts/build_all.sh
        '';

        installPhase = ''
          set -euo pipefail

          mkdir -p $out/bin
          mkdir -p $out/baremetal
          mkdir -p $out/kernel
          
          aarch64-none-elf-objcopy -O binary baremetal/target/aarch64-unknown-none/release/bare_metal_pi_zero $out/baremetal/dsp_peg_fw.bin
          cp target/aarch64-unknown-linux-gnu/release/userspace $out/bin
        '';
        
      };
    };
}
