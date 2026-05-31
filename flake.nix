{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, flake-utils, nixpkgs, rust-overlay, crane }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = (import nixpkgs) { inherit system overlays; };
        lib = nixpkgs.lib;
        craneLib = crane.mkLib pkgs;

        src = lib.cleanSourceWith {
          src = craneLib.path ./.;
          filter = path: type:
            (lib.hasInfix "/locales/" path) ||
            (craneLib.filterCargoSources path type);
        };

        commonArgs = {
          inherit src;
          strictDeps = true;
          buildInputs = with pkgs; [
            sqlite
            openssl
          ];
          env = { OPENSSL_NO_VENDOR = 1; };
        };


        # Layer 1: deps only — rebuilt only when Cargo.lock changes
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Layer 2: compile the binary
        waytrogen-bin = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

        # Layer 3: Meson handles everything else (i18n, schemas, icons, desktop file)
        waytrogen = pkgs.stdenv.mkDerivation {
          pname = "waytrogen";
          version = "0.9.8";
          src = ./.;


          nativeBuildInputs = with pkgs; [
            meson
            ninja
            pkg-config
            desktop-file-utils
            wrapGAppsHook4
            cargo-udeps
            rustc
            sqlite
          ];

          buildInputs = with pkgs; [
            ffmpeg
            dbus
            libX11
            libXcursor
            libXrandr
            libXi
            libxcb
            libxkbcommon
            vulkan-loader
            wayland
            xdg-utils
            xdg-desktop-portal
          ];

          mesonFlags = [
            "-Dprecompiled_binary=${waytrogen-bin}/bin/waytrogen"
          ];

          preFixup = ''
            gappsWrapperArgs+=(
              --suffix LD_LIBRARY_PATH : ${pkgs.lib.makeLibraryPath (with pkgs; [
                wayland
                libxkbcommon
                vulkan-loader
                libGL
                dbus
              ])}
            )
          '';

          env = { OPENSSL_NO_VENDOR = 1; };

          meta = {
            description = "A lightning fast wallpaper setter for Wayland.";
            longDescription = ''
              A GUI wallpaper setter for Wayland that is a spiritual successor
              for the minimalistic wallpaper changer for X11 nitrogen. Written
              purely in the Rust language. Supports hyprpaper, swaybg,
              mpvpaper and awww wallpaper changers.
            '';
            homepage = "https://github.com/nikolaizombie1/waytrogen";
          };
        };
      in {
        packages = {
          inherit waytrogen;
          inherit waytrogen-bin;
          default = waytrogen;
        };

        checks = {
          inherit waytrogen-bin;
          waytrogen-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });
          waytrogen-fmt = craneLib.cargoFmt { inherit src; };
        };

        devShells.default = pkgs.mkShell rec {
          shellHook = ''
            export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
            export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath buildInputs}"
          '';
          buildInputs = with pkgs; [
            clippy
            rust-analyzer
            cargo
            rustc
          ];
          env = { OPENSSL_NO_VENDOR = 1; };
        };
      });
}
