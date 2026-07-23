-- bash.lua — default fallback plugin (priority -1000)
-- Calls sift.exec() with the command, returns raw output.
return {
    name = "__default__",
    priority = -1000,
    pattern = "__default__",

    execute = function(ctx, args, stdin)
        -- Build full command from context command + args, shell-quoting each arg
        local parts = {ctx.command}
        for i = 1, #args do
            parts[#parts + 1] = sift.str.shell_quote(ctx, args[i])
        end
        local cmd = table.concat(parts, " ")
        local output, stderr, exit_code = sift.exec(ctx, cmd)
        return {
            status = "handled",
            output = output,
            exit_code = exit_code,
            streamed = true
        }
    end
}
