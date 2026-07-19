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
        -- Track flags that take a value to avoid false positives
        local value_flags = {
            ["-m"] = true, ["--message"] = true,
            ["-F"] = true, ["--file"] = true,
            ["-C"] = true, ["--reuse-message"] = true,
            ["-c"] = true, ["--reedit-message"] = true,
            ["-t"] = true, ["--template"] = true,
            ["--fixup"] = true, ["--squash"] = true,
        }

        local i = 1
        while i <= #args do
            local arg = args[i]

            -- Check for -n or --no-verify
            if arg == "-n" or arg == "--no-verify" then
                sift.nudge(ctx, "git commit --no-verify (-n) is forbidden: hooks must run")
                return {
                    status = "handled",
                    output = "",
                    exit_code = 1
                }
            end

            -- Skip value for flags that take a value
            if value_flags[arg] then
                i = i + 2  -- skip flag + its value
            elseif arg:match("^--message=") or arg:match("^--file=")
                or arg:match("^--reuse-message=") or arg:match("^--reedit-message=")
                or arg:match("^--fixup=") or arg:match("^--squash=")
                or arg:match("^--template=") or arg:match("^-S") then
                i = i + 1  -- value is part of the flag (--key=val or -Skey)
            else
                i = i + 1
            end
        end

        -- No -n/--no-verify found: passthrough runs directly in bash
        -- (bypasses all plugins, does NOT go to rtk)
        return { status = "passthrough" }
    end
}
