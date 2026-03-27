{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };
  outputs = { self, flake-utils, nixpkgs, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = (import nixpkgs) { inherit system overlays; };
      in {
        packages = rec {
          waytrogen = pkgs.rustPlatform.buildRustPackage {
            pname = "waytrogen";
            version = "0.9.0";
            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
              glib
              wrapGAppsHook4
              sqlite
              bash
              meson
              ninja
              desktop-file-utils
            ];
            buildInputs = with pkgs; [
              glib
              gtk4
              ffmpeg
              sqlite
              openssl
              gsettings-desktop-schemas
              socat
              rust-analyzer
            ];

            preBuild = "export OUT_PATH=$out";

            buildPhase = ''
            runHook preBuild
            meson setup --prefix $out -Dcargo_features=nixos build
            meson compile -C build
            runHook postBuild
            '';

            installPhase = ''
            runHook preInstall
            meson install -C build
            runHook postInstall
            '';
            
            env = { OPENSSL_NO_VENDOR = 1; };

            meta = {
              description = "A lightning fast wallpaper setter for Wayland.";
              longDescription =
                "A GUI wallpaper setter for Wayland that is a spiritual successor for the minimalistic wallpaper changer for X11 nitrogen. Written purely in the Rust 🦀 programming language. Supports hyprpaper, swaybg, mpvpaper and swww wallpaper changers.";
              homepage = "https://github.com/nikolaizombie1/waytrogen";
            };
          };
          default = waytrogen;
        };

        devShells.default = pkgs.mkShell {
              shellHook = ''
                export RUST_SRC_PATH=${pkgs.rustPlatform.rustLibSrc}
              '';
          nativeBuildInputs = with pkgs; [
            glibcLocales
            pkg-config
            glib
            wrapGAppsHook4
            sqlite
            bash
            rust-bin.nightly.latest.default
            cargo-udeps
          ];
          buildInputs = with pkgs; [
            glib
            gtk4
            ffmpeg
            sqlite
            openssl
            gsettings-desktop-schemas
            killall
            meson
            ninja
            socat
            cargo
            gettext
            clippy
            sqlite
          ];

          env = { OPENSSL_NO_VENDOR = 1; };
        };
      });
}
