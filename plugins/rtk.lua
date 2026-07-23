-- rtk.lua — Delegate commands to rtk (priority 0)
-- Uses wildcard pattern "*" to catch all commands not handled by specific plugins.
-- Tries `rtk <command>`, falls through to bash on failure.
return {
    name = "rtk",
    priority = 0,
    pattern = "*",

    execute = function(ctx, args, stdin)
        local parts = {ctx.command}
        for i = 1, #args do
            parts[#parts + 1] = sift.str.shell_quote(ctx, args[i])
        end
        local cmd = table.concat(parts, " ")

        local output, stderr, exit_code = sift.exec(ctx, "rtk " .. cmd)
        if exit_code == 0 then
            return {
                status = "handled",
                output = output .. stderr,
                exit_code = 0
            }
        end

        return { status = "passthrough" }
    end
}
