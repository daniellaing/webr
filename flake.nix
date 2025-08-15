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
        deploy = pkgs.callPackage ({
          writeShellApplication,
          openssh,
          cargo,
          patchelf,
        }:
          writeShellApplication {
            name = "deploy";
            runtimeInputs = [openssh cargo patchelf];
            text = ''
              cargo clean
              cargo build --release
              patchelf --set-interpreter /usr/lib64/ld-linux-x86_64.so.2 target/release/webr
              scp target/release/webr root@daniellaing.com:~
              ssh root@daniellaing.com <<'EOF'
                chown root:root ~/webr
                chmod 511 ~/webr
                mv -u ~/webr /usr/local/bin/webr
                systemctl restart webr.service
              EOF
            '';
          }) {};

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
