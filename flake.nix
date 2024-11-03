{
  description = "Solana integration for Race Protocol";

  inputs = {
    rust-overlay.url = "github:oxalica/rust-overlay";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils = { url = "github:numtide/flake-utils"; };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
      in
        {
          devShell = pkgs.mkShell {
            buildInputs = with pkgs; [
              (rust-bin.stable."1.75.0".default.override {
                extensions = [ "rust-src" ];
              })
              rust-analyzer
            ];
          };
        }
    );
}
