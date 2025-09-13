{
  pkgs ? import <nixpkgs-unstable> { },
}:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    sea-orm-cli
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
    bun
    unixtools.netstat
    lsb-release
    xdg-utils
    sqlite
    sqlite.dev
    openssl.dev
    pkg-config
  ];
  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
  RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
