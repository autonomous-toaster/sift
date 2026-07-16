-- sed.lua — intercept sed -n range reads (priority 0)
-- Only intercepts: sed -n '<start>,<end>p' <path>
-- Passthrough for substitutions, -i, patterns, and other operations.
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

-- Parse sed -n '<start>,<end>p' <path>
-- Returns { path, start, end } or nil if not a range read
local function parse_sed_range(args)
    local has_n = false
    local expr
    local path
    local i = 1

    while i <= #args do
        local arg = args[i]
        if arg == "-n" then
            has_n = true
        elseif arg:sub(1, 1) == "-" and arg ~= "-n" then
            -- Other flags: passthrough
            return nil
        elseif not expr then
            expr = arg
        elseif not path then
            path = arg
        end
        i = i + 1
    end

    if not has_n or not expr or not path then
        return nil
    end

    -- Strip surrounding quotes
    expr = expr:match("^['\"](.*)['\"]$") or expr

    -- Match <start>,<end>p or <start>p
    local start, end_line = expr:match("^(%d+),(%d+)p$")
    if start then
        return { path = path, start = tonumber(start), end_line = tonumber(end_line) }
    end
    local single = expr:match("^(%d+)p$")
    if single then
        return { path = path, start = tonumber(single), end_line = tonumber(single) }
    end

    return nil
end

return {
    name = "sed",
    priority = 0,
    pattern = "sed",

    execute = function(ctx, args, stdin)
        local range = parse_sed_range(args)
        if not range then
            return { status = "passthrough" }
        end

        local path = range.path
        if path:sub(1, 1) ~= "/" then
            path = ctx.cwd .. "/" .. path
        end

        local content = sift.fs.read(ctx, path)
        if content == nil then
            return { status = "passthrough" }
        end

        local hash = sift.hash.sha256(ctx, content)
        local total_lines = #split_lines(content)
        local range_end = math.min(range.end_line, total_lines)

        -- Check cache
        if sift.cache.has_file(ctx, hash) then
            sift.nudge(ctx, "bypass: 'command sed -n '" .. range.start .. "," .. range_end .. "p' " .. range.path .. "'")
            return {
                status = "unchanged",
                message = string.format("[sift] %s lines %d-%d unchanged", range.path, range.start, range_end)
            }
        end
        if sift.cache.has_range(ctx, hash, range.start, range_end) then
            sift.nudge(ctx, "bypass: 'command sed -n '" .. range.start .. "," .. range_end .. "p' " .. range.path .. "'")
            return {
                status = "unchanged",
                message = string.format("[sift] %s lines %d-%d unchanged", range.path, range.start, range_end)
            }
        end

        -- Cache miss: store content and mark range
        sift.cache.store_content(ctx, hash, content)
        sift.cache.add_range(ctx, hash, range.start, range_end)

        local sliced = slice_text(content, range.start, range_end)
        return {
            status = "handled",
            output = sliced,
            exit_code = 0
        }
    end
}
