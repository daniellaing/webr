{
  description = "A Rust project";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    flake-parts,
    ...
  } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = nixpkgs.lib.systems.flakeExposed;

      perSystem = {pkgs, ...}: let
        deploy = pkgs.writeShellApplication {
          name = "deploy";
          runtimeInputs = with pkgs; [openssh];
          text = ''
            ssh root@daniellaing.com <<'EOF'
              rm -rf /tmp/webr

              git clone /home/git/webr.git /tmp/webr
              cd /tmp/webr
              cargo install --path . --root /usr/local

              systemctl restart webr.service
            EOF
          '';
        };

        webr = pkgs.callPackage ./default.nix {};
      in {
        packages = {
          inherit webr;
          default = webr;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            deploy

            rustc
            cargo
            rust-analyzer
            rustfmt
            clippy
            cargo-watch
            cargo-edit

            prettierd

            # Markdown
            proselint
            marksman
            markdownlint-cli2
          ];

          shellHook = ''
            echo "Let's get Rusty" | "${pkgs.lolcat}/bin/lolcat"
            echo "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}"
          '';
        };
      };
    };
}
