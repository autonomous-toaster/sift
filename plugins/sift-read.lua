-- sift-read.lua — file read plugin with caching, range support, and diff emission (priority 0)
-- Usage: sift-read <path> [<offset> [<limit>]]
--        sift-read --fresh <path> [<offset> [<limit>]]
-- Shares cache with cat.lua via file-based content store.
-- Returns "unchanged" on cache hit, unified diff on content change, or full content.

return {
    name = "sift-read",
    priority = 0,
    pattern = "sift-read",

    execute = function(ctx, args, stdin)
        -- Parse args: [--fresh] <path> [<offset> [<limit>]]
        local parsed, err = sift.args.parse(args, {
            flags = { fresh = { "--fresh" } },
            args = {
                { name = "path", required = true },
                { name = "offset", type = "int" },
                { name = "limit", type = "int" },
            },
        })
        if not parsed then
            if err then return { status = "error", output = err } end
            return { status = "passthrough" }
        end

        local fresh = parsed.fresh or false
        local path = parsed.path
        local raw_path = path
        local offset = parsed.offset
        local limit = parsed.limit

        -- Resolve path
        if path:sub(1, 1) ~= "/" then
            path = ctx.cwd .. "/" .. path
        end

        -- Sensitive path bypass
        if sift.str.is_sensitive(ctx, path) then
            local stat = sift.fs.stat(ctx, path)
            local content = sift.fs.read(ctx, path)
            if content == nil then
                return { status = "error", output = "sift-read: " .. raw_path .. ": No such file or directory" }
            end
            if offset and limit then
                local lines = sift.str.split_lines(ctx, content)
                local start = offset
                local end_line = math.min(offset + limit - 1, #lines)
                content = sift.str.slice_text(ctx, content, start, end_line)
            elseif offset then
                local lines = sift.str.split_lines(ctx, content)
                content = sift.str.slice_text(ctx, content, offset, #lines)
            end
            return { status = "handled", output = content, exit_code = 0, raw_bytes = stat.size }
        end

        -- Detect MIME type for binary document routing
        -- Only route non-text documents to xberg
        local mime = sift.ext.mime.detect(ctx, path)
        local is_binary_document = not mime:match("^text/")

        if is_binary_document and sift.ext.xberg ~= nil and sift.ext.xberg.is_supported(ctx, mime) then
            -- Read raw bytes for hashing (Lua strings are binary-safe)
            local stat = sift.fs.stat(ctx, path)
            local raw_content = sift.fs.read(ctx, path)
            if raw_content == nil then
                return { status = "error", output = "sift-read: " .. raw_path .. ": No such file or directory" }
            end

            local hash = sift.hash.sha256(ctx, raw_content)

            -- Check cache by file hash
            if not fresh and sift.cache.has_file(ctx, hash) then
                local display_name = path:match("([^/]+)$") or path
                return {
                    status = "unchanged",
                    message = "[sift] " .. display_name .. " unchanged (cached)\n      (bypass if stale: sift-read --fresh " .. path .. ")",
                    raw_bytes = stat.size
                }
            end

            -- Extract text via xberg
            local text = sift.ext.xberg.extract(ctx, path, { format = "markdown" })
            -- Compress via mdmin
            if sift.ext.markdown ~= nil then
                text = sift.ext.markdown.compress(ctx, text, { level = 2, code_blocks = "compress", dictionary = true })
            end
            -- Cache extracted text by file hash
            sift.cache.store_file(ctx, hash, text)
            sift.cache.set_path_hash(ctx, path, hash)

            return {
                status = "handled",
                output = text,
                exit_code = 0,
                raw_bytes = stat.size
            }
        end

        -- Binary document without xberg: return helpful message
        if is_binary_document then
            local display_name = path:match("([^/]+)$") or path
            local msg = string.format("[sift] %s is a binary document (%s). Install sift with --features xberg to extract text automatically.\n      (fallback: command cat %s)", display_name, mime, path)
            return {
                status = "handled",
                output = msg,
                exit_code = 0
            }
        end

        -- Read full file (text files)
        local stat = sift.fs.stat(ctx, path)
        local content = sift.fs.read(ctx, path)
        if content == nil then
            return { status = "error", output = "sift-read: " .. raw_path .. ": No such file or directory" }
        end

        -- Compress markdown files via mdmin (level 2, preserve code blocks)
        if sift.ext.markdown ~= nil and (path:match("%.md$") or path:match("%.markdown$")) then
            content = sift.ext.markdown.compress(ctx, content, { level = 2, code_blocks = "preserve", dictionary = true })
        end

        local total_lines = #sift.str.split_lines(ctx, content)
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
                local display_name = path:match("([^/]+)$") or path
                if offset or limit then
                    local msg
                    if range_start == range_end then
                        msg = string.format("[sift] %s line %d unchanged (cached)\n      (bypass if stale: sift-read --fresh %s %d)", display_name, range_start, path, range_start)
                    else
                        msg = string.format("[sift] %s lines %d-%d unchanged (cached)\n      (bypass if stale: sift-read --fresh %s %d %d)", display_name, range_start, range_end, path, range_start, range_end - range_start + 1)
                    end
                    return {
                        status = "unchanged",
                        message = msg,
                        raw_bytes = stat.size
                    }
                end
                return {
                    status = "unchanged",
                    message = "[sift] " .. display_name .. " unchanged (cached)\n      (bypass if stale: sift-read --fresh " .. path .. ")",
                    raw_bytes = stat.size
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
                    -- Usefulness gate: only emit diff if non-empty and < 90% of full content
                    if #diff > 0 and #diff < #content * 0.9 then
                        -- Count changed lines for header
                        local changed = 0
                        for line in diff:gmatch("[^\n]+") do
                            local first = line:sub(1, 1)
                            if first == "+" or first == "-" then
                                if line:sub(1, 3) ~= "---" and line:sub(1, 3) ~= "+++" then
                                    changed = changed + 1
                                end
                            end
                        end
                        local header = string.format("[sift: %d lines changed of %d]\n", changed, total_lines)
                        if offset or limit then
                            sift.cache.store_content(ctx, hash, content)
                            sift.cache.add_range(ctx, hash, range_start, range_end)
                        else
                            sift.cache.store_file(ctx, hash, content)
                        end
                        sift.cache.set_path_hash(ctx, path, hash)
                        return {
                            status = "handled",
                            output = header .. diff,
                            exit_code = 0,
                            raw_bytes = stat.size
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
            content = sift.str.slice_text(ctx, content, range_start, range_end)
        end

        return {
            status = "handled",
            output = content,
            exit_code = 0,
            raw_bytes = stat.size
        }
    end
}
