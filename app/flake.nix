{
  description = "Hestia Flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default =
          with pkgs;
          mkShell {
            name = "hestia";
            buildInputs = [
              (rust-bin.stable.latest.default.override { extensions = [ "rust-src" ]; })
              sea-orm-cli
              gobject-introspection
              xorg.libX11
              atk
              unixtools.netstat
              # lsb-release
              xdg-utils
              sqlite
              sqlite.dev
              openssl.dev
              pkg-config
              atkmm
              at-spi2-atk
              cairo
              gdk-pixbuf
              glib
              gtk3
              harfbuzz
              librsvg
              libsoup_3
              pango
              libiconv.dev
            ];
            nativeBuildInputs = [
              pkg-config
              openssl.dev
              # darwin.libiconv.dev
            ]
            ++ lib.optionals pkgs.stdenv.isDarwin [
            ];
            NIX_SHELL = "hestia";
            shellHook = ''
              nu
            '';
          };
      }
    );
}
