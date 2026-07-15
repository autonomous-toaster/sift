-- cat.lua — file read plugin (priority 0)
-- Reads file via sift.fs.read(), caches by hash, returns "unchanged" on cache hit.
-- Shares cache with sift-read.lua via file-based content store.
-- Also handles piped stdin: caches by content hash, returns unchanged on repeat.
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

return {
    name = "cat",
    priority = 0,
    pattern = "cat",

    execute = function(ctx, args, stdin)
        -- Handle piped stdin
        if stdin ~= nil then
            local hash = sift.hash.sha256(ctx, stdin)
            local cache_key = "stdin:" .. hash

            if sift.cache.has(ctx, cache_key) then
                return {
                    status = "unchanged",
                    fingerprint = cache_key,
                    message = "[sift] piped content unchanged since last read"
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
        for _, arg in ipairs(args) do
            if arg:sub(1, 1) == "-" then
                return { status = "passthrough" }
            end
        end

        if #args ~= 1 then
            return { status = "passthrough" }
        end

        local path = args[1]
        if path:sub(1, 1) ~= "/" then
            path = ctx.cwd .. "/" .. path
        end

        -- Sensitive path bypass: don't cache
        if is_sensitive(path) then
            local content = sift.fs.read(ctx, path)
            if content == nil then
                return nil, "cat: " .. args[1] .. ": No such file or directory"
            end
            return { status = "handled", output = content, exit_code = 0 }
        end

        local content = sift.fs.read(ctx, path)
        if content == nil then
            return nil, "cat: " .. args[1] .. ": No such file or directory"
        end

        -- Compute hash for cache
        local hash = sift.hash.sha256(ctx, content)

        -- Check file-based cache first (shared with sift-read)
        if sift.cache.has_file(ctx, hash) then
            sift.nudge(ctx, "bypass: 'command cat " .. args[1] .. "'")
            return {
                status = "unchanged",
                message = "[sift] " .. args[1] .. " unchanged since last read"
            }
        end

        -- Also check in-memory cache (for piped stdin compatibility)
        local cache_key = path .. ":" .. hash
        if sift.cache.has(ctx, cache_key) then
            sift.nudge(ctx, "bypass: 'command cat " .. args[1] .. "'")
            return {
                status = "unchanged",
                fingerprint = cache_key,
                message = "[sift] " .. args[1] .. " unchanged since last read"
            }
        end

        -- Store in both caches
        sift.cache.store_file(ctx, hash, content)
        sift.cache.set(ctx, cache_key)

        return {
            status = "handled",
            output = content,
            exit_code = 0
        }
    end
}
