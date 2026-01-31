{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane.url = "github:ipetkov/crane";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    weather-icons = {
      url = "github:roe-dl/weathericons";
      flake = false;
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      rust-overlay,
      crane,
      weather-icons,
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
          options.services.inky-display = {
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
            weather_lat = lib.mkOption {
              type = lib.types.float;
              description = "Latitude to get weather from";
            };
            weather_long = lib.mkOption {
              type = lib.types.float;
              description = "Longitude to get weather from";
            };
            football_api_key = lib.mkOption {
              type = lib.types.str;
              description = "API key for api.football-data.org";
            };
            weather_api_key = lib.mkOption {
              type = lib.types.str;
              description = "API key for open-meteo.com";
            };
          };

          config = lib.mkIf config.services.inky-display.enable {
            systemd.services.inky-display = {
              description = "Inky Display Server";
              wantedBy = [ "multi-user.target" ];
              after = [ "network.target" ];
              serviceConfig = {
                Type = "simple";
                DynamicUser = true;
                Environment = [
                  "PATH=${pkgs.ungoogled-chromium}/bin:${pkgs.chromedriver}/bin:$PATH"
                ];
                ExecStart = "${self.packages.${pkgs.system}.server}/bin/server";
                Restart = "on-failure";
              };
              environment = {
                PORT = toString config.services.inky-display.port;
                FRAME_URL = toString config.services.inky-display.frame_url;
                WEATHER_LAT = toString config.services.inky-display.weather_lat;
                WEATHER_LONG = toString config.services.inky-display.weather_long;
                FOOTBALL_API_KEY = toString config.services.inky-display.football_api_key;
                TUBE_API_KEY = toString config.services.inky-display.weather_api_key;
              };
            };
          };
        };
    in
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
                "ico"
                "js"
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
              tailwindcss -c ./tailwind.config.js -i ./input.css -o $out/bin/static/output.css --minify
            '';

            installPhaseCommand = ''
              mkdir -p $out/bin/static/fonts

              cp target/release/${pname} $out/bin/
              cp ${pkgs.nerd-fonts.iosevka}/share/fonts/truetype/NerdFonts/Iosevka/IosevkaNerdFont-{Regular,Bold}.ttf $out/bin/static/fonts/

              mkdir -p $out/bin/static/icons
              cp ${weather-icons}/weathericons-filled/*.svg $out/bin/static/icons/

              cp ./favicon.ico $out/bin/static/

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
        };
      }
    )
    // {
      nixosModules.default = nixosModule;
    };
}
