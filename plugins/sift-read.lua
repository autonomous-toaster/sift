-- sift-read.lua — file read plugin with caching, range support, and diff emission (priority 0)
-- Usage: sift-read <path> [<offset> [<limit>]]
--        sift-read --fresh <path> [<offset> [<limit>]]
-- Shares cache with cat.lua via file-based content store.
-- Returns "unchanged" on cache hit, unified diff on content change, or full content.
local SENSITIVE_PATTERNS = {
    "^%.env",
    "%.pem$",
    "%.key$",
    "%.p12$",
    "%.pfx$",
    "%.crt$",
    "%.cer$",
    "%.der$",
    "%.pk8$",
    "id_rsa$",
    "id_ed25519$",
    "%.npmrc$",
    "%.netrc$",
}

local function is_sensitive(path)
    local lower = path:lower()
    for _, pattern in ipairs(SENSITIVE_PATTERNS) do
        if lower:match(pattern) then
            return true
        end
    end
    return false
end

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

return {
    name = "sift-read",
    priority = 0,
    pattern = "sift-read",

    execute = function(ctx, args, stdin)
        -- Parse args: [--fresh] <path> [<offset> [<limit>]]
        local fresh = false
        local path
        local raw_path
        local offset
        local limit
        local arg_idx = 1

        if #args >= 1 and args[1] == "--fresh" then
            fresh = true
            arg_idx = 2
        end

        if #args >= arg_idx then
            path = args[arg_idx]
            raw_path = args[arg_idx]
            arg_idx = arg_idx + 1
        else
            return { status = "passthrough" }
        end

        if #args >= arg_idx then
            offset = tonumber(args[arg_idx])
            arg_idx = arg_idx + 1
        end

        if #args >= arg_idx then
            limit = tonumber(args[arg_idx])
        end

        -- Resolve path
        if path:sub(1, 1) ~= "/" then
            path = ctx.cwd .. "/" .. path
        end

        -- Sensitive path bypass
        if is_sensitive(path) then
            local content = sift.fs.read(ctx, path)
            if content == nil then
                return nil, "sift-read: " .. raw_path .. ": No such file or directory"
            end
            if offset and limit then
                local lines = split_lines(content)
                local start = offset
                local end_line = math.min(offset + limit - 1, #lines)
                content = slice_text(content, start, end_line)
            elseif offset then
                local lines = split_lines(content)
                content = slice_text(content, offset, #lines)
            end
            return { status = "handled", output = content, exit_code = 0 }
        end

        -- Read full file
        local content = sift.fs.read(ctx, path)
        if content == nil then
            return nil, "sift-read: " .. raw_path .. ": No such file or directory"
        end

        local total_lines = #split_lines(content)
        local hash = sift.hash.sha256(ctx, content)

        -- Compute range
        local range_start = offset or 1
        local range_end = limit and (offset or 1) + limit - 1 or total_lines
        range_end = math.min(range_end, total_lines)

        -- Check file-based cache (persists across invocations)
        -- Full hash satisfies any range; range keys satisfy themselves
        if not fresh then
            local cached = sift.cache.has_file(ctx, hash)
            if not cached and (offset or limit) then
                cached = sift.cache.has_range(ctx, hash, range_start, range_end)
            end
            if cached then
                sift.nudge(ctx, "bypass: 'sift-read --fresh " .. raw_path .. "'")
                if offset or limit then
                    return {
                        status = "unchanged",
                        message = string.format("[sift] %s lines %d-%d unchanged", raw_path, range_start, range_end)
                    }
                end
                return {
                    status = "unchanged",
                    message = "[sift] " .. raw_path .. " unchanged since last read"
                }
            end
        end

        -- Cache miss: try to load old content and emit diff
        if not fresh then
            local old_hash = sift.cache.get_path_hash(ctx, path)
            if old_hash then
                local old_content = sift.cache.load_file(ctx, old_hash)
                if old_content then
                    local diff = sift.diff(ctx, old_content, content)
                    -- Usefulness gate: only emit diff if < 90% of full content
                    if #diff < #content * 0.9 then
                        if offset or limit then
                            sift.cache.store_content(ctx, hash, content)
                            sift.cache.add_range(ctx, hash, range_start, range_end)
                        else
                            sift.cache.store_file(ctx, hash, content)
                        end
                        sift.cache.set_path_hash(ctx, path, hash)
                        return {
                            status = "handled",
                            output = diff,
                            exit_code = 0
                        }
                    end
                end
            end
        end

        -- Store new content and cache
        if offset or limit then
            -- Range read: store content without full hash marker
            sift.cache.store_content(ctx, hash, content)
            sift.cache.add_range(ctx, hash, range_start, range_end)
        else
            -- Full read: store content with full hash marker
            sift.cache.store_file(ctx, hash, content)
        end
        sift.cache.set_path_hash(ctx, path, hash)

        -- Return content (possibly sliced)
        if offset or limit then
            content = slice_text(content, range_start, range_end)
        end

        return {
            status = "handled",
            output = content,
            exit_code = 0
        }
    end
}
