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
    let
      nixosModule =
        {
          pkgs,
          lib,
          config,
          ...
        }:
        {
          options.services.inky_display_server = {
            enable = lib.mkEnableOption "Enable Inky Display Server";

            port = lib.mkOption {
              type = lib.types.port;
              default = 8080;
              description = "Port to listen on";
            };
            frame_url = lib.mkOption {
              type = lib.types.str;
              description = "Url of the frame";
            };
          };

          config = lib.mkIf config.services.inky_display_server.enable {
            systemd.services.inky_display_server = {
              description = "Inky Display Server";
              wantedBy = [ "multi-user.target" ];
              after = [ "network.target" ];
              serviceConfig = {
                Type = "simple";
                DynamicUser = true;
                Environment = [
                  "PATH=${pkgs.google-chrome}/bin:$PATH"
                ];
                ExecStart = "${self.packages.${pkgs.system}.server}/bin/server";
                Restart = "on-failure";
              };
              environment = {
                PORT = toString config.services.inky_display_server.port;
                FRAME_URL = toString config.services.inky_display_server.frame_url;
              };
            };
          };
        };
    in
    (
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
                  "ico"
                  "png"
                  "svg"
                  "webmanifest"
                ]
              ) unfilteredRoot)
            ];
          };

          nativeBuildInputs = with pkgs; [
            cargo
            chromedriver
            openssl
            pkg-config
            rustc
            ungoogled-chromium
            tailwindcss
            makeWrapper
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
          server = craneLib.buildPackage (
            commonArgs
            // rec {
              inherit cargoArtifacts;
              pname = "server";
              cargoExtraArgs = "--bin server";

              preBuild = ''
                tailwindcss -i ./input.css -o $out/bin/static/output.css --minify
              '';

              installPhaseCommand = ''
                mkdir -p $out/bin/static/fonts
                cp target/release/${pname} $out/bin/
                cp ${pkgs.nerd-fonts.iosevka}/share/fonts/truetype/NerdFonts/Iosevka/IosevkaNerdFont-{Regular,Bold}.ttf $out/bin/static/fonts/
                wrapProgram $out/bin/${pname} --set STATIC_ROOT $out/bin/static
              '';
            }
          );

          cross-crate = crossPkgs.callPackage craneLibCross.buildPackage {
            nativeBuildInputs = with pkgs; [
              openssl
              pkg-config
            ];
            inherit src;
            strictDeps = true;
            pname = "frame";
            cargoExtraArgs = "--bin frame";
          };
        in
        {
          packages = {
            inherit server;
            default = server;
            cross = cross-crate;
          };

          apps.default = flake-utils.lib.mkApp {
            drv = server;
          };

          devShells.default = craneLib.devShell {
            inputsFrom = [ server ];
            shellHook = ''
              exec fish
            '';
          };
        }
      )
      // {
        nixosModules.default = nixosModule;
      }
    );
}
