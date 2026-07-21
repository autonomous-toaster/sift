-- curl.lua — JSON/HTML/document response optimizer
-- Detects JSON responses via -w "%{content_type}", compresses with sift.json.shortest().
-- Detects HTML responses, converts to Markdown via sift.ext.html.to_markdown().
-- Detects PDF/document responses, extracts text via sift.ext.xberg.
-- If -v/--verbose explicitly requested, runs as-is with full output.
-- Always propagates curl exit code.

return {
    name = "curl",
    priority = 0,
    pattern = "curl",

    execute = function(ctx, args, stdin)
        -- Helper: derive a filesystem-safe slug from the curl URL
        local function slug_from_args(content_type)
            -- Extract URL: last non-flag argument
            local url
            for _, arg in ipairs(args) do
                if arg:sub(1, 1) ~= "-" then
                    url = arg
                end
            end
            if url then
                -- Strip query params and fragment
                local path = url:gsub("%?.*$", ""):gsub("#.*$", "")
                -- Get last path segment
                local name = path:match("([^/]+)$")
                if name and name ~= "" then
                    -- Sanitize for filesystem
                    local slug = name:gsub("[^%w%-_%.]", "_")
                    -- Append extension from content-type if slug has none
                    if not slug:match("%.%w+$") then
                        local ext = sift.ext.mime.extension(ctx, content_type)
                        if ext and ext ~= "" then
                            slug = slug .. "." .. ext
                        end
                    end
                    return slug
                end
            end
            -- Fallback: content-type based slug
            return "response_" .. content_type:gsub("/", "_"):gsub("[^%w%-_]", "_")
        end

        -- Check if -v or --verbose was explicitly requested
        local parsed, err = sift.args.parse(args, {
            flags = {
                v = { "-v" },
                verbose = { "--verbose" },
                w = { "-w", type = "str" },
                ["write-out"] = { "--write-out", type = "str" },
            },
            opts = { allow_unknown = true },
        })
        if not parsed then
            if err then return { status = "error", output = err } end
            return { status = "passthrough" }
        end

        if parsed.v or parsed.verbose or parsed.w or parsed["write-out"] then
            -- Agent asked for verbose or custom -w: run as-is, return full output
            local parts = { "curl" }
            for _, arg in ipairs(args) do
                parts[#parts + 1] = sift.str.shell_quote(ctx, arg)
            end
            local cmd = table.concat(parts, " ")
            local output, stderr, exit_code = sift.exec(ctx, cmd)
            return {
                status = "handled",
                output = output,
                exit_code = exit_code,
                streamed = true
            }
        end

        -- No -v: add -s to suppress progress meter, -w to detect content type
        -- Use single quotes to prevent bash from eating \n
        local new_args = { "-s", "-w", "'\\n%{content_type}'" }
        for _, arg in ipairs(args) do
            new_args[#new_args + 1] = sift.str.shell_quote(ctx, arg)
        end
        local cmd = "curl " .. table.concat(new_args, " ")
        local output, stderr, exit_code = sift.exec(ctx, cmd, { silent = true })

        if exit_code ~= 0 then
            return {
                status = "handled",
                output = output,
                exit_code = exit_code
            }
        end

        -- Parse: last line is content type, everything before is body
        -- Trim trailing newline from -w output
        local trimmed = output:gsub("\n$", "")
        local body, content_type = trimmed:match("^(.*)\n([^\n]*)$")

        if body and content_type then
            if content_type:find("json") then
                -- JSON detected: compress (shortest stores raw + nudges automatically)
                local compressed = sift.json.shortest(ctx, body, { toon = true })
                return {
                    status = "handled",
                    output = compressed,
                    exit_code = exit_code,
                    raw_bytes = #body
                }
            elseif content_type:find("html") and sift.ext.html ~= nil then
                -- HTML detected: convert to Markdown
                local md = sift.ext.html.to_markdown(ctx, body)
                -- Optionally compress via mdmin
                if sift.ext.markdown ~= nil then
                    md = sift.ext.markdown.compress(ctx, md, { level = 2 })
                end
                -- Store raw HTML for re-read
                local slug = slug_from_args(content_type)
                sift.store(ctx, body, slug)
                return {
                    status = "handled",
                    output = md,
                    exit_code = exit_code,
                    raw_bytes = #body
                }
            elseif sift.ext.xberg ~= nil and sift.ext.xberg.is_supported(ctx, content_type) then
                -- Document detected (PDF, etc.): extract via xberg (no temp file)
                local text = sift.ext.xberg.extract_bytes(ctx, body, content_type, { format = "markdown" })
                -- Compress via mdmin
                if sift.ext.markdown ~= nil then
                    text = sift.ext.markdown.compress(ctx, text, { level = 2, code_blocks = "compress", dictionary = true })
                end
                -- Store raw document for re-read
                local slug = slug_from_args(content_type)
                sift.store(ctx, body, slug)
                return {
                    status = "handled",
                    output = text,
                    exit_code = exit_code,
                    raw_bytes = #body
                }
            end
        end

        -- Not a recognized format: return as-is
        return {
            status = "handled",
            output = body or output,
            exit_code = exit_code
        }
    end
}
