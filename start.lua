local client = vim.lsp.start_client({
    name = "grug-ls",
    cmd = { "./target/debug/grug-ls" }
})

if not client then
    vim.notify("bad client")
else
    vim.cmd("echo 'grug good'")
end


