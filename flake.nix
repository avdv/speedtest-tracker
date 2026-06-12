{
  description = "Speedtest Tracker — Rust development shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-26.05";
    flake-utils.url = "github:numtide/flake-utils";
    fenix.url = "github:nix-community/fenix";
    fenix.inputs.nixpkgs.follows = "nixpkgs";
  };

  outputs = { self, nixpkgs, flake-utils, fenix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        fx = fenix.packages.${system};

        rustDefault = fx.stable.toolchain;

        rustCross = fx.combine [
          fx.stable.rustc
          fx.stable.cargo
          fx.stable.clippy
          fx.targets.armv7-unknown-linux-musleabihf.stable.rust-std
        ];

        # Always inject -target arm-linux-musleabihf as the first argument and
        # strip any -target / --target= passed by cargo (compiler or linker
        # invocations) to avoid conflicts. This is necessary because when zig cc
        # is called as a pure linker it receives no -target flag from Rust and
        # would default to the host, failing to locate its bundled musl.
        translateTarget = ''
          args=(-target arm-linux-musleabihf)
          skip_next=false
          for arg in "$@"; do
            if $skip_next; then
              skip_next=false
            elif [ "$arg" = "-target" ]; then
              skip_next=true
            elif [[ "$arg" == --target=* ]]; then
              : # drop it
            else
              args+=("$arg")
            fi
          done
        '';

        zigCC = pkgs.writeShellScriptBin "zig-cc-armv7" ''
          ${translateTarget}
          exec ${pkgs.zig}/bin/zig cc "''${args[@]}"
        '';

        zigCXX = pkgs.writeShellScriptBin "zig-cxx-armv7" ''
          ${translateTarget}
          exec ${pkgs.zig}/bin/zig c++ "''${args[@]}"
        '';

        zigAR = pkgs.writeShellScriptBin "zig-ar-armv7" ''
          exec ${pkgs.zig}/bin/zig ar "$@"
        '';
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            rustDefault
            pkgs.tailwindcss_4
          ];
        };

        devShells.cross-armv7 = pkgs.mkShellNoCC {
          packages = [
            rustCross
            pkgs.tailwindcss_4
            pkgs.zig
          ];

          env = {
            CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER = "${zigCC}/bin/zig-cc-armv7";
            CC_armv7_unknown_linux_musleabihf = "${zigCC}/bin/zig-cc-armv7";
            CXX_armv7_unknown_linux_musleabihf = "${zigCXX}/bin/zig-cxx-armv7";
            AR_armv7_unknown_linux_musleabihf = "${zigAR}/bin/zig-ar-armv7";
            # Prevent Rust from injecting its own self-contained crt1.o / crti.o;
            # zig already provides musl startup files from its bundled libc.
            CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_RUSTFLAGS = "-C link-self-contained=no";
          };

          shellHook = ''
            echo "Cross-compilation shell for armv7-unknown-linux-musleabihf (via zig cc)"
          '';
        };
      }
    );
}
