with import <nixpkgs> { };

mkShell rec {
  # include any libraries or programs in buildInputs
  nativeBuildInputs = [ pkg-config cmake pkg-config freetype expat fontconfig ];

  buildInputs = [
    sccache
    udev
    alsa-lib
    vulkan-loader
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    xorg.libXrandr # To use the x11 feature
    libxkbcommon
    wayland # To use the wayland feature
  ];
  # shell commands to be ran upon entering shell
  shellHook = "";

  LD_LIBRARY_PATH = lib.makeLibraryPath buildInputs;

}
