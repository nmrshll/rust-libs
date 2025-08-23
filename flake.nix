{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
    utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "utils";
    };
    # crane = {
    #   url = "github:ipetkov/crane";
    #   inputs.nixpkgs.follows = "nixpkgs";
    # };
    my-utils = {
      url = "github:nmrshll/nix-utils";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.utils.follows = "utils";
      # inputs.rust-overlay.follows = "rust-overlay";
    };
  };

  outputs = { self, nixpkgs, rust-overlay, utils, my-utils }:
    with builtins; utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        customRust = pkgs.rust-bin.stable."1.87.0".default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ ];
        };

        buildDependencies = [
          customRust
          pkgs.nodePackages_latest.pnpm
          pkgs.pkg-config
        ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
          # darwin.apple_sdk.frameworks.Security
          # darwin.apple_sdk.frameworks.SystemConfiguration
          # darwin.apple_sdk.frameworks.CoreServices
          # darwin.apple_sdk.frameworks.CoreFoundation
          # darwin.apple_sdk.frameworks.Foundation
          pkgs.apple-sdk_15
          pkgs.libiconv
        ];
        devDependencies = [
          pkgs.cargo-edit
          pkgs.watchexec
        ];

        env = {
          RUST_LOG = "debug";
          RUST_BACKTRACE = 1;
        };

        binaries = my-utils.binaries.${system};
        scripts = with pkgs; attrValues my-utils.packages.${system} ++ [
          (writeScriptBin "utest" ''cargo test --workspace -- --nocapture'')
        ];

      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = buildDependencies ++ devDependencies ++ scripts;
          shellHook = ''
            ${my-utils.binaries.${system}.configure-editors};
            dotenv
          '';
          inherit env;
        };
      }
    );
}
