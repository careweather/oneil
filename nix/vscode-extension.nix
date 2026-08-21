{
  lib,
  buildNpmPackage,
  vscode-utils,
  vsce,
  jq,
  moreutils,
  oneil,
}:

let
  version = (lib.importJSON ../vscode/package.json).version;

  src = lib.fileset.toSource {
    root = ../.;
    fileset = lib.fileset.difference (lib.fileset.unions [
      ../vscode
      ../packages/ts-interfaces
      ../docs/icons
    ]) (lib.fileset.unions [
      (lib.fileset.maybeMissing ../vscode/node_modules)
      (lib.fileset.maybeMissing ../vscode/out)
      (lib.fileset.maybeMissing ../vscode/model-renderer/node_modules)
    ]);
  };

  modelRenderer = buildNpmPackage {
    pname = "oneil-model-renderer";
    inherit version src;
    sourceRoot = "source/vscode/model-renderer";
    npmDepsHash = "sha256-mWcU/eIdzEthoAjcZbmY47IUxk0VoTYuFKUFs5jo4/4=";
    dontNpmInstall = true;

    # Vite writes to ../out/model-renderer; unpackPhase only chmod's sourceRoot.
    preBuild = ''
      chmod u+w ..
      mkdir -p ../out
    '';
    installPhase = ''
      runHook preInstall
      mkdir -p "$out"
      cp -R ../out/model-renderer/. "$out/"
      runHook postInstall
    '';
  };

  vsix = buildNpmPackage {
    pname = "oneil-vscode";
    name = "oneil-${version}.vsix";
    inherit version src;
    sourceRoot = "source/vscode";
    npmDepsHash = "sha256-xSrw3r/RdMSE+nROgSsOctBJjJPITfygCmheOPuPKtU=";
    nativeBuildInputs = [
      vsce
      jq
    ];
    npmBuildScript = "bundle";
    npmBuildFlags = [
      "--"
      "--production"
    ];
    dontNpmInstall = true;

    preBuild = ''
      mkdir -p out/model-renderer
      cp -R ${modelRenderer}/. out/model-renderer/
    '';

    installPhase = ''
      runHook preInstall
      # vsce runs vscode:prepublish, which would try to rebuild the webview
      # without its npm dependencies. The renderer is already copied in preBuild.
      jq 'del(.scripts["vscode:prepublish"])' package.json > package.json.tmp
      mv package.json.tmp package.json
      vsce package --follow-symlinks --out "$out"
      runHook postInstall
    '';
  };
in
vscode-utils.buildVscodeExtension {
  pname = "oneil";
  inherit version vsix;
  src = vsix;
  vscodeExtPublisher = "careweather";
  vscodeExtName = "oneil";
  vscodeExtUniqueId = "careweather.oneil";

  nativeBuildInputs = [
    jq
    moreutils
  ];

  preInstall = ''
    jq '.contributes.configuration.properties["oneil.serverPath"].default = $s' \
      --arg s "${oneil}/bin/oneil" \
      package.json | sponge package.json

    grep -Fq "${oneil}/bin/oneil" package.json || {
      echo "Setting oneil.serverPath default in package.json failed."
      exit 1
    }
  '';

  passthru = {
    inherit vsix modelRenderer;
  };

  meta = {
    description = "VS Code support for the Oneil design specification language";
    homepage = "https://github.com/careweather/oneil";
    changelog = "https://github.com/careweather/oneil/blob/main/vscode/CHANGELOG.md";
    downloadPage = "https://marketplace.visualstudio.com/items?itemName=careweather.oneil";
    license = lib.licenses.mpl20;
    platforms = lib.platforms.unix;
  };
}
