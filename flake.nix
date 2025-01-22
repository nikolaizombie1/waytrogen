{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };
  outputs = { self, flake-utils, naersk, nixpkgs }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = (import nixpkgs) { inherit system; };
        naersk' = pkgs.callPackage naersk { };

      in rec {
        defaultPackage = naersk'.buildPackage {
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
            killall
          ];

          env = { OPENSSL_NO_VENDOR = 1; };

          postInstall = ''
            mkdir -p $out/share/glib-2.0/schemas && cp org.Waytrogen.Waytrogen.gschema.xml $out/share/glib-2.0/schemas/
            glib-compile-schemas $out/share/glib-2.0/schemas
            mkdir -p $out/share/locale/en/LC_MESSAGES && msgfmt locales/en/LC_MESSAGES/waytrogen.po -o waytrogen.mo && cp locales/en/LC_MESSAGES/waytrogen.mo $out/share/locale/en/LC_MESSAGES
            mkdir -p $out/share/locale/es/LC_MESSAGES && msgfmt locales/es/LC_MESSAGES/waytrogen.po -o waytrogen.mo && cp locales/es/LC_MESSAGES/waytrogen.mo $out/share/locale/es/LC_MESSAGES
            mkdir -p $out/share/applications && cp waytrogen.desktop $out/share/applications/
            mkdir -p $out/share/icons/hicolor/scalable/apps && cp README-Assets/WaytrogenLogo.svg $out/share/icons/hicolor/scalable/apps/waytrogen.svg
          '';

          meta = {
            description = "A lightning fast wallpaper setter for Wayland.";
            longDescription =
              "A GUI wallpaper setter for Wayland that is a spiritual successor for the minimalistic wallpaper changer for X11 nitrogen. Written purely in the Rust 🦀 programming language. Supports hyprpaper, swaybg, mpvpaper and swww wallpaper changers.";
            homepage = "https://github.com/nikolaizombie1/waytrogen";
          };
        };

        devShell = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustc
            cargo
            rustfmt
            clippy
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
            killall
          ];

          env = { OPENSSL_NO_VENDOR = 1; };
        };
      });
}
