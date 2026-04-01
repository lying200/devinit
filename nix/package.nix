{
  lib,
  rustPlatform,
  makeWrapper,
  git,
}:

rustPlatform.buildRustPackage {
  pname = "devinit";
  # Keep in sync with Cargo.toml
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
    homepage = "https://github.com/lying200/devinit";
    license = lib.licenses.mit;
    mainProgram = "devinit";
    platforms = lib.platforms.linux;
  };
}
