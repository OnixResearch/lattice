# Fixture driver tools from Lattice's immutable nixpkgs input. The producer
# component/runtime build deliberately uses the separately retained producer lock.
let
  lock = builtins.fromJSON (builtins.readFile ../../flake.lock);
  pkgs = import (builtins.fetchTree lock.nodes.nixpkgs.locked) {};
in pkgs.mkShell {
  nativeBuildInputs = [ pkgs.python3 pkgs.b3sum pkgs.git pkgs.nix pkgs.cacert pkgs.procps ];
  SSL_CERT_FILE = "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt";
}
