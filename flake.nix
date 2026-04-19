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
        craneLib = crane.mkLib pkgs;

        src = craneLib.cleanCargoSource ./.;

        commonArgs = {
          inherit src;
          strictDeps = true;
          nativeBuildInputs = with pkgs; [
            pkg-config
            glib
            wrapGAppsHook4
          ];
          buildInputs = with pkgs; [
            glib
            gtk4
            ffmpeg
            sqlite
            openssl
            gsettings-desktop-schemas
            rust-bin.nightly.latest.default
            libX11
            libXcursor
            libXrandr
            libXi
            libxcb
            libxkbcommon
            vulkan-loader
            wayland
          ];
          env = { OPENSSL_NO_VENDOR = 1; };
        };


        # Layer 1: deps only — rebuilt only when Cargo.lock changes
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        # Layer 2: compile the binary
        waytrogen-bin = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          # cargoExtraArgs = "--features nixos";
          preBuild = "export OUT_PATH=$out";
        });

        # Layer 3: Meson handles everything else (i18n, schemas, icons, desktop file)
        waytrogen_iced = pkgs.stdenv.mkDerivation {
          pname = "waytrogen_iced";
          version = "0.9.3";
          src = ./.;


          nativeBuildInputs = with pkgs; [
            meson
            ninja
            pkg-config
            desktop-file-utils
            wrapGAppsHook4
            glib
            gettext          # for i18n / po subdir
            sqlite
            bash
            rustc
            gtk4
          ];

          buildInputs = with pkgs; [
            glib
            gtk4
            ffmpeg
            sqlite
            openssl
            gsettings-desktop-schemas
          ];

          mesonFlags = [
            "-Dcargo_features=nixos"
            # Point meson at the pre-built binary from layer 2
            "-Dprecompiled_binary=${waytrogen-bin}/bin/waytrogen_iced"
          ];

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
          inherit waytrogen_iced;
          inherit waytrogen-bin;
          default = waytrogen_iced;
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
          nativeBuildInputs = with pkgs; [
            glibcLocales
            pkg-config
            glib
            wrapGAppsHook4
            bash
            cargo-udeps
            meson
            ninja
            desktop-file-utils
            gettext
          ];
          buildInputs = with pkgs; [
            glib
            gtk4
            ffmpeg
            sqlite
            openssl
            gsettings-desktop-schemas
            killall
            socat
            cargo
            clippy
            rust-analyzer
            sqlite
            pkg-config
            rust-bin.nightly.latest.default
            libX11
            libXcursor
            libXrandr
            libXi
            libxcb
            libxkbcommon
            vulkan-loader
            wayland
            cargo
          ];
          env = { OPENSSL_NO_VENDOR = 1; };
        };
      });
}
