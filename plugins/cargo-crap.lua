-- cargo-crap.lua — CRAP complexity check optimization (priority 0)
-- On success: compact "✓ crap passed"
-- On failure: filtered functions above threshold
-- Adds --format json for structured output when no format flag is specified.
-- Uses sift.args.parse() for declarative argument parsing.
-- Respects user's explicit format choice.

return {
    name = "cargo-crap",
    priority = 0,
    pattern = "cargo crap",

    execute = function(ctx, args, stdin)
        -- Parse args declaratively
        local parsed, err = sift.args.parse(args, {
            flags = {
                threshold = { "--threshold", type = "int" },
                format = { "--format" },
                min = { "--min", type = "int" },
            },
            opts = { allow_unknown = true },
        })
        if not parsed then
            if err then return { status = "error", output = err } end
            return { status = "passthrough" }
        end

        local threshold = parsed.threshold or 30
        local has_format = parsed.format ~= nil

        -- Build command: args[1] is the subcommand ("crap"), rest are flags
        -- Insert --format json after the subcommand when no format is specified
        local cmd
        if has_format then
            cmd = "cargo " .. table.concat(args, " ")
        else
            cmd = "cargo " .. args[1] .. " --format json"
            for i = 2, #args do
                cmd = cmd .. " " .. args[i]
            end
        end

        local output, stderr, exit_code = sift.exec(ctx, cmd, { silent = true })
        local combined = output .. stderr

        -- Parse JSON output
        local ok, data = pcall(sift.json.decode, ctx, output)
        if not ok or not data then
            return {
                status = "handled",
                output = combined,
                exit_code = exit_code,
                raw_bytes = #combined
            }
        end

        -- cargo-crap JSON: { entries: [{ function, file, line, crap, cyclomatic, coverage, crate }] }
        local entries = data.entries or {}

        -- Filter functions above threshold
        local crappy = {}
        for i = 1, #entries do
            local e = entries[i]
            if e.crap and e.crap > threshold then
                crappy[#crappy + 1] = e
            end
        end

        if #crappy == 0 then
            return {
                status = "handled",
                output = "✓ crap passed\n",
                exit_code = 0,
                raw_bytes = #combined
            }
        end

        -- Crappy functions found: store full output for re-read
        sift.store(ctx, combined, "cargo_crap")

        -- Build compact failure output
        local result = string.format("%d function(s) exceed CRAP threshold %d:\n", #crappy, threshold)
        for i = 1, #crappy do
            local e = crappy[i]
            local short_file = e.file:match("/([^/]+)$") or e.file
            result = result .. string.format(
                "  CRAP=%.0f  cyclomatic=%.0f  coverage=%.0f%%  %s  %s:%d\n",
                e.crap, e.cyclomatic or 0, e.coverage or 0, e["function"], short_file, e.line
            )
        end

        return {
            status = "handled",
            output = result,
            exit_code = exit_code,
            raw_bytes = #combined
        }
    end
}
