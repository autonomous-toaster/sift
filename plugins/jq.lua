-- jq.lua — intercept jq commands, use sift.jq.query() in-process, compress output
-- Supports: -r (raw), -c (compact), -n (null input)
-- Unknown flags → fall through to real jq
-- No piped stdin and no -n → fall through to real jq (file arguments)

return {
    name = "jq",
    priority = 0,
    pattern = "jq",

    execute = function(ctx, args, stdin)
        -- Parse jq arguments declaratively
        local parsed, err = sift.args.parse(args, {
            flags = {
                r = { "-r", "--raw-output" },
                c = { "-c", "--compact-output" },
                n = { "-n", "--null-input" },
                f = { "-f", "--from-file", type = "str" },
            },
            args = {
                { name = "filter", required = true },
            },
            opts = { allow_unknown = false },
        })
        if not parsed then
            if err then return { status = "error", output = err } end
            return { status = "passthrough" }
        end

        -- -f (from-file): would need to read filter from file → fall through
        if parsed.f then
            return { status = "passthrough" }
        end

        -- Determine input source
        local input
        if parsed.n then
            -- -n: null input, no stdin needed
            input = "null"
        elseif stdin ~= nil then
            -- Piped stdin: read all via tostring
            input = tostring(stdin)
            if input == nil or input == "" then
                return { status = "passthrough" }
            end
        else
            -- No stdin and no -n → jq reads files → fall through
            return { status = "passthrough" }
        end

        -- Apply jq filter in-process
        local ok, result = pcall(sift.jq.query, ctx, input, parsed.filter)
        if not ok then
            -- Filter error → fall through to real jq
            return { status = "passthrough" }
        end

        -- Handle -r (raw output): decode JSON array, extract values as strings
        if parsed.r then
            local ok2, decoded = pcall(sift.json.decode, ctx, result)
            if not ok2 then
                return { status = "passthrough" }
            end
            -- decoded is a Lua table (array of values)
            local lines = {}
            for _, v in ipairs(decoded) do
                lines[#lines + 1] = tostring(v)
            end
            local raw_output = table.concat(lines, "\n")
            -- Pass through shortest() — non-JSON input returns unchanged
            local compressed = sift.json.shortest(ctx, raw_output, { toon = true })
            sift.nudge(ctx, "compressed output. raw: command " .. ctx.original_cmd)
            return {
                status = "handled",
                output = compressed,
                exit_code = 0,
                raw_bytes = #input
            }
        end

        -- Default (including -c): compress with shortest format
        local compressed = sift.json.shortest(ctx, result, { toon = true })
        sift.nudge(ctx, "compressed output. raw: command " .. ctx.original_cmd)
        return {
            status = "handled",
            output = compressed,
            exit_code = 0,
            raw_bytes = #input
        }
    end
}
