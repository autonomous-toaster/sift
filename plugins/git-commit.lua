-- git-commit.lua — forbid -n/--no-verify on git commit
-- Returns non-zero exit code with nudge when hooks are bypassed.
-- When -n is absent, passthrough runs the command directly in bash.
-- Other git commands (status, push, etc.) don't match "git commit" pattern
-- and fall through to the wildcard plugin (rtk.lua) or default (bash.lua).

return {
    name = "git-commit",
    priority = 0,
    pattern = "git commit",

    execute = function(ctx, args, stdin)
        -- Parse args: scan for -n/--no-verify, allow all other git flags
        local parsed, err = sift.args.parse(args, {
            flags = {
                n = { "-n" },
                ["no-verify"] = { "--no-verify" },
            },
            opts = { allow_unknown = true },
        })
        if not parsed then
            if err then return { status = "error", output = err } end
            return { status = "passthrough" }
        end

        if parsed.n or parsed["no-verify"] then
            sift.nudge(ctx, "git commit --no-verify (-n) is forbidden: hooks must run")
            return {
                status = "handled",
                output = "",
                exit_code = 1
            }
        end

        -- No -n/--no-verify found: passthrough runs directly in bash
        return { status = "passthrough" }
    end
}
