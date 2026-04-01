{
  lib,
  rustPlatform,
  makeWrapper,
  git,
}:

let
  cargoToml = builtins.fromTOML (builtins.readFile ../Cargo.toml);
in
rustPlatform.buildRustPackage {
  pname = cargoToml.package.name;
  inherit (cargoToml.package) version;

  src = lib.cleanSource ../.;

  cargoLock = {
    lockFile = ../Cargo.lock;
  };

  nativeBuildInputs = [ makeWrapper ];
  nativeCheckInputs = [ git ];

  postInstall = ''
    wrapProgram "$out/bin/devinit" \
      --prefix PATH : ${lib.makeBinPath [ git ]}
  '';

  meta = {
    description = "Generate devenv files for development projects";
    homepage = "https://github.com/lying200/devinit";
    license = lib.licenses.mit;
    mainProgram = "devinit";
    platforms = lib.platforms.linux;
  };
}
