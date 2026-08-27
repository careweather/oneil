# Change Log

<!--
    All notable changes to the "oneil" extension will be documented in this file.

    Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.
-->

## v1.0.1 - 2026-08-26

- Bump ESLint 10, vscode-languageclient 10, Vite 8, and @vitejs/plugin-react 6 so patched transitives (minimatch, brace-expansion, postcss, picomatch) come in from parent ranges. The extension now requires VS Code 1.91+.

## v1.0.0 - 2026-08-12

- Align extension version with Oneil 1.0.0 (Rust implementation on `main`)
- Extension-managed CLI: download / update / select GitHub release versions for the language server (1.x only, including 1.0.0 prereleases; 0.x hidden). “Latest” prefers stable 1.x and falls back to the newest 1.x beta. The release CLI needs Python 3.12; the extension picks the Homebrew, system, or uv archive that matches the machine. Listing uses git tags (not the GitHub Releases collection API) so large multi-flavor releases do not 504.
- Resolve note images and relative citation PDFs from the model file’s directory first, then the workspace root (same order as `references.bib`)


## v0.3.0 - 2026-06-29

- Syntax highlighting update
- Rendering model view

## v0.2.2 - 2026-04-15

- Syntax highlighting update

### Fixed

- Updated the syntax highlighting grammar to better match the actual grammar
  - Fixes [issue #21](https://github.com/careweather/oneil/issues/21)
  - Fixes [issue #22](https://github.com/careweather/oneil/issues/22)
  - Fixes [issue #24](https://github.com/careweather/oneil/issues/24)

## v0.2.1 - 2026-04-01

- Dependency fix

### Fixed

- Moved `vscode-languageclient` from a dev dependency to a release dependency

## v0.2.0 - 2026-04-01

- Initial LSP release

### Added

- jump to definition
- docs on hover
- inline errors
- logo

## v0.1.0

- Initial release

### Added

- basic syntax highlighting
