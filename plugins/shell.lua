-- shell.lua — User override for the default bash plugin (priority -500)
-- Redacts all command output through sift.ext.scred when available.
-- Falls through to the built-in __default__ plugin when scred is not available.
return {
    name = "__default__",
    priority = -500,
    pattern = "__default__",

    execute = function(ctx, args, stdin)
        local cmd = ctx.original_cmd
        if cmd == nil or cmd == "" then
            local parts = {ctx.command}
            for i = 1, #args do
                parts[#parts + 1] = sift.str.shell_quote(ctx, args[i])
            end
            cmd = table.concat(parts, " ")
        end

        -- Check if scred extension is available
        if sift.ext ~= nil and sift.ext.scred ~= nil then
            local transform, finalize = sift.ext.scred.create_transform()
            local out, stderr, code = sift.exec(ctx, cmd, {
                transform = transform,
                silent = true
            })
            local final, stats = finalize()
            return {
                status = "handled",
                output = out .. final,
                exit_code = code,
                streamed = true
            }
        end

        -- scred not available — execute normally
        local output, stderr, exit_code = sift.exec(ctx, cmd)
        return {
            status = "handled",
            output = output,
            exit_code = exit_code,
            streamed = true
        }
    end
}
