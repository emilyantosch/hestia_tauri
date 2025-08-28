{
  pkgs ? import <nixpkgs-unstable> { },
}:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    sea-orm-cli
    gobject-introspection
    rustc
    rustfmt
    clippy
    cargo
    openssl
    xorg.libX11
    atk
    webkitgtk_4_1
    nodejs
    deno
    unixtools.netstat
    lsb-release
    xdg-utils
    sqlite
    sqlite.dev
    openssl.dev
    pkg-config
    # npm
    nodejs
    bun
    deno
  ];
  buildInputs = with pkgs; [
    at-spi2-atk
    atkmm
    cairo
    gdk-pixbuf
    glib
    gtk3
    harfbuzz
    librsvg
    libsoup_3
    pango
    webkitgtk_4_1
    openssl
  ];
  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
