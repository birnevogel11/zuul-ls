# Zuul langauge server + zuul-search


## Development using VSCode

**Known limitation**: The plugin can not work with ansible-ls together in VSCode

1. `pnpm i`
2. `cargo build`
3. Open the project in VSCode: `code .`
4. In VSCode, press <kbd>F5</kbd> or change to the Debug panel and click <kbd>Launch Client</kbd>.
5. In the newly launched VSCode instance, open the file `examples/test.nrs` from this project.
6. If the LSP is working correctly you should see syntax highlighting and the features described below should work.
