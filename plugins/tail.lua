-- tail.lua — intercept tail -n <count> <path> (priority 0)
-- Passthrough for -c (byte count) and other flags.

-- Parse tail -n <count> <path> or -<count> <path>
local function parse_tail(args)
    local count
    local path
    local i = 1

    while i <= #args do
        local arg = args[i]
        if arg == "-n" and i < #args then
            i = i + 1
            count = tonumber(args[i])
        elseif arg:match("^-%d+$") then
            count = tonumber(arg:sub(2))
        elseif arg:sub(1, 1) == "-" and arg ~= "-n" then
            return nil
        elseif not path then
            path = arg
        end
        i = i + 1
    end

    if not count or not path then
        return nil
    end
    return { path = path, count = count }
end

return {
    name = "tail",
    priority = 0,
    pattern = "tail",

    execute = function(ctx, args, stdin)
        local parsed = parse_tail(args)
        if not parsed then
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
        local range_start = math.max(1, total_lines - parsed.count + 1)
        local range_end = total_lines

        -- Check cache
        if sift.cache.has_file(ctx, hash) then
            local display_name = path:match("([^/]+)$") or parsed.path
            return {
                status = "unchanged",
                message = string.format("[sift] %s lines %d-%d unchanged (cached)\n      (bypass if stale: command tail -n %d %s)", display_name, range_start, range_end, parsed.count, path)
            }
        end
        if sift.cache.has_range(ctx, hash, range_start, range_end) then
            local display_name = path:match("([^/]+)$") or parsed.path
            return {
                status = "unchanged",
                message = string.format("[sift] %s lines %d-%d unchanged (cached)\n      (bypass if stale: command tail -n %d %s)", display_name, range_start, range_end, parsed.count, path)
            }
        end

        -- Cache miss
        sift.cache.store_content(ctx, hash, content)
        sift.cache.add_range(ctx, hash, range_start, range_end)

        local sliced = sift.str.slice_text(ctx, content, range_start, range_end)
        return {
            status = "handled",
            output = sliced,
            exit_code = 0
        }
    end
}
