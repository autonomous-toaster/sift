-- sed.lua — intercept sed -n range reads (priority 0)
-- Only intercepts: sed -n '<start>,<end>p' <path>
-- Passthrough for substitutions, -i, patterns, and other operations.

return {
    name = "sed",
    priority = 0,
    pattern = "sed",

    execute = function(ctx, args, stdin)
        local parsed, err = sift.args.parse(args, {
            flags = { n = { "-n" } },
            args = {
                { name = "expression", required = true },
                { name = "path", required = true },
            },
            opts = { allow_unknown = false },
        })
        if not parsed then
            if err then return { status = "error", output = err } end
            return { status = "passthrough" }
        end

        -- Parse sed expression: <start>,<end>p or <start>p
        local expr = parsed.expression
        -- Strip surrounding quotes
        expr = expr:match("^['\"](.*)['\"]$") or expr

        local start, end_line = expr:match("^(%d+),(%d+)p$")
        local range
        if start then
            range = { start = tonumber(start), end_line = tonumber(end_line) }
        else
            local single = expr:match("^(%d+)p$")
            if single then
                range = { start = tonumber(single), end_line = tonumber(single) }
            end
        end

        if not range then
            return { status = "passthrough" }
        end

        local path = parsed.path
        if path:sub(1, 1) ~= "/" then
            path = ctx.cwd .. "/" .. path
        end

        local content = sift.fs.read(ctx, path)
        if content == nil then
            return { status = "passthrough" }
        end

        local stat = sift.fs.stat(ctx, path)
        local hash = sift.hash.sha256(ctx, content)
        local total_lines = #sift.str.split_lines(ctx, content)
        local range_end = math.min(range.end_line, total_lines)

        -- Check cache
        if sift.cache.has_file(ctx, hash) then
            local display_name = path:match("([^/]+)$") or range.path
            return {
                status = "unchanged",
                message = string.format("[sift] %s lines %d-%d unchanged (cached)\n      (bypass if stale: command sed -n '%d,%dp' %s)", display_name, range.start, range_end, range.start, range_end, path),
                raw_bytes = stat.size
            }
        end
        if sift.cache.has_range(ctx, hash, range.start, range_end) then
            local display_name = path:match("([^/]+)$") or range.path
            return {
                status = "unchanged",
                message = string.format("[sift] %s lines %d-%d unchanged (cached)\n      (bypass if stale: command sed -n '%d,%dp' %s)", display_name, range.start, range_end, range.start, range_end, path),
                raw_bytes = stat.size
            }
        end

        -- Cache miss: store content and mark range
        sift.cache.store_content(ctx, hash, content)
        sift.cache.add_range(ctx, hash, range.start, range_end)

        local sliced = sift.str.slice_text(ctx, content, range.start, range_end)
        return {
            status = "handled",
            output = sliced,
            exit_code = 0,
            raw_bytes = stat.size
        }
    end
}
