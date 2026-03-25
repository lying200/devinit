{
  lib,
  rustPlatform,
  makeWrapper,
  git,
}:

rustPlatform.buildRustPackage rec {
  pname = "devinit";
  version = "0.1.0";

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
    mainProgram = "devinit";
    platforms = lib.platforms.linux;
  };
}
