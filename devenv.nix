{ pkgs, ... }:

{
  packages = [ pkgs.git ];

  languages.rust = {
    enable = true;
    channel = "stable";
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
    ];
  };

  git-hooks.hooks = {
    rustfmt.enable = false;
    clippy.enable = false;
  };
}
