-- cargo-llvm-cov.lua — coverage report optimization (priority 0)
-- On success: compact "XX.X% coverage"
-- On failure: show errors
-- Adds --json for structured output when no format flag is specified.
-- Uses sift.jq.query() to extract coverage percentage (no full JSON tree in Lua).
-- Respects user's explicit format choice.

return {
    name = "cargo-llvm-cov",
    priority = 0,
    pattern = "cargo llvm-cov",

    execute = function(ctx, args, stdin)
        -- Parse args declaratively to detect format flags
        local parsed, err = sift.args.parse(args, {
            flags = {
                json = { "--json", type = "boolean" },
                lcov = { "--lcov", type = "boolean" },
                cobertura = { "--cobertura", type = "boolean" },
                text = { "--text", type = "boolean" },
                html = { "--html", type = "boolean" },
                codecov = { "--codecov", type = "boolean" },
            },
            opts = { allow_unknown = true },
        })
        if not parsed then
            if err then return { status = "error", output = err } end
            return { status = "passthrough" }
        end

        -- Check if user already specified an output format
        local has_format = parsed.json or parsed.lcov or parsed.cobertura
                        or parsed.text or parsed.html or parsed.codecov

        -- Build command: args[1] is the subcommand ("llvm-cov"), rest are flags
        -- Insert --json after the subcommand when no format is specified
        local cmd
        if has_format then
            cmd = "cargo " .. table.concat(args, " ")
        else
            cmd = "cargo " .. args[1] .. " --json"
            for i = 2, #args do
                cmd = cmd .. " " .. args[i]
            end
        end

        local output, stderr, exit_code = sift.exec(ctx, cmd, { silent = true })
        local combined = output .. stderr

        if exit_code ~= 0 then
            sift.store(ctx, combined, "cargo_llvm_cov")
            return {
                status = "handled",
                output = combined,
                exit_code = exit_code,
                raw_bytes = #combined
            }
        end

        -- Extract coverage percentage via jq (no full JSON tree in Lua)
        -- Format: { data: [{ totals: { lines: { percent: 77.5 }, regions: { ... }, functions: { ... } } }] }
        local pct_str = sift.jq.query(ctx, output, '.data[0].totals.lines.percent')
        local pct = tonumber(pct_str:match("[%d.]+"))

        if not pct then
            -- Fallback to regions
            pct_str = sift.jq.query(ctx, output, '.data[0].totals.regions.percent')
            pct = tonumber(pct_str:match("[%d.]+"))
        end

        if pct then
            local summary = string.format("%.1f%% coverage\n", pct)
            return {
                status = "handled",
                output = summary,
                exit_code = 0,
                raw_bytes = #combined
            }
        end

        -- Fallback: return raw output
        return {
            status = "handled",
            output = output,
            exit_code = 0,
            raw_bytes = #combined
        }
    end
}
