{ pkgs }:
pkgs.mkShell {
  nativeBuiltInputs = (pkgs.lib.optionals pkgs.stdenv.isDarwin [
    pkgs.darwin.apple_sdk.frameworks.AppKit
    pkgs.darwin.apple_sdk.frameworks.IOKit
    pkgs.darwin.apple_sdk.frameworks.Foundation
  ]);
  buildInputs = with pkgs;
    (pkgs.lib.optionals pkgs.stdenv.isLinux ([ libudev ])) ++ [
      rustup
      cargo-deps
      gh

      # sdk
      nodejs
      yarn
      python3

      pkgconfig
      openssl
      jq
      gnused
      libiconv

      anchor
      spl-token-cli
      solana-cli
    ];
  shellHook = ''
    export PATH=$PATH:$HOME/.cargo/bin
  '';
}
