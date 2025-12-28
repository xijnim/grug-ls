# Grug-LS
> A Language server for grug

## Installation
You can install it from cargo:
```bash
cargo install grug-ls
```

Or you can build it from source
```bash
git clone git@github.com:xijnim/grug-ls.git
cd grug-ls
```

Then build the project
```bash
cargo build
```

After that, the process is editor specific on how to add a language server.

### Neovim
Using lspconfig, you can simply add this code to your init.lua
```lua
local util = require('lspconfig.util')
require('lspconfig.configs').grug = {
      default_config = {
        cmd = { 'grug-ls' },
        filetypes = { 'grug' },
        single_file_support = false,
        root_dir = function(fname)
          return util.search_ancestors(fname, function(path)
            if util.path.is_file(util.path.join(path, 'mod_api.json')) then
              return path
            end
          end)
        end,
      },
      docs = {
        description = [[
    https://github.com/xijnim/grug-ls

    Language server for Grug.
        ]],
      },
}
```
Then you can setup the lsp like you would any other:
```lua
require("lspconfig").grug.setup({
    on_attach = on_attach,
    capabilities = capabilities,
})
```

### VSCode
Go install the vscode extension called "grug"

## Development
For debugging the LSP, go to the project's directory and run:
```bash
ln -s "$(pwd)/target/debug/grug-ls" ~/.local/bin/grug-ls
```
That way, you can use your text editor and the Language Client will use the one you get using `cargo build`

For analyzing logs, you can check /tmp/grug-ls-logs.json
