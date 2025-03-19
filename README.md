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

## Config file format

The program can search zuul configs cross multiple projects with config file in
`~/.config/zuul-ls/config.yaml`. The file format is:

```yaml
# The search scope is tenant. It supports multiple tenants
tenant:
  # The name of the tenant. Only for debug
  example_tenant_name:
    # The base directory of the tenant. Zuul-ls searches all jobs,
    # project-templates in the directory
    # It searches ansible roles in `<base_dir>/zuul-shared` and
    # `<base_dir>/zuul-trusted`
    base_dir: ~/code/ci/tenant
    # The extra base directory of the tenant. It searches all jobs,
    # project-templates in the directory
    # It searches ansible roles in `<extra_base_dir>/zuul-shared` and
    # `<extra_base_dir>/zuul-trusted`
    extra_base_dir:
      - ~/ci/common-repo
      - ~/ci/common-repo2
    # The extra role directory of the tenant. It searches ansible roles in
    # `<extra_role_dir>/roles` directory.
    extra_role_dir:
      - ~/ci/common-rule-repo
      - ~/ci/common-rule-repo2
```

### Neovim

1. Assume `neovim/nvim-lspconfig` is installed with lazy.nvim. Add the if code
   block in config function

   ```lua
   return {
     {
       "neovim/nvim-lspconfig",
       dependencies = {
         { "nvim-lua/plenary.nvim" },
       },
       config = function()
         local configs = require("lspconfig.configs")
         local lspconfig = require("lspconfig")
         if not configs.zuul_ls then
           local util = require("lspconfig.util")
           configs.zuul_ls = {
             default_config = {
               cmd = { "zuul-ls" },
               filetypes = { "yaml", "yaml.ansible" },
               root_dir = util.find_git_ancestor,
               single_file_support = true,
               settings = {},
             },
             docs = {
               description = [[ zuul language server ]],
               default_config = {
                 root_dir = [[ util.find_git_ancestor ]],
               },
             },
           }
         end

       end,
     },
   }
   ```

2. Add lsp config

    ```lua
    require("lspconfig").zuul_ls.setup(...)
    ```

3. Install [telescope-zuul-search.nvim][telescope-zuul-search.nvim]

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
