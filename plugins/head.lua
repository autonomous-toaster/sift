-- head.lua — intercept head -n <count> <path> (priority 0)
-- Passthrough for -c (byte count) and other flags.

return {
    name = "head",
    priority = 0,
    pattern = "head",

    execute = function(ctx, args, stdin)
        local parsed, err = sift.args.parse(args, {
            flags = { n = { "-n", type = "int" } },
            args  = { { name = "path", required = true } },
            opts  = { short_count = true, allow_unknown = false },
        })
        if not parsed then
            if err then return nil, err end
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

        local hash = sift.hash.sha256(ctx, content)
        local total_lines = #sift.str.split_lines(ctx, content)
        local range_end = math.min(parsed.n, total_lines)

        -- Check cache
        if sift.cache.has_file(ctx, hash) then
            local display_name = path:match("([^/]+)$") or parsed.path
            return {
                status = "unchanged",
                message = string.format("[sift] %s lines 1-%d unchanged (cached)\n      (bypass if stale: command head -n %d %s)", display_name, range_end, range_end, path)
            }
        end
        if sift.cache.has_range(ctx, hash, 1, range_end) then
            local display_name = path:match("([^/]+)$") or parsed.path
            return {
                status = "unchanged",
                message = string.format("[sift] %s lines 1-%d unchanged (cached)\n      (bypass if stale: command head -n %d %s)", display_name, range_end, range_end, path)
            }
        end

        -- Cache miss
        sift.cache.store_content(ctx, hash, content)
        sift.cache.add_range(ctx, hash, 1, range_end)

        local sliced = sift.str.slice_text(ctx, content, 1, range_end)
        return {
            status = "handled",
            output = sliced,
            exit_code = 0
        }
    end
}
