--[[
cat.lua — Example sift plugin for `cat` command

Demonstrates:
  - Using sift.fs.read() to read files
  - Using sift.hash.sha256() for content hashing
  - Using sift.cache for deduplication
  - Returning "unchanged" status on cache hit

Install: copy to ~/.config/sift/plugins/cat.lua
--]]

return {
    name = "cat",
    priority = 0,
    pattern = "cat",

    execute = function(ctx, args, stdin)
        if stdin ~= nil then
            return { status = "passthrough" }
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

        local content = sift.fs.read(path)
        if content == nil then
            return nil, "cat: " .. args[1] .. ": No such file or directory"
        end

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
