{ pkgs, ... }:
{
  # Used to find the project root
  projectRootFile = "flake.nix";
  programs.nixfmt.enable = true;
  programs.rustfmt.enable = true;
  programs.rustfmt.package = pkgs.fx.stable.rustfmt;
}
