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
      lib = pkgs.lib;
      craneLib = crane.lib.${system};

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

      LD_LIBRARY_PATH = ''${lib.makeLibraryPath [
          pkgs.alsaLib
          pkgs.udev
          pkgs.xorg.libX11
          pkgs.xorg.libXcursor
          pkgs.vulkan-loader
          pkgs.xorg.libXi
          pkgs.xorg.libXrandr
      ]}'';

      commonArgs = {
        src = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (lib.hasInfix "/assets/" path) ||
            (craneLib.filterCargoSources path type)
          ;
        };
      };

      cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
        inherit buildInputs;
      });

      vs-rs-unwrapped = with craneLib; buildPackage (commonArgs // {
        pname = "vs-rs";
        version = "0.1.0";

        inherit cargoArtifacts LD_LIBRARY_PATH;
        cargoVendorDir = vendorCargoDeps { cargoLock = ./Cargo.lock; };

        postInstall = ''
          cp -r --no-preserve=mode,ownership ./assets/ $out/bin/assets
        '';
      });

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
