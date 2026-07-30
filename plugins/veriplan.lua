-- veriplan.lua — Optimize veriplan check output for AI agents (priority 0)
-- Ensures --json output and compresses via sift.json.shortest() (compact JSON vs toon).
-- Falls through to the default plugin for non-check subcommands.
return {
    name = "veriplan",
    priority = 0,
    pattern = "veriplan",

    execute = function(ctx, args, stdin)
        -- Only optimize "check" subcommand
        if args[1] ~= "check" then
            return { status = "passthrough" }
        end

        -- Ensure --json flag is present (compact by default without --verbose)
        local has_json = false
        local has_format = false
        for _, arg in ipairs(args) do
            if arg == "--json" then has_json = true end
            if arg == "--format" then has_format = true end
        end

        local new_args = {}
        for _, arg in ipairs(args) do
            new_args[#new_args + 1] = arg
        end
        if not has_json and not has_format then
            new_args[#new_args + 1] = "--json"
        end

        local cmd = "veriplan " .. table.concat(new_args, " ")
        local output, stderr, exit_code = sift.exec(ctx, cmd, { silent = true })

        -- Always process output, even on non-zero exit (veriplan exits 2 for blockers)
        -- The output still contains useful information (blockers, warnings, etc.)
        local optimized = sift.json.shortest(ctx, output, { toon = true })
        return {
            status = "handled",
            output = optimized,
            exit_code = exit_code
        }
    end
}
