# Contributing

Thank you for your interest in contributing to the Oneil VS Code extension! This extension provides language support for the Oneil programming language, enhancing the development experience with features like syntax highlighting and more to come. Whether you want to report bugs, suggest features, improve documentation, or contribute code, your help is greatly appreciated. This guide will help you get started with contributing to the project.

## Developing

### Running the extension locally

In order to run/debug the extension locally, follow the instructions from the
[VS Code docs](https://code.visualstudio.com/api/get-started/your-first-extension):

> Inside the editor, [ ... ] press `F5` or run the command > `Debug: Start
> Debugging` from the Command Palette (`Ctrl+Shift+P`). This will > compile and
> run the extension in a new Extension Development Host window.

### Syntax highlighting

In order to modify the syntax highlighting, edit
`syntaxes/oneil.tmLanguage.json`. For more details, see the [VS Code docs on
syntax highlighting](https://code.visualstudio.com/api/language-extensions/syntax-highlight-guide).

<!-- TODO: Setup CI workflow for publishing extension, as described at
https://github.com/EclipseFdn/publish-extensions/blob/master/docs/direct_publish_setup.md#setup-vs-code-extension-publishing-ci-workflow -->

### Publishing the extension for VS Code

For details on how the extension is published, reference [the VS Code
docs on publishing extensions](https://code.visualstudio.com/api/working-with-extensions/publishing-extension).

### Publishing the extension for VS Code forks

In order to use the extension on VS Code forks, such as Cursor, it
needs to be published on the OpenVSX registry. That process is described
[here](https://github.com/eclipse/openvsx/wiki/Publishing-Extensions).
You also need to be associated with the `careweather` namespace,
described [here](https://github.com/eclipse/openvsx/wiki/Namespace-Access).
