{
  description = "Build a cargo workspace";

  nixConfig = {
    extra-trusted-public-keys = "devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=";
    extra-substituters = "https://devenv.cachix.org";
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    devenv.url = "github:cachix/devenv";
    devenv-root = {
      url = "file+file:///dev/null";
      flake = false;
    };
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      imports = [ inputs.devenv.flakeModule ];
      systems = [ "x86_64-linux" ];

      perSystem =
        {
          pkgs,
          system,
          lib,
          ...
        }:
        let
          rustVersion = "stable";
          fenixPkgs = inputs.fenix.packages.${system}.${rustVersion};
          rustToolchain = fenixPkgs.toolchain;

          runtimeInputs = with pkgs; [ ];
          nativeBuildInputs = with pkgs; [
            mold
          ];
          buildInputs = with pkgs; [
            openssl
          ];
        in
        {
          _module.args.pkgs = import inputs.nixpkgs {
            inherit system;
            overlays = [ inputs.fenix.overlays.default ];
          };

          devenv.shells.default = {
            devenv.root =
              let
                devenvRootFileContent = builtins.readFile inputs.devenv-root.outPath;
              in
              lib.mkIf (devenvRootFileContent != "") devenvRootFileContent;

            name = "proxy-processor-devenv";
            env = {
              RUST_LOG = "debug,maxminddb::decoder=warn";
            };
            packages =
              with pkgs;
              [
                cargo-audit
                cargo-flamegraph
                cargo-machete
                cargo-nextest
                nixfmt-rfc-style
              ]
              ++ runtimeInputs
              ++ buildInputs
              ++ nativeBuildInputs;
            languages.rust = {
              enable = true;
              channel = "nixpkgs";
              components = [
                "cargo"
                "clippy"
                "rustc"
                "rustfmt"
              ];
              toolchain = rustToolchain;
              mold.enable = true;
            };
            git-hooks.hooks = {
              clippy = {
                enable = true;
                settings.denyWarnings = true;
              };
              deadnix.enable = true;
              nixfmt-rfc-style.enable = true;
              ripsecrets.enable = true;
              rustfmt.enable = true;
            };
            containers = lib.mkForce { };
          };
        };
    };
}
