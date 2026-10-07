{
  description = "Lightweight Markdown previewer for Wayland";

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
      packages.x86_64-linux.default = pkgs.rustPlatform.buildRustPackage {
        pname = "uitdraai";
        version = "0.4.0";
        src = ./.;
        cargoLock.lockFile = ./Cargo.lock;
        nativeBuildInputs = [
          pkgs.pkg-config
          pkgs.wrapGAppsHook4
        ];
        buildInputs = libs;
        # PDF export shells out to weasyprint.
        preFixup = ''
          gappsWrapperArgs+=(--prefix PATH : ${pkgs.lib.makeBinPath [ pkgs.python3Packages.weasyprint ]})
        '';
        postInstall = ''
          install -Dm644 data/io.github.kjvdven.uitdraai.desktop -t $out/share/applications
          cp -r data/icons $out/share/icons
        '';
      };

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
