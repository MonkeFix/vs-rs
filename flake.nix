{
  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { nixpkgs, flake-utils, ... }@inputs:
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells.default = pkgs.mkShell {
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

        shellHook = ''export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${pkgs.lib.makeLibraryPath [
          pkgs.alsaLib
          pkgs.udev
          pkgs.xorg.libX11
          pkgs.xorg.libXcursor
          pkgs.vulkan-loader
          pkgs.xorg.libXi
          pkgs.xorg.libXrandr
        ]}"'';
      };
    });
}
