{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [ rustc cargo gcc rustfmt clippy vscode-extensions.vadimcn.vscode-lldb vscode-extensions.arrterian.nix-env-selector vscode-extensions.dracula-theme.theme-dracula vscode-extensions.vscodevim.vim nix nixfmt-rfc-style gtk4 pkg-config wrapGAppsHook4 gsettings-desktop-schemas glib dconf ];

  # Certain Rust tools won't work without this
  # This can also be fixed by using oxalica/rust-overlay and specifying the rust-src extension
  # See https://discourse.nixos.org/t/rust-src-not-found-and-other-misadventures-of-developing-rust-on-nixos/11570/3?u=samuela. for more details.
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
