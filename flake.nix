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

        lib = pkgs.lib;
        rustDefault = fx.stable.toolchain;

        hostTriple = pkgs.stdenv.hostPlatform.config;

        # Collect fenix targets (keys) as a list
        fenixTargets = builtins.attrNames fx.targets;

        supportedTargets = builtins.filter (t: lib.elem t fenixTargets) [
          # Linux
          "aarch64-unknown-linux-gnu"
          "aarch64-unknown-linux-musl"
          "arm-unknown-linux-gnueabi"
          "arm-unknown-linux-gnueabihf"
          "arm-unknown-linux-musleabi"
          "arm-unknown-linux-musleabihf"
          "armv7-unknown-linux-gnueabi"
          "armv7-unknown-linux-gnueabihf"
          "armv7-unknown-linux-musleabi"
          "armv7-unknown-linux-musleabihf"
          "x86_64-unknown-linux-gnu"
          "x86_64-unknown-linux-musl"
          # Mac
          "aarch64-apple-darwin"
          "x86_64-apple-darwin"
          # Windows
          "x86_64-pc-windows-gnu"
        ];

        # Convert fenix rust target triple -> zig target triple
        toZigTarget = triple:
          let
            parts = builtins.split "-" triple;
            arch = builtins.elemAt parts 0;
            os = builtins.elemAt parts 4;
            abi = if (builtins.length parts) > 6 then builtins.elemAt parts 6 else "";
            archFixed = if arch == "armv7" then "arm" else arch;
            # rebuild: arch-os(-abi) where abi may already contain musleabihf etc.
            base = if abi != "" then "${archFixed}-${os}-${abi}" else "${archFixed}-${os}";
          in
          base;

        # Helper to make zig wrapper scripts that inject -target when needed
        makeZigWrappers = target: pkgs.runCommand "zig-wrappers-${target}" { }
          ''
            mkdir -p $out/bin
            cat > $out/bin/zig-target <<'EOF'
            #!${pkgs.stdenv.shell}
            args=(-target ${toZigTarget target})
            skip_next=false
            for arg in "$@"; do
              if $skip_next; then
                skip_next=false
              elif [ "$arg" = "-target" ]; then
                skip_next=true
              elif [ "''${arg#--target=}" != "$arg" ]; then
                : # drop --target=*
              else
                args+=("$arg")
              fi
            done
            exec ${pkgs.zig}/bin/zig "$CMD" "''${args[@]}"
            EOF

            cat > $out/bin/zig-cc <<EOF
            #!${pkgs.stdenv.shell}
            CMD=cc source $out/bin/zig-target
            EOF
            chmod +x $out/bin/zig-cc

            cat > $out/bin/zig-cxx <<EOF
            #!${pkgs.stdenv.shell}
            CMD=c++ source $out/bin/zig-target
            EOF

            cat > $out/bin/zig-ar <<'EOF'
            #!${pkgs.stdenv.shell}
            exec ${pkgs.zig}/bin/zig ar "$@"
            EOF
            chmod +x $out/bin/zig-ar
          '';

        # Build cross shells for every fenix target except hostTripleFixed
        crossShells = builtins.map
          (target:
            let
              # try to find matching rust std in fx.targets.<target>.stable.rust-std or fx.stable.targets.<target>.rust-std
              targetAttrs = builtins.getAttr target fx.targets;
              rustStd =
                if targetAttrs != null && targetAttrs.stable != null && targetAttrs.stable.rust-std != null then
                  targetAttrs.stable.rust-std
                else
                # safe lookup: fx.targets may directly hold rust-std attr
                  builtins.getAttrOr null ("rust-std") targetAttrs;
              # combine rustc/cargo/clippy plus rust std if available
              rustPkgs =
                if rustStd != null then fx.combine [ fx.stable.rustc fx.stable.cargo fx.stable.clippy rustStd ]
                else fx.combine [ fx.stable.rustc fx.stable.cargo fx.stable.clippy ];
              zigWrappers = makeZigWrappers target;
            in
            {
              "cross-${target}" = pkgs.mkShellNoCC {
                name = "cross-${target}";
                packages = [
                  rustPkgs
                  pkgs.tailwindcss_4
                  pkgs.zig
                ];

                env =
                  let
                    # sanitize env var name from target triple for cargo variable
                    cargoVar = "CARGO_TARGET_${builtins.replaceStrings [ "-" ] [ "_" ] (lib.toUpper target)}_LINKER";
                    ccVar = "CC_${builtins.replaceStrings [ "-" ] [ "_" ] target}";
                    cxxVar = "CXX_${builtins.replaceStrings [ "-" ] [ "_" ] target}";
                    arVar = "AR_${builtins.replaceStrings [ "-" ] [ "_" ] target}";
                    rustflagsVar = "CARGO_TARGET_${builtins.replaceStrings [ "-" ] [ "_" ] (lib.toUpper target)}_RUSTFLAGS";
                  in
                  {
                    "CARGO_BUILD_TARGET" = target;
                    # point to our wrappers
                    "${cargoVar}" = "${zigWrappers}/bin/zig-cc";
                    "${ccVar}" = "${zigWrappers}/bin/zig-cc";
                    "${cxxVar}" = "${zigWrappers}/bin/zig-cxx";
                    "${arVar}" = "${zigWrappers}/bin/zig-ar";
                    # prevent rust from linking its own crt if using musl-like targets
                    "${rustflagsVar}" = "-C link-self-contained=no";
                  };

                shellHook = ''
                  echo "Cross-compilation shell for target: ${target} (via zig cc)"
                '';
              };
            })
          (builtins.filter (t: t != hostTriple && (builtins.length (lib.strings.split "-" t) > 4)) supportedTargets);

        defaultShell = pkgs.mkShell {
          packages = [
            rustDefault
            pkgs.tailwindcss_4
          ];
        };

      in
      {
        # expose all cross shells as attributes under devShells
        devShells = builtins.foldl' (acc: x: acc // x)
          {
            default = defaultShell;
          }
          crossShells;

      }
    );
}

