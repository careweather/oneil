# Oneil Programming Language Support for VS Code

This VS Code extension adds language support for the Oneil programming language.

## Current Features

- **Syntax Highlighting**: Enhances code readability with semantic syntax highlighting for Oneil language constructs.
- **Go to Definition**: Jump to the definition of parameters, models, and imported Python files and functions
- **Hover Information**: View the labels and notes associated with a parameter, model, function, or Python module by hovering
- **Inline Errors**: See errors directly in the editor
- **Rendered View**: Preview models with notes rendered as Markdown (math, `{{param:value|equation}}` interpolation, citations). Prefer a `references.bib` beside the model or at the workspace root — see the [Notes](https://careweather.github.io/oneil/08-notes.html) guide chapter.
- **CLI install / update**: Download the Oneil CLI from GitHub Releases via Command Palette commands when it is not already on PATH

## Planned Features

We are actively working on expanding the capabilities of this extension. Upcoming features include:

- **IntelliSense and Autocomplete**: Smart code completion for Oneil
- **More to come!**: We're continuously working to improve the development experience for Oneil programmers

## Installation

1. Open VS Code
2. Go to the Extensions view (Ctrl+Shift+X)
3. Search for "Oneil"
4. Click Install

## Requirements

- Visual Studio Code version 1.0.0 or higher
- An Oneil CLI for the language server. The extension can **download it from GitHub Releases** (Command Palette: “Oneil: Install or Update CLI”, or accept the prompt when none is found). You can also put `oneil` on your PATH, set `ONEIL_PATH`, or set **Oneil: Server Path** to a local build.
- **Python 3.12 or 3.14** for the release CLI (`brew install python@3.14`, the python.org installer, or `uv python install 3.14`). The extension downloads the archive for the newest of those that is installed.

>[!NOTE]
> `oneil.serverPath` always wins over the extension-managed install. Clear it to use managed updates / version selection. Use **Oneil: Select CLI Version…** to install a different published release (re-downloads over the managed binary). Use **Oneil: Check for CLI Updates** to compare against the latest GitHub release.

## Known Issues

Please [report any issues](https://github.com/careweather/oneil/issues) you encounter on our GitHub repository.

## Contributing

We welcome contributions! Feel free to [submit pull requests](https://github.com/careweather/oneil/pulls) or [open issues](https://github.com/careweather/oneil/issues) on our GitHub repository.

