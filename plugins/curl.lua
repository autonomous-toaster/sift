-- curl.lua — JSON response optimizer
-- Detects JSON responses via -w "%{content_type}", compresses with sift.json.shortest().
-- If -v/--verbose explicitly requested, runs as-is with full output.
-- Always propagates curl exit code.

return {
    name = "curl",
    priority = 0,
    pattern = "curl",

    execute = function(ctx, args, stdin)
        -- Check if -v or --verbose was explicitly requested
        local has_verbose = false
        local has_write_out = false
        for _, arg in ipairs(args) do
            if arg == "-v" or arg == "--verbose" then
                has_verbose = true
            elseif arg == "-w" or arg == "--write-out" then
                has_write_out = true
            end
        end

        if has_verbose or has_write_out then
            -- Agent asked for verbose or custom -w: run as-is, return full output
            local parts = { "curl" }
            for _, arg in ipairs(args) do
                parts[#parts + 1] = arg
            end
            local cmd = table.concat(parts, " ")
            local output, stderr, exit_code = sift.exec(ctx, cmd)
            return {
                status = "handled",
                output = output,
                exit_code = exit_code,
                streamed = true
            }
        end

        -- No -v: add -w to detect content type
        -- Use single quotes to prevent bash from eating \n
        local new_args = { "-w", "'\\n%{content_type}'" }
        for _, arg in ipairs(args) do
            new_args[#new_args + 1] = arg
        end
        local cmd = "curl " .. table.concat(new_args, " ")
        local output, stderr, exit_code = sift.exec(ctx, cmd, { silent = true })

        if exit_code ~= 0 then
            return {
                status = "handled",
                output = output,
                exit_code = exit_code
            }
        end

        -- Parse: last line is content type, everything before is body
        -- Trim trailing newline from -w output
        local trimmed = output:gsub("\n$", "")
        local body, content_type = trimmed:match("^(.*)\n([^\n]*)$")

        if body and content_type and content_type:find("json") then
            -- JSON detected: compress (shortest stores raw + nudges automatically)
            local compressed = sift.json.shortest(ctx, body, { toon = true })
            return {
                status = "handled",
                output = compressed,
                exit_code = exit_code,
                raw_bytes = #body
            }
        end

        -- Not JSON or empty body: return as-is
        return {
            status = "handled",
            output = body or output,
            exit_code = exit_code
        }
    end
}
