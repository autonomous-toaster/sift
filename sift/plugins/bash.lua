-- bash.lua — default fallback plugin (priority -1000)
-- Calls sift.exec() with the command, returns raw output.
return {
    name = "__default__",
    priority = -1000,
    pattern = "__default__",

    execute = function(ctx, args, stdin)
        -- Build full command from context command + args
        local parts = {ctx.command}
        for i = 1, #args do
            parts[#parts + 1] = args[i]
        end
        local cmd = table.concat(parts, " ")
        local output, stderr, exit_code = sift.exec(cmd)
        -- Combine stdout and stderr for output
        local combined = output .. stderr
        return {
            status = "handled",
            output = combined,
            exit_code = exit_code
        }
    end
}
