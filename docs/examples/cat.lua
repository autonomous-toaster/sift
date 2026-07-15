--[[
cat.lua — Example sift plugin for `cat` command

Demonstrates:
  - Using sift.fs.read(ctx, path) to read files
  - Using sift.hash.sha256(ctx, data) for content hashing
  - Using sift.cache for deduplication
  - Returning "unchanged" status on cache hit
  - Handling piped stdin with caching

Install: copy to plugins/cat.lua or ~/.config/sift/plugins/cat.lua
--]]

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

        local content = sift.fs.read(ctx, path)
        if content == nil then
            return nil, "cat: " .. args[1] .. ": No such file or directory"
        end

        local hash = sift.hash.sha256(ctx, content)
        local cache_key = path .. ":" .. hash

        if sift.cache.has(ctx, cache_key) then
            return {
                status = "unchanged",
                fingerprint = cache_key,
                message = "[sift] " .. args[1] .. " unchanged since last read"
            }
        end

        sift.cache.set(ctx, cache_key)

        return {
            status = "handled",
            output = content,
            exit_code = 0
        }
    end
}
