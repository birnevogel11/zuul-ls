# The project is still on heavily development. It does not guarantee any compatibility.

# zuul-ls (Zuul Language Server) + zuul-search

zuul-ls and zuul-search aim to provide a more efficient way to search and edit
[Zuul][zuul] CI config.

zuul-ls provides a small LSP with go-to-definition, auto-complete and
workspace symbols method for jobs, variables, playbooks, project-templates and
roles.

zuul-search can search jobs, project-templates or a job's variables or job
hierarchy.

## Need help

- Package the project into a VSCode plugin and release it

## Required dependencies

zuul-ls does not have any dependencies. zuul-search is wrapped by a `zs` script.
It requires [fzf][fzf] and [bat][bat].

## Installation

The project is tested with neovim and VSCode(partial). Need clone the repo and
copy/link the `zs` script

```bash
git clone https://github.com/birnevogel11/zuul-ls
cd zuul-ls
cargo build  # or `cargo build --release`
cargo install
cp zuul-ls/scripts/zs <bin>/  # e.g. `cp zuul-ls/scripts/zs /usr/local/bin`
# If the soure folder is not removed later ...
ln -sf zuul-s/scripts/zs <bin>/zs
```

### Neovim

1. Replace `neovim/nvim-lspconfig` with `birnevogel11/nvim-lspconfig`
2. Add lsp config

```lua
require("lspconfig").zuul_ls.setup(...)
```

3. Install [telescope-zuul-search.nvim][telescope-zuul-search.nvim]


### Development using VSCode

The document is copied from [IWANABETHATGUY/tower-lsp-boilerplate][tower-lsp-boilerplate].

**Known limitations**
1. The plugin can not work with ansible-ls together in VSCode.
2. It requires run with yaml plugin.

Steps:

1. `pnpm i`
2. `cargo build`
3. Open the project in VSCode: `code .`
4. In VSCode, press <kbd>F5</kbd> or change to the Debug panel and click
   <kbd>Launch Client</kbd>.
5. In the newly launched VSCode instance, open the file `examples/test.nrs`
   from this project.
6. If the LSP is working correctly you should see syntax highlighting and the
   features described below should work.

## Personal Note

The reason to create the project is that I would like to learn Rust and it's
useful for my daily job.  It's always welcome to send patches for any reason.

I know tower-lsp is a async program but the code has a lot of blocking IO call.
The most important reason is that I am unfamiliar with async in Rust. Maybe we
can change/rewrite these parts in the future.

Thanks to [IWANABETHATGUY/tower-lsp-boilerplate][tower-lsp-boilerplate]. It
saves me a lot of time to learn how to write a LS from scratch.

## Related projects

- [alexander-scott/zuul_job_browser](https://github.com/alexander-scott/zuul_job_browser)



[zuul]: https://zuul-ci.org/
[tower-lsp-boilerplate]: https://github.com/IWANABETHATGUY/tower-lsp-boilerplate/
[fzf]: https://github.com/junegunn/fzf
[bat]: https://github.com/sharkdp/bat
[telescope-zuul-search.nvim]: https://github.com/birnevogel11/telescope-zuul-search.nvim
