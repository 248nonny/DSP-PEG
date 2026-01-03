{
  description = "DSP-PEG project flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, rust-overlay, crane }:
    let
      globalVersion = "0.1.0";
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
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

      craneLib = (crane.mkLib pkgs).overrideToolchain rust;

      # Copy linker script
      src = pkgs.lib.cleanSourceWith {
        src = ./.;
        filter = path: type:
          (craneLib.filterCargoSources path type) || (pkgs.lib.hasSuffix ".ld" path);
      };

      # Cross compilation helpers
      cross_bare = pkgs.pkgsCross.aarch64-embedded;
      cross = pkgs.pkgsCross.aarch64-multiplatform;

      commonNativeInputs = [
        # rust is injected via craneLib, but pkg-config/zig are still needed
        pkgs.pkg-config
        pkgs.zig
        pkgs.cargo-zigbuild
        pkgs.cargo-show-asm
        cross.buildPackages.gcc
        cross_bare.buildPackages.gcc
      ];

      # Common arguments for both builds
      commonArgs = {
        inherit src globalVersion;
        strictDeps = true;
        nativeBuildInputs = commonNativeInputs;
      };

    in {
      devShells.${system}.default = pkgs.mkShell {
        nativeBuildInputs = [
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
          export LD_LIBRARY_PATH=/run/opengl-driver/lib/:${pkgs.lib.makeLibraryPath ([pkgs.libGL pkgs.libGLU pkgs.libxkbcommon])}
          GREEN="\[\033[0;32m\]"
          RESET="\[\033[0m\]"
          export PS1="$GREEN (DSP-dev)$RESET $PS1"
        '';
      };

      packages.${system} = rec {

        userspaceArgs = commonArgs // {
            pname = "DSP-PEG-userspace";

            # Fixes home dir permission error for cargo-zigbuild
            preBuild = ''
              export HOME=$TMPDIR
            '';

            cargoBuildCommand = "cargo zigbuild --release --package userspace --target aarch64-unknown-linux-gnu.2.36 --frozen";
            cargoExtraArgs = "--package userspace";
        };

        userspaceArtifacts = craneLib.buildDepsOnly userspaceArgs;

        userspace = craneLib.buildPackage (userspaceArgs // {
            cargoArtifacts = userspaceArtifacts;
            installPhase = ''
              mkdir -p $out/bin
              cp ./target/aarch64-unknown-linux-gnu/release/editor $out/bin/DSP-PEG-ui
            '';
        });

        baremetalArgs = commonArgs // {
            pname = "DSP-PEG-baremetal";
            
            cargoExtraArgs = "--package baremetal --target aarch64-unknown-none --frozen";

            doCheck = false;
            doDoc = false;
        };

        baremetalArtifacts = craneLib.buildDepsOnly (baremetalArgs // {
            dummyMappings = [
               ./baremetal/linker.ld
            ];
        });

        baremetal = craneLib.buildPackage (baremetalArgs // {
            cargoArtifacts = baremetalArtifacts;

            installPhase = ''
              mkdir -p $out/baremetal
              aarch64-none-elf-objcopy -O binary target/aarch64-unknown-none/release/baremetal $out/baremetal/dsp_peg_fw.bin
            '';
        });

        default = pkgs.symlinkJoin {
          name = "DSP-PEG";
          paths = [ userspace baremetal ];
        };
      };
    };
}
