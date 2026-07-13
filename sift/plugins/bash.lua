-- bash.lua — default fallback plugin (priority -1000)
-- Calls sift.exec() with the command, returns raw output.
return {
    name = "__default__",
    priority = -1000,
    pattern = "",

    execute = function(ctx, args, stdin)
        local cmd = table.concat(args, " ")
        local output, exit_code = sift.exec(cmd)
        return {
            status = "handled",
            output = output,
            exit_code = exit_code
        }
    end
}
