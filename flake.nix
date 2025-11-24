{
  description = "DSP-PEG project flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let

      globalVersion = "0.1.0";

      pkgs = import nixpkgs {
        system = "x86_64-linux";
        overlays = [ rust-overlay.overlays.default ];
      };

      # Set up rust toolchain with pinned version,
      # as well as dev tools and the needed targets.
      rust = pkgs.rust-bin.stable."1.91.0".default.override {
        extensions = [ "rust-src" "rustfmt-preview" "clippy-preview" "llvm-tools-preview" ];
        targets = [
          "aarch64-unknown-none"
          "aarch64-unknown-linux-gnu"
          "x86_64-unknown-linux-gnu"
        ];
      };


      # Specify a rust platform for building rust packages later;
      # use the pinned toolchain we just created above.
      rustPlatform = pkgs.makeRustPlatform {
        cargo = rust;
        rustc = rust;
      };

      # So we don't have to type this out over and over.
      cross_bare = pkgs.pkgsCross.aarch64-embedded;
      cross = pkgs.pkgsCross.aarch64-multiplatform;
        
      # nativeBuildInputs to be used everywhere.
      # Mainly cross compiling stuff and the pinned rust toolchain.
      commonNativeInputs = [
        rust
        pkgs.pkg-config
        pkgs.zig
        pkgs.cargo-zigbuild
        # pkgs.llvmPackages.lld
        # pkgs.llvmPackages.bintools
        cross.buildPackages.gcc
        cross_bare.buildPackages.gcc
        # rpiLinker
      ];

    in {
      # Specify dev shell for local building and testing.
      devShells.x86_64-linux.default = pkgs.mkShell {
        nativeBuildInputs = [

          # Dependencies for native eframe rust program;
          # the idea is that you can run the GUI natively to quickly
          # test how it looks.
          pkgs.xorg.libxcb
          pkgs.xorg.libXcursor
          pkgs.xorg.libXrandr
          pkgs.xorg.libXi
          pkgs.libxkbcommon
          pkgs.xorg.libX11
          pkgs.libGL
          pkgs.libGLU
        ] ++ commonNativeInputs;

        buildInputs = [
          pkgs.xorg.libX11
          pkgs.libxkbcommon
        ];

        shellHook = ''
          # This is needed to run eframe / egui rust programs; i.e. for testing the
          # userspace app natively.
          export LD_LIBRARY_PATH=/run/opengl-driver/lib/:${pkgs.lib.makeLibraryPath ([pkgs.libGL pkgs.libGLU pkgs.libxkbcommon])}

          
          # Green color code; Prefix PS1 to show that we are in the dev shell.
          GREEN="\[\033[0;32m\]"
          RESET="\[\033[0m\]"
          export PS1="$GREEN (DSP-dev)$RESET $PS1"
        '';
      };

      # Specify packages to build.
      # For now, only the rust packages are being built here,
      # the kernel module (which will be kept small by design)
      # will be compiled on the rpi (FOR NOW) to avoid cross
      # compilation headaches.
      packages.x86_64-linux = {
        # The userspace rpi gui app, cross compiled.
        userspace = rustPlatform.buildRustPackage {
          pname = "DSP-PEG-userspace";
          version = globalVersion;
          dontFixup = true;

          auditable = false;

          src = ./.;

          nativeBuildInputs = commonNativeInputs;

          # Don't add rpath since the nix rpath
          # won't work on the rpi.
          NIX_NO_SELF_RPATH = "1";
          NIX_DONT_SET_RPATH = "1";
          
          
          buildAndTestSubdir = "userspace";

          cargoLock.lockFile = ./Cargo.lock;
          cargoHash = pkgs.lib.fakeHash;

          

          cargoBuildFlags = [
            "--package" "userspace"
            "--target" "aarch64-unknown-linux-gnu"
          ];

          buildPhase = ''
            runHook preBuild

            export HOME=$PWD
            
            # Zigbuild needed to match rpi glibc version.
            # (tried using old nixpkgs, but other things broke)
            cargo zigbuild \
              --release \
              --package userspace \
              --target aarch64-unknown-linux-gnu.2.36 \
              --frozen

            runHook postBuild
          '';

          # Copy the binary to the output directory.
          installPhase = ''
            mkdir -p $out/bin
            cp target/aarch64-unknown-linux-gnu/release/userspace $out/bin/DSP-PEG-ui
          '';
        };

        # The bare-metal DSP payload, cross-compiled.
        baremetal = rustPlatform.buildRustPackage {
          pname = "DSP-PEG-baremetal";
          version = globalVersion;
          dontFixup = true;

          src = ./.;

          nativeBuildInputs = commonNativeInputs;

          buildAndTestSubdir = "baremetal";

          cargoLock.lockFile = ./Cargo.lock;
          cargoHash = pkgs.lib.fakeHash;

          CARGO_BUILD_TARGET = "aarch64-unknown-none";

          cargoBuildFlags = [
            "--package" "baremetal"
            "--target" "aarch64-unknown-none"
          ];

          doCheck = false;
          doDoc = false;

          buildPhase = ''
            runHook preBuild


            cargo build \
              --release \
              --package baremetal \
              --target aarch64-unknown-none \
              --frozen

            runHook postBuild
          '';
          
          # Extract raw binary file and place it in the output dir;
          # this will get loaded straight into memory.
          installPhase = ''
            mkdir -p $out/baremetal
            aarch64-none-elf-objcopy -O binary target/aarch64-unknown-none/release/baremetal $out/baremetal/dsp_peg_fw.bin
          '';
        };

        
      };

      # Default package just merges userspace and baremetal.
      packages.x86_64-linux.default = pkgs.symlinkJoin {
        name = "DSP-PEG";

        paths = [
          self.packages.x86_64-linux.userspace
          self.packages.x86_64-linux.baremetal
        ];
      };
    };
}
