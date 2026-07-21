{
  lib,
  stdenv,
  bun,
  nodejs,
  darwin,
  electron_41,
  makeWrapper,
  writableTmpDirAsHomeHook,
  autoPatchelfHook,
  copyDesktopItems,
  makeDesktopItem,
  bugbee,
}:
let
  electron = electron_41;
in
stdenv.mkDerivation (finalAttrs: {
  pname = "bugbee-desktop";
  inherit (bugbee)
    version
    src
    node_modules
    patches
    ;

  nativeBuildInputs = [
    bun
    nodejs
    makeWrapper
    writableTmpDirAsHomeHook
  ]
  ++ lib.optionals stdenv.hostPlatform.isLinux [
    autoPatchelfHook
    copyDesktopItems
  ]
  ++ lib.optionals stdenv.hostPlatform.isDarwin [
    # Ad-hoc sign the .app: --config.mac.identity=null below skips signing.
    darwin.autoSignDarwinBinariesHook
  ];

  buildInputs = lib.optionals stdenv.hostPlatform.isLinux [
    (lib.getLib stdenv.cc.cc)
  ];

  desktopItems = lib.optional stdenv.hostPlatform.isLinux (makeDesktopItem {
    name = "ai.bugbee.desktop";
    desktopName = "Bugbee";
    exec = "bugbee-desktop %U";
    icon = "ai.bugbee.desktop";
    # Electron 41 derives X11 WM_CLASS from app.name.
    startupWMClass = "Bugbee";
    categories = [ "Development" ];
  });

  env = bugbee.env // {
    ELECTRON_SKIP_BINARY_DOWNLOAD = "1";
  };

  postPatch =
    # NOTE: Relax Bun version check to be a warning instead of an error
    ''
      substituteInPlace packages/script/src/index.ts \
        --replace-fail 'throw new Error(`This script requires bun@''${expectedBunVersionRange}' \
                       'console.warn(`Warning: This script requires bun@''${expectedBunVersionRange}'
    ''
    # https://github.com/electron/electron/issues/31121
    # mac builds use a .app bundle which doesnt have this issue
    + lib.optionalString stdenv.isLinux ''
      BASE_PATH=packages/desktop
      FILES=(src/main/windows.ts)
      for file in "''${FILES[@]}"; do
        substituteInPlace $BASE_PATH/$file \
          --replace-fail "process.resourcesPath" "'$out/opt/bugbee-desktop/resources'"
      done
    '';

  preBuild = ''
    cp -r "${electron.dist}" $HOME/.electron-dist
    chmod -R u+w $HOME/.electron-dist

    cp -R ${finalAttrs.node_modules}/. .
    patchShebangs node_modules
    patchShebangs packages/*/node_modules
  '';

  buildPhase = ''
    runHook preBuild

    cd packages/desktop

    bun run build
    npx electron-builder --dir \
      --config electron-builder.config.ts \
      --config.mac.identity=null \
      --config.electronDist="$HOME/.electron-dist"

    runHook postBuild
  '';

  installPhase = ''
    runHook preInstall
  ''
  + lib.optionalString stdenv.hostPlatform.isDarwin ''
    mkdir -p $out/Applications
    mv dist/mac*/*.app $out/Applications
    makeWrapper "$out/Applications/Bugbee.app/Contents/MacOS/Bugbee" $out/bin/bugbee-desktop
  ''
  + lib.optionalString stdenv.hostPlatform.isLinux ''
    mkdir -p $out/opt/bugbee-desktop
    cp -r dist/linux*-unpacked/{resources,LICENSE*} $out/opt/bugbee-desktop
    install -Dm644 resources/icons/32x32.png \
      "$out/share/icons/hicolor/32x32/apps/ai.bugbee.desktop.png"
    install -Dm644 resources/icons/64x64.png \
      "$out/share/icons/hicolor/64x64/apps/ai.bugbee.desktop.png"
    install -Dm644 resources/icons/128x128.png \
      "$out/share/icons/hicolor/128x128/apps/ai.bugbee.desktop.png"
    install -Dm644 resources/icons/128x128@2x.png \
      "$out/share/icons/hicolor/256x256/apps/ai.bugbee.desktop.png"
    install -Dm644 resources/icons/icon.png \
      "$out/share/icons/hicolor/512x512/apps/ai.bugbee.desktop.png"
    install -Dm644 resources/ai.bugbee.desktop.metainfo.xml \
      "$out/share/metainfo/ai.bugbee.desktop.metainfo.xml"
    makeWrapper ${lib.getExe electron} $out/bin/bugbee-desktop \
     --inherit-argv0 \
     --set ELECTRON_FORCE_IS_PACKAGED 1 \
     --add-flags $out/opt/bugbee-desktop/resources/app.asar \
     --add-flags "\''${NIXOS_OZONE_WL:+\''${WAYLAND_DISPLAY:+--ozone-platform-hint=auto --enable-features=WaylandWindowDecorations --enable-wayland-ime=true}}"
  ''
  + ''
    runHook postInstall
  '';

  autoPatchelfIgnoreMissingDeps = [
    "libc.musl-x86_64.so.1"
  ];

  meta = {
    description = "Bugbee Desktop App";
    mainProgram = "bugbee-desktop";
    inherit (bugbee.meta) homepage license platforms;
  };
})
