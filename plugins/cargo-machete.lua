-- cargo-machete.lua — unused dependency check (priority 0)
-- On success: compact "✓ machete passed"
-- On failure: show unused deps list as-is
-- Falls through to rtk if args can't be handled.

return {
    name = "cargo-machete",
    priority = 0,
    pattern = "cargo machete",

    execute = function(ctx, args, stdin)
        -- Build command: args already contain the subcommand (e.g., {"machete", "--fix"})
        local cmd = "cargo " .. table.concat(args, " ")

        local output, stderr, exit_code = sift.exec(ctx, cmd, { silent = true })
        local combined = output .. stderr

        if exit_code == 0 then
            return {
                status = "handled",
                output = "✓ machete passed\n",
                exit_code = 0,
                raw_bytes = #combined
            }
        end

        -- Failure: store output for re-read, show unused deps list
        sift.store(ctx, combined, "cargo_machete")

        -- Failure: show unused deps list (already compact)
        return {
            status = "handled",
            output = combined,
            exit_code = exit_code,
            raw_bytes = #combined
        }
    end
}
