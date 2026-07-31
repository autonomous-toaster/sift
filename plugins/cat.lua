-- cat.lua — file read plugin (priority 0)
-- Reads file via sift.fs.read(), caches by hash, returns "unchanged" on cache hit.
-- Shares cache with sift-read.lua via file-based content store.
-- Also handles piped stdin: caches by content hash, returns unchanged on repeat.
-- Passthrough on heredoc syntax or non-existent paths to avoid crashes.

return {
    name = "cat",
    priority = 0,
    pattern = "cat",
    append_prompt = "Markdown files may be compressed via mdmin. " ..
        "Use 'command cat' to read without compression.",

    execute = function(ctx, args, stdin)
        -- Handle piped stdin (supports both string and StdinReader)
        if stdin ~= nil then
            if type(stdin) == "userdata" then
                stdin = tostring(stdin)
            end
            local hash = sift.hash.sha256(ctx, stdin)
            local cache_key = "stdin:" .. hash

            if sift.cache.has(ctx, cache_key) then
                return {
                    status = "unchanged",
                    fingerprint = cache_key,
                    message = "[nudge] piped content unchanged — already in your context."
                }
            end

            sift.cache.set(ctx, cache_key)

            return {
                status = "handled",
                output = stdin,
                exit_code = 0
            }
        end

        -- Passthrough if flags are present or wrong number of args
        local parsed, err = sift.args.parse(args, {
            args = { { name = "path", required = true } },
            opts = { allow_unknown = false },
        })
        if not parsed then
            if err then return { status = "error", output = err } end
            return { status = "passthrough" }
        end

        local path = parsed.path
        if path:sub(1, 1) ~= "/" then
            path = ctx.cwd .. "/" .. path
        end

        -- Passthrough if path contains shell metacharacters (heredoc, redirect, etc.)
        if path:match("[<>|;&$`]") then
            return { status = "passthrough" }
        end

        -- Passthrough if path doesn't exist (avoids crash from sift.fs.stat on non-existent paths)
        if not sift.fs.exists(ctx, path) then
            return { status = "passthrough" }
        end

        -- Sensitive path bypass: don't cache
        if sift.str.is_sensitive(ctx, path) then
            local stat = sift.fs.stat(ctx, path)
            local content = sift.fs.read(ctx, path)
            if content == nil then
                return { status = "error", output = "cat: " .. args[1] .. ": No such file or directory" }
            end
            return { status = "handled", output = content, exit_code = 0, raw_bytes = stat and stat.size or 0 }
        end

        local stat = sift.fs.stat(ctx, path)
        local content = sift.fs.read(ctx, path)
        if content == nil then
            return { status = "error", output = "cat: " .. args[1] .. ": No such file or directory" }
        end

        -- Compress markdown files via mdmin (level 2, preserve code blocks)
        if sift.ext.markdown ~= nil and (path:match("%.md$") or path:match("%.markdown$")) then
            content = sift.ext.markdown.compress(ctx, content, { level = 2, code_blocks = "preserve", dictionary = true })
            sift.nudge(ctx, "compressed output. raw: command cat " .. path)
        end

        -- Compute hash for cache
        local hash = sift.hash.sha256(ctx, content)

        -- Check file-based cache first (shared with sift-read)
        if sift.cache.has_file(ctx, hash) then
            -- Mtime check: if file mtime differs from cached mtime, force re-read
            local current_mtime = stat and stat.mtime
            local cached_mtime = sift.cache.get_mtime(ctx, hash)
            if current_mtime and cached_mtime and current_mtime ~= cached_mtime then
                -- File modified since cache — fall through to re-read
                sift.nudge(ctx, (path:match("([^/]+)$") or args[1]) .. " mtime changed, re-reading")
            else
                local display_name = path:match("([^/]+)$") or args[1]
                return {
                    status = "unchanged",
                    message = "[nudge] " .. display_name .. " unchanged — already in your context. command cat " .. path .. " to re-read.",
                    raw_bytes = stat and stat.size or 0
                }
            end
        end

        -- Also check in-memory cache (for piped stdin compatibility)
        local cache_key = path .. ":" .. hash
        if sift.cache.has(ctx, cache_key) then
            return {
                status = "unchanged",
                fingerprint = cache_key,
                message = "[nudge] piped content unchanged — already in your context."
            }
        end

        -- Store in both caches
        sift.cache.store_file(ctx, hash, content)
        -- Store mtime for staleness detection
        if stat and stat.mtime then
            sift.cache.set_mtime(ctx, hash, stat.mtime)
        end
        sift.cache.set(ctx, cache_key)

        return {
            status = "handled",
            output = content,
            exit_code = 0,
            raw_bytes = stat and stat.size or 0
        }
    end
}
