-- cat.lua — file read plugin (priority 0)
-- Reads file via sift.fs.read(), caches by hash, returns "unchanged" on cache hit.
return {
    name = "cat",
    priority = 0,
    pattern = "cat",

    execute = function(ctx, args, stdin)
        -- Passthrough if stdin is piped
        if stdin ~= nil then
            return { status = "passthrough" }
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

        local content = sift.fs.read(path)
        if content == nil then
            return nil, "cat: " .. args[1] .. ": No such file or directory"
        end

        -- Compute hash for cache
        local hash = sift.hash.sha256(content)
        local cache_key = path .. ":" .. hash

        if sift.cache.has(cache_key) then
            return {
                status = "unchanged",
                fingerprint = cache_key,
                message = "[sift] " .. args[1] .. " unchanged since last read"
            }
        end

        sift.cache.set(cache_key, true)

        return {
            status = "handled",
            output = content,
            exit_code = 0
        }
    end
}
