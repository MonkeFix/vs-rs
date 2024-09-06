{
  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, crane, fenix, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs { inherit system; };
      lib = pkgs.lib;
      craneLib = (crane.mkLib nixpkgs.legacyPackages.${system}).overrideToolchain
        fenix.packages.${system}.minimal.toolchain;

      buildInputs = with pkgs; [
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
      # TODO: deduplicate
      LD_LIBRARY_PATH = with pkgs; ''${lib.makeLibraryPath [
        alsaLib
        udev
        xorg.libX11
        xorg.libXcursor
        vulkan-loader
        xorg.libXi
        xorg.libXrandr
        libxkbcommon
      ]}'';

      commonArgs = {
        src = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (lib.hasInfix "/vs-rs/assets/" path) ||
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

        # TODO: Fix tests
        doCheck = false;

        postInstall = ''
          cp -r --no-preserve=mode,ownership ./vs-rs/assets $out/bin/
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
      devShells.default = craneLib.devShell {
        inherit LD_LIBRARY_PATH buildInputs;
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
