{
  description = "A Rust project";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  inputs.flake-utils.url = "github:numtide/flake-utils";

  outputs = { nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        deploy = pkgs.writeShellApplication {
          name = "deploy";
          runtimeInputs = with pkgs; [ openssh ];
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
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            deploy

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
