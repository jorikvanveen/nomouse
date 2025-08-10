{
  inputs = { utils.url = "github:numtide/flake-utils"; };
  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let pkgs = nixpkgs.legacyPackages.${system};
      in {
        devShell = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            rust-analyzer
            rustfmt
            pkg-config
            udev
            wayland-protocols
            libxkbcommon
          ];
        };
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "nomouse";
          version = "0.0.1";
          src = ./.;
          cargoLock = { lockFile = ./Cargo.lock; };
          nativeBuildInputs = [ pkgs.pkg-config ];
          buildInputs = [ pkgs.libxkbcommon ];
        };
      });
}
