-- bash.lua — default fallback shell plugin (priority -1000)
-- Uses ctx.original_cmd to preserve shell semantics (variable expansion,
-- command substitution, heredocs). Users can override this plugin by creating
-- ~/.config/sift/plugins/shell.lua with pattern = "__default__".
return {
    name = "__default__",
    priority = -1000,
    pattern = "__default__",

    execute = function(ctx, args, stdin)
        -- Use original_cmd to preserve shell semantics
        local cmd = ctx.original_cmd
        if cmd == nil or cmd == "" then
            -- Fallback: reconstruct from command + args
            local parts = {ctx.command}
            for i = 1, #args do
                parts[#parts + 1] = sift.str.shell_quote(ctx, args[i])
            end
            cmd = table.concat(parts, " ")
        end
        local output, stderr, exit_code = sift.exec(ctx, cmd)
        return {
            status = "handled",
            output = output,
            exit_code = exit_code,
            streamed = true
        }
    end
}
