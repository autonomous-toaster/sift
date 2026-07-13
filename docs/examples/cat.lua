--[[
cat.lua — Example Lua plugin for baish
========================================

This plugin demonstrates the baish plugin API by implementing a
cat command that caches file reads and emits an "unchanged" marker
when the same file is read again.

Plugin API Reference
--------------------

PluginContext (read-only, passed to execute()):
  .cwd         — Current working directory (string)
  .cmd_count   — Number of commands executed so far (number)
  .env         — Environment variables (table)
  .session_id  — Current AI_SESSION identifier (string or nil)

PluginResult (returned from execute()):
  { status = "handled", output = "...", exit_code = 0 }
      — Plugin handled the command. Output goes to stdout.
  { status = "passthrough" }
      — Plugin did not handle this command. Fall through to PTY.
  { status = "unchanged", fingerprint = "...", message = "..." }
      — Output is identical to a previous invocation. Emit a short marker.

Cache functions (available through the plugin API):
  cache_get(key)       — Get cached value by key (returns nil if not found)
  cache_set(key, val)  — Set cached value
  cache_has(key)       — Check if key exists in cache
--]]

local plugin = {}

-- Plugin metadata
plugin.name = "cat"
plugin.priority = 100  -- Higher priority than built-in (-100)

function plugin.execute(ctx, args, stdin)
    -- Passthrough if stdin is piped
    if stdin ~= nil then
        return { status = "passthrough" }
    end

    -- Passthrough if flags are present
    for _, arg in ipairs(args) do
        if arg:sub(1, 1) == "-" then
            return { status = "passthrough" }
        end
    end

    -- Need exactly one file argument
    if #args ~= 1 then
        return { status = "passthrough" }
    end

    local path = args[1]

    -- Resolve relative paths
    if path:sub(1, 1) ~= "/" then
        path = ctx.cwd .. "/" .. path
    end

    -- Read the file
    local file, err = io.open(path, "rb")
    if not file then
        return nil, "cat: " .. path .. ": " .. err
    end
    local content = file:read("*all")
    file:close()

    -- Compute hash for cache
    local hash = compute_hash(content)

    -- Check cache
    local cache_key = path .. ":" .. hash
    local cached = cache_get(cache_key)
    if cached ~= nil and cached.shown_count > 0 then
        -- File unchanged since last read
        return {
            status = "unchanged",
            fingerprint = cache_key,
            message = "[baish] " .. args[1] .. " unchanged since last read"
        }
    end

    -- Update cache
    cache_set(cache_key, {
        shown_count = 0,
        first_shown = os.time()
    })

    return {
        status = "handled",
        output = content,
        exit_code = 0
    }
end

-- Simple hash function (SHA256 would be better, but Lua doesn't have it built-in)
function compute_hash(content)
    local hash = 0
    for i = 1, #content do
        local byte = content:byte(i)
        hash = hash * 31 + byte
        hash = hash % 2^32
    end
    return string.format("%x", hash)
end

return plugin
