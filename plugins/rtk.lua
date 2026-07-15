-- rtk.lua — Delegate commands to rtk (priority 0)
-- Matches docker, podman, kubectl, oc, gh, glab, curl, wget, npm, pnpm, pip, uv
-- Delegates to rtk binary for compact output
return {
    name = "rtk",
    priority = 0,
    pattern = {"docker", "podman", "kubectl", "oc", "gh", "glab", "curl", "wget", "npm", "pnpm", "pip", "uv"},

    execute = function(ctx, args, stdin)
        -- Build original command
        local parts = {ctx.command}
        for i = 1, #args do
            parts[#parts + 1] = args[i]
        end
        local original_cmd = table.concat(parts, " ")

        -- Delegate to rtk
        local rtk_cmd = "rtk " .. original_cmd
        local output, stderr, exit_code = sift.exec(ctx, rtk_cmd)

        if exit_code == 0 then
            return {
                status = "handled",
                output = output,
                exit_code = 0
            }
        end

        -- If rtk fails, fall through to passthrough (bash will run the original command)
        return { status = "passthrough" }
    end
}
