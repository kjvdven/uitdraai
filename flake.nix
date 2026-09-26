{
  description = "uitdraai development shell";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
    in
    {
      # Rust itself comes from mise (mise.toml); this shell only adds system libraries.
      devShells.x86_64-linux.default = pkgs.mkShell {
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = [
          pkgs.gtk4
          pkgs.webkitgtk_6_0
        ];
        packages = [ pkgs.python3Packages.weasyprint ];
      };
    };
}
