{
  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs { inherit system; };

      buildInputs = with pkgs; [
          cargo
          pkg-config
          udev
          alsaLib
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libXi
          vulkan-tools
          vulkan-headers
          vulkan-loader
          vulkan-validation-layers
      ];

      LD_LIBRARY_PATH = ''${pkgs.lib.makeLibraryPath [
          pkgs.alsaLib
          pkgs.udev
          pkgs.xorg.libX11
          pkgs.xorg.libXcursor
          pkgs.vulkan-loader
          pkgs.xorg.libXi
          pkgs.xorg.libXrandr
      ]}'';

      vs-rs-unwrapped = with crane.lib.${system}; buildPackage {
        src = cleanCargoSource (path ./.);
        pname = "vs-rs";
        version = "0.1.0";

        inherit buildInputs LD_LIBRARY_PATH;
        cargoVendorDir = vendorCargoDeps { cargoLock = ./Cargo.lock; };
      };

      wrap = pkg: pkgs.runCommand pkg.name {
        inherit (pkg) pname version;
        nativeBuildInputs = [pkgs.makeWrapper];
      }
        ''
          cp -rs --no-preserve=mode,ownership ${pkg} $out
          wrapProgram "$out/bin/${pkg.pname}" ''${makeWrapperArgs[@]} --set LD_LIBRARY_PATH "${LD_LIBRARY_PATH}"
        '';

      vs-rs = wrap vs-rs-unwrapped;
    in {
      devShells.default = pkgs.mkShell {
        inherit buildInputs LD_LIBRARY_PATH;
      };

      packages = {
        inherit vs-rs vs-rs-unwrapped;

        default = self.packages.${system}.vs-rs;
      };

      apps.default = {
        type = "app";
        program = "${self.packages.${system}.default}/bin/vs-rs";
      };
    });
}
