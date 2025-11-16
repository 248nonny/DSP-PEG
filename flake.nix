{
  description = "Rust 1.86.0 dev shell with components/targets";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let

      globalVersion = "0.1.0";

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

      aarch64Gcc = pkgs.pkgsCross.aarch64-multiplatform.buildPackages.gcc;

      rpiLinker = pkgs.writeShellScriptBin "aarch64-rpi-gcc" ''
        #!/usr/bin/env bash
        set -euo pipefail
        
        exec ${aarch64Gcc}/bin/aarch64-unknown-linux-gnu-gcc \
          --sysroot="$SYSROOT" \
          -Wl,-rpath-link,"$SYSROOT/lib/aarch64-linux-gnu":"$SYSROOT/usr/lib/aarch64-linux-gnu" \
          -Wl,--dynamic-linker=/lib/ld-linux-aarch64.so.1 \
          "$@"
      '';
        
      commonNativeInputs = [
        rust
        pkgs.pkg-config
        # pkgs.llvmPackages.lld
        # pkgs.llvmPackages.bintools
        pkgs.pkgsCross.aarch64-multiplatform.buildPackages.gcc
        pkgs.pkgsCross.aarch64-embedded.buildPackages.gcc
        rpiLinker
      ];

    in {
      devShells.x86_64-linux.default = pkgs.mkShell {
        nativeBuildInputs = commonNativeInputs;

        shellHook = ''
          export SYSROOT="$src/sysroot/"
          
          # Green color code
          GREEN="\[\033[0;32m\]"
          RESET="\[\033[0m\]"
          export PS1="$GREEN (DSP-dev)$RESET $PS1"
        '';
      };

      packages.x86_64-linux = {
        userspace = pkgs.stdenv.mkDerivation {
          pname = "DSP-PEG-userspace";
          version = globalVersion;
          dontFixup = true;

          src = ./.;

          nativeBuildInputs = commonNativeInputs;

          configurePhase = "true";

          buildPhase = ''
            export SYSROOT="$src/sysroot/"
            cargo build --release --package userspace --target aarch64-unknown-linux-gnu
          '';

          installPhase = ''
            mkdir -p $out/bin
            cp target/aarch64-unknown-linux-gnu/release/userspace $out/bin/DSP-PEG-ui
          '';
        };

        baremetal = pkgs.stdenv.mkDerivation {
          pname = "DSP-PEG-baremetal";
          version = globalVersion;
          dontFixup = true;

          src = ./.;

          nativeBuildInputs = commonNativeInputs;

          configurePhase = "true";

          buildPhase = ''
            cargo build --release --package baremetal --target aarch64-unknown-none
          '';

          installPhase = ''
            mkdir -p $out/baremetal
          aarch64-none-elf-objcopy -O binary target/aarch64-unknown-none/release/baremetal $out/baremetal/dsp_peg_fw.bin
          '';
        };

        
      };

      packages.x86_64-linux.default = pkgs.symlinkJoin {
        name = "DSP-PEG";

        paths = [
          self.packages.x86_64-linux.userspace
          self.packages.x86_64-linux.baremetal
          # self.packages.x86_64-linux.kernel
        ];
      };
    };
}
