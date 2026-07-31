-- sift-read.lua — file read plugin with caching, range support (priority 0)
-- Usage: sift-read <path> [<offset> [<limit>]]
--        sift-read --fresh <path> [<offset> [<limit>]]
-- Shares cache with cat.lua via file-based content store.
-- Returns "unchanged" on cache hit, or full content on change with a notification.

return {
    name = "sift-read",
    priority = 0,
    pattern = "sift-read",
    append_prompt = "File contents may be compressed or extracted. " ..
        "Use sift-read --raw <path> for original content.",

    execute = function(ctx, args, stdin)
        -- Parse args: [--fresh] <path> [<offset> [<limit>]]
        local parsed, err = sift.args.parse(args, {
            flags = { fresh = { "--fresh" }, raw = { "--raw" } },
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
        local raw = parsed.raw or false
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
            return { status = "handled", output = content, exit_code = 0, raw_bytes = stat and stat.size or 0 }
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
                    message = "[nudge] " .. display_name .. " unchanged (cached)\n      (bypass if stale: sift-read --fresh " .. path .. ")",
                    raw_bytes = stat and stat.size or 0
                }
            end

            -- Extract text via xberg
            local text = sift.ext.xberg.extract(ctx, path, { format = "markdown" })
            -- Compress via mdmin (skip if --raw)
            if not raw and sift.ext.markdown ~= nil then
                text = sift.ext.markdown.compress(ctx, text, { level = 2, code_blocks = "compress", dictionary = true })
            end
            -- Nudge agent that content was compressed
            if not raw then
                local nudge_args = ""
                if offset then nudge_args = nudge_args .. " " .. offset end
                if limit then nudge_args = nudge_args .. " " .. limit end
                sift.nudge(ctx, "raw: sift-read --raw " .. path .. nudge_args)
            end
            -- Cache extracted text by file hash
            sift.cache.store_file(ctx, hash, text)
            sift.cache.set_path_hash(ctx, path, hash)

            return {
                status = "handled",
                output = text,
                exit_code = 0,
                raw_bytes = stat and stat.size or 0
            }
        end

        -- Binary document without xberg: return helpful message
        if is_binary_document then
            local display_name = path:match("([^/]+)$") or path
            local msg = string.format("[nudge] %s is a binary document (%s). Install sift with --features xberg to extract text automatically.\n      (fallback: command cat %s)", display_name, mime, path)
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

        -- Compress markdown files via mdmin (skip if --raw)
        if not raw and sift.ext.markdown ~= nil and (path:match("%.md$") or path:match("%.markdown$")) then
            content = sift.ext.markdown.compress(ctx, content, { level = 2, code_blocks = "preserve", dictionary = true })
            local nudge_args = ""
            if offset then nudge_args = nudge_args .. " " .. offset end
            if limit then nudge_args = nudge_args .. " " .. limit end
            sift.nudge(ctx, "raw: sift-read --raw " .. path .. nudge_args)
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
                -- Mtime check: if file mtime differs from cached mtime, force re-read
                local current_mtime = stat and stat.mtime
                local cached_mtime = sift.cache.get_mtime(ctx, hash)
                if current_mtime and cached_mtime and current_mtime ~= cached_mtime then
                    -- File modified since cache — fall through to re-read
                    sift.nudge(ctx, path:match("([^/]+)$") or path .. " mtime changed, re-reading")
                else
                    local display_name = path:match("([^/]+)$") or path
                    if offset or limit then
                        local msg
                        if range_start == range_end then
                            msg = string.format("[nudge] %s line %d unchanged (cached)\n      (bypass if stale: sift-read --fresh %s %d)", display_name, range_start, path, range_start)
                        else
                            msg = string.format("[nudge] %s lines %d-%d unchanged (cached)\n      (bypass if stale: sift-read --fresh %s %d %d)", display_name, range_start, range_end, path, range_start, range_end - range_start + 1)
                        end
                        return {
                            status = "unchanged",
                            message = msg,
                            raw_bytes = stat and stat.size or 0
                        }
                    end
                    return {
                        status = "unchanged",
                        message = "[nudge] " .. display_name .. " unchanged — already in your context. sift-read --fresh " .. path .. " to re-read.",
                        raw_bytes = stat and stat.size or 0
                    }
                end
            end
        end

        -- Cache miss: check if file changed and add a brief notification
        -- Always return full content (no diff emission — diffs cause agent errors)
        if not fresh then
            local old_hash = sift.cache.get_path_hash(ctx, path)
            if old_hash and sift.cache.load_file(ctx, old_hash) then
                local display_name = path:match("([^/]+)$") or path
                sift.nudge(ctx, display_name .. " changed since last read")
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
            -- Store mtime for staleness detection
            if stat and stat.mtime then
                sift.cache.set_mtime(ctx, hash, stat.mtime)
            end
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
            raw_bytes = stat and stat.size or 0
        }
    end
}
