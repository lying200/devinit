{
  description = "devinit – generate devenv files for development projects";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    { self, nixpkgs }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          package = pkgs.callPackage ./nix/package.nix { };
        in
        {
          default = package;
          devinit = package;
        }
      );

      apps = forAllSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.devinit}/bin/devinit";
        };
      });

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              clippy
              git
              nixfmt-rfc-style
              rust-analyzer
              rustc
              rustfmt
            ];
          };
        }
      );
    };
}
