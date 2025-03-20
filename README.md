# zuul-ls (Zuul Language Server) + zuul-search

> [!WARNING]
> The project is in ALPHA PHASE. It does not guarantee any compatibility.

zuul-ls and zuul-search aim to provide a more efficient way to search and edit
[Zuul][zuul] CI config.

zuul-ls provides a small LSP with go-to-definition, auto-complete and
workspace symbols method for jobs, variables, playbooks, project-templates and
roles.

zuul-search can search jobs, project-templates or a job's variables or job
hierarchy.

## Required dependencies

zuul-ls does not have any dependencies. zuul-search is wrapped by a `zs` script.
It requires [fzf][fzf] and [bat][bat].

## Installation

The project is tested with neovim and VSCode. Need clone the repo and
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

### VSCode

It's conflict to existing yaml language server(e.g. Ansible VSCode extension). Please disable them to avoid any conflicts.

You can build the VSCode extension manually with

```bash
# Install the vscode package tool
npm install -g @vscode/vsce

cd editors/vscode/zuul-ls/
# Install required packages
npm install
# Build the extension
npm run compile
vsce package

# Install the extension
code --install-extension ./zuul-ls-*.vsix
```

## zuul-search

zuul-search can search jobs, variables, project-templates, job hierarchy, playbooks
from command line. `zuul-search` binary does not provide any search function.
It's wrapped with fzf and bat to search and preview the result.

There is also a way to integrate it into Neovim with telescope or Snacks.picker.

Please run the command in the repo folders:

```
$ zs --help
zs - zuul search - Search zuul config with zuul-search, fzf and bat

Command:
  zs <search> [job_name]
    j,jobs              - Search all jobs
    h,hierarchy         - Search the job hierarchy of a job
    v,vars              - Search job variables of a job
    wv,workdir-vars     - Search variables in cwd
    p,playbooks         - Search playbooks of a job
    t,project-templates - Search all project-templates
    help                - Show the help

Example:
    zs j          - Search all jobs
    zs job        - Same as 'zs j'

    zs h          - Search the job name first, search the job hierarchy of the job name
    zs hierarchy  - Same as 'zs h'

    zs h zuul-job - Search the job hierarhcy of 'zuul-job'
```

### Integrate with Snacks.Picker

- Copy the file `./editors/neovim/zuul.lua` into your config and add the code

```lua
-- Fill the real module path
local Zuul = require("<module>.zuul")

-- In your snacks config
return
  {
    "folke/snacks.nvim",
    opts = {
      picker = {},  -- Enable the picker
    }
    -- ...
    keys = {
      { "<leader>zj", function() Zuul.job() end, desc = "Zuul jobs", },
      { "<leader>zr", function() Zuul.role() end, desc = "Zuul Roles", },
      { "<leader>zt", function() Zuul.project_template() end, desc = "Zuul Project Templates", },
      { "<leader>zv", function() Zuul.workspace_var() end, desc = "Zuul workspace vars", },
      { "<leader>zh", function() Zuul.job_hierarchy() end, desc = "Zuul Job Hierarchy", mode = { "v", "x"  } },
      { "<leader>zp", function() Zuul.job_playbooks() end, desc = "Zuul Job Playbooks", mode = { "v", "x"  } },
    }
  }
```

### Integrate with telescope.nvim

- Install [telescope-zuul-search.nvim][telescope-zuul-search.nvim]

## Personal Note

The reason to create the project is that I would like to learn Rust and it's
useful for my daily job.  It's always welcome to send patches for any reason.

I know tower-lsp is a async program but the code has a lot of blocking IO call.
The most important reason is that I am unfamiliar with async in Rust. Maybe we
can change/rewrite these parts in the future.


## Related projects

- [alexander-scott/zuul_job_browser](https://github.com/alexander-scott/zuul_job_browser)

## Acknowledgements

- [IWANABETHATGUY/tower-lsp-boilerplate][tower-lsp-boilerplate].
   - Save me a lot of time to learn how to write a LS from scratch.
- [LaBatata101/sith-language-server][sith-language-server]
    - Learn how to package the vscode extension from it


[zuul]: https://zuul-ci.org/
[tower-lsp-boilerplate]: https://github.com/IWANABETHATGUY/tower-lsp-boilerplate/
[fzf]: https://github.com/junegunn/fzf
[bat]: https://github.com/sharkdp/bat
[telescope-zuul-search.nvim]: https://github.com/birnevogel11/telescope-zuul-search.nvim
[sith-language-server]: https://github.com/LaBatata101/sith-language-server
