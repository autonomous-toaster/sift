-- head.lua — intercept head -n <count> <path> (priority 0)
-- Passthrough for -c (byte count) and other flags.
local function split_lines(text)
    local lines = {}
    for line in text:gmatch("([^\n]*)\n?") do
        table.insert(lines, line)
    end
    if text:sub(-1) == "\n" then
        table.insert(lines, "")
    end
    return lines
end

local function slice_text(text, start_line, end_line)
    local lines = split_lines(text)
    local clamped_end = math.min(end_line, #lines)
    if start_line > #lines then
        return ""
    end
    local result = {}
    for i = start_line, clamped_end do
        table.insert(result, lines[i])
    end
    return table.concat(result, "\n")
end

-- Parse head -n <count> <path> or -<count> <path>
local function parse_head(args)
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
            -- Other flags like -c: passthrough
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
    name = "head",
    priority = 0,
    pattern = "head",

    execute = function(ctx, args, stdin)
        local parsed = parse_head(args)
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
        local total_lines = #split_lines(content)
        local range_end = math.min(parsed.count, total_lines)

        -- Check cache
        if sift.cache.has_file(ctx, hash) then
            sift.nudge(ctx, "bypass: 'command head -n " .. range_end .. " " .. parsed.path .. "'")
            return {
                status = "unchanged",
                message = string.format("[sift] %s lines 1-%d unchanged", parsed.path, range_end)
            }
        end
        if sift.cache.has_range(ctx, hash, 1, range_end) then
            sift.nudge(ctx, "bypass: 'command head -n " .. range_end .. " " .. parsed.path .. "'")
            return {
                status = "unchanged",
                message = string.format("[sift] %s lines 1-%d unchanged", parsed.path, range_end)
            }
        end

        -- Cache miss
        sift.cache.store_content(ctx, hash, content)
        sift.cache.add_range(ctx, hash, 1, range_end)

        local sliced = slice_text(content, 1, range_end)
        return {
            status = "handled",
            output = sliced,
            exit_code = 0
        }
    end
}
