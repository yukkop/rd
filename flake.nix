{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile
          ./rust-toolchain.toml;
        nativeBuildInputs = with pkgs; [ rustToolchain pkg-config ];
        buildInputs = with pkgs; [
          openssl
          glibc
          ffmpeg
          ffmpeg.lib
	  pkg-config
	  SDL2
	  SDL2.dev
          stdenv.cc.cc.lib
          libclang
	  alsa-lib

          # cargo utility
	  wasm-pack
	  trunk
        ];
      in
      with pkgs; {
        formatter = alejandra;
        devShells.default = mkShell.override { stdenv = clangStdenv; } {
          LIBCLANG_PATH = "${pkgs.libclang.lib}/lib";
          BINDGEN_EXTRA_CLANG_ARGS = with pkgs; ''
            -isystem ${llvmPackages.libclang.lib}/lib/clang/${
              lib.getVersion clang
            }/include
            -isystem ${llvmPackages.libclang.out}/lib/clang/${
              lib.getVersion clang
            }/include
            -isystem ${glibc.dev}/include
          '';

          inherit buildInputs nativeBuildInputs;
          shellHook = ''
            export LD_LIBRARY_PATH=${stdenv.cc.cc.lib}/lib
          '';
        };
      });
}
