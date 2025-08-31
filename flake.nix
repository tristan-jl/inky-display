{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    self.submodules = true;
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };

        crossPkgs = import nixpkgs {
          crossSystem = "aarch64-linux-musl";
          localSystem = system;
          overlays = [ (import rust-overlay) ];
        };

        rustToolchain = pkgs.pkgsBuildHost.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;
        craneLibCross = (crane.mkLib crossPkgs).overrideToolchain (p: p.rust-bin.stable.latest.default);

        inherit (pkgs) lib;
        unfilteredRoot = ./.;
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            (lib.fileset.fileFilter (
              file:
              lib.any file.hasExt [
                "css"
                "html"
              ]
            ) unfilteredRoot)
          ];
        };

        nativeBuildInputs = with pkgs; [
          cargo
          chromedriver
          ungoogled-chromium
          rustc
        ];
        buildInputs = with pkgs; [
          nerd-fonts.iosevka
          nixd
          rust-analyzer
          rustPackages.clippy
          rustToolchain
          tailwindcss
          tailwindcss-language-server
        ];

        commonArgs = {
          inherit src buildInputs nativeBuildInputs;
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        bin = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
            preBuild = ''
              mkdir -p $out/static/fonts
              tailwindcss -i ./input.css -o $out/static/output.css --minify
              cp ${pkgs.nerd-fonts.iosevka}/share/fonts/truetype/NerdFonts/Iosevka/IosevkaNerdFont-Regular.ttf $out/static/fonts/
              cp ${pkgs.nerd-fonts.iosevka}/share/fonts/truetype/NerdFonts/Iosevka/IosevkaNerdFont-Bold.ttf $out/static/fonts/
            '';
          }
        );

        cross-crate = crossPkgs.callPackage craneLibCross.buildPackage {
          inherit src;
          strictDeps = true;
        };
      in
      with pkgs;
      {
        packages = {
          inherit bin;
          default = bin;
          cross = cross-crate;
        };

        devShells.default = mkShell {
          inherit buildInputs nativeBuildInputs;

          shellHook = ''
            exec fish
          '';
        };
      }
    );
}
