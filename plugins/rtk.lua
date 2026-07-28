-- rtk.lua — Delegate commands to rtk (priority 0)
-- Uses specific command patterns to match only commands rtk handles.
-- Tries `rtk <command>`, falls through to bash on failure.
return {
    name = "rtk",
    priority = 0,
    pattern = { "ls", "tree", "read", "git", "gh", "glab", "aws", "psql",
                "pnpm", "err", "test", "json", "deps", "env", "find", "diff",
                "log", "dotnet", "docker", "kubectl", "summary",
                "init", "wget", "wc", "gain", "cc-economics", "config",
                "jest", "vitest", "prisma", "tsc", "next", "lint", "smart" },
    append_prompt = "Output from git, docker, ls and other commands is compressed by rtk. " ..
        "Follow [nudge] command: hints to get raw output.",

    execute = function(ctx, args, stdin)
        local parts = {ctx.command}
        for i = 1, #args do
            parts[#parts + 1] = sift.str.shell_quote(ctx, args[i])
        end
        local cmd = table.concat(parts, " ")

        local output, stderr, exit_code = sift.exec(ctx, "rtk " .. cmd)
        if exit_code == 0 then
            sift.nudge(ctx, "command: '" .. ctx.original_cmd .. "'")
            return {
                status = "handled",
                output = output .. stderr,
                exit_code = 0
            }
        end

        return { status = "passthrough" }
    end
}
