{
  description = "LED Star - Rust-based LED controller for Arduino";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            # Rust toolchain
            rustup
            
            # AVR toolchain for Arduino
            pkgsCross.avr.buildPackages.gcc
            avrdude
            
            # Additional tools
            cargo-watch
            
            # Node.js for visualizer
            nodejs
            
            # Build tools
            pkg-config
          ];

          shellHook = ''
            echo "🎄 LED Star Development Environment"
            echo ""
            echo "Available commands:"
            echo "  cargo xtask arduino build  - Build Arduino firmware"
            echo "  cargo xtask arduino flash  - Flash to Arduino"
            echo "  cargo xtask visualizer dev - Run web visualizer"
            echo ""
            echo "AVR toolchain: $(avr-gcc --version | head -n1)"
            echo "avrdude: $(avrdude -? 2>&1 | head -n1)"
          '';
        };
      }
    );
}
