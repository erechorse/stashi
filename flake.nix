{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs = inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
      perSystem = { self', pkgs, system, ... }:
        let
          craneLib = inputs.crane.mkLib pkgs;

          src = craneLib.cleanCargoSource ./.;

          nativeBuildInputs = with pkgs; [ pkg-config cacert ];

          buildInputs = with pkgs; [
            openssl
          ] ++ lib.optionals stdenv.isDarwin [
            libiconv
            darwin.apple_sdk.frameworks.Security
            darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          commonArgs = {
            inherit src buildInputs nativeBuildInputs;
          };

          cargoArtifacts = craneLib.buildDepsOnly commonArgs;

          stashi = craneLib.buildPackage (commonArgs // {
            inherit cargoArtifacts;
          });
        in
        {
          checks = {
            inherit stashi;
          };

          packages.default = stashi;
          packages.stashi = stashi;

          devShells.default = craneLib.devShell {
            checks = self'.checks;
            packages = with pkgs; [
              cargo
              rustc
              rustfmt
              clippy
            ];
          };
        };
    };
}
