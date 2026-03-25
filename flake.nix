{
  description = "devinit flake package";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      flake-utils,
      nixpkgs,
    }:
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        package = pkgs.callPackage ./nix/package.nix { };
      in
      {
        packages = {
          default = package;
          devinit = package;
        };

        apps.default = flake-utils.lib.mkApp { drv = package; };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            clippy
            git
            nixfmt
            rust-analyzer
            rustc
            rustfmt
          ];
        };
      }
    );
}
