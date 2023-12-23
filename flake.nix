{
  description = "A Rust project";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustc
            cargo
            rust-analyzer
            rustfmt
            clippy
            cargo-watch

            prettierd

            # Markdown
            proselint
            marksman
            markdownlint-cli2
          ];

          shellHook = ''
            echo "Let's get Rusty" | "${pkgs.lolcat}/bin/lolcat"
          '';
        };
      });
}
