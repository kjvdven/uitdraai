{
  description = "uitdraai development shell";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
      libs = [
        pkgs.glib
        pkgs.gtk4
        pkgs.pango
        pkgs.webkitgtk_6_0
      ];
    in
    {
      # Rust itself comes from mise (mise.toml); this shell only adds system libraries.
      devShells.x86_64-linux.default = pkgs.mkShell {
        nativeBuildInputs = [ pkgs.pkg-config ];
        buildInputs = libs;
        packages = [ pkgs.python3Packages.weasyprint ];
        # cargo-built binaries get no RUNPATH to the Nix store. LD_LIBRARY_PATH is no
        # option: a global mise [env] can overwrite it, so bake the path in instead.
        RUSTFLAGS = "-C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath libs}";
      };
    };
}
