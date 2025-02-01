{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };
  outputs = { self, flake-utils, naersk, nixpkgs, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = (import nixpkgs) { inherit system overlays; };
        naersk' = pkgs.callPackage naersk { };
      in {
        packages = rec {
          waytrogen = naersk'.buildPackage {
            src = ./.;

            nativeBuildInputs = with pkgs; [
              pkg-config
              glib
              wrapGAppsHook4
              sqlite
              bash
            ];
            buildInputs = with pkgs; [
              glib
              gtk4
              ffmpeg
              sqlite
              openssl
              gsettings-desktop-schemas
            ];

            preBuild = "export OUT_PATH=$out";
            cargoBuildOptions = p: p ++ [ "--features nixos" ];

            env = { OPENSSL_NO_VENDOR = 1; };

            postInstall = ''
              install -Dm644 org.Waytrogen.Waytrogen.gschema.xml -t $out/share/gsettings-schemas/$name/glib-2.0/schemas
              glib-compile-schemas $out/share/gsettings-schemas/$name/glib-2.0/schemas
              install -Dm644 waytrogen.desktop $out/share/applications/waytrogen.desktop
              install -Dm644 README-Assets/WaytrogenLogo.svg $out/share/icons/hicolor/scalable/apps/waytrogen.svg
              while IFS= read -r lang; do
                    mkdir -p $out/share/locale/$lang/LC_MESSAGES
                    msgfmt locales/$lang/LC_MESSAGES/waytrogen.po -o $out/share/locale/$lang/LC_MESSAGES/waytrogen.mo
              done < locales/LINGUAS
            '';

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
          ];

          env = { OPENSSL_NO_VENDOR = 1; };

          LOCALE_ARCHIVE = "${pkgs.glibcLocales}/lib/locale/locale-archive";
        };
      });
}
