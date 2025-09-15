# Grug-LS
> A Language server for grug

## Installation
First clone the repo, and cd into it
```bash
git clone [git@github.com:MyNameIsTrez/grug.git](https://github.com/xijnim/grug-ls)
cd grug-ls
```

Then create a file called "log_path" and add to it the place you wanna store the log files.
Example:
```bash
echo "$HOME/Projects/grug-ls > log_path
```

Then build the project
```bash
cargo build
```

After that, the process is editor specific on how to add a language server
