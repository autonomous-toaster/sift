-- openspec.lua — OpenSpec plugin (priority 0)
-- Injects --json flag, converts output via sift.json.shortest()
return {
    name = "openspec",
    priority = 0,
    pattern = "openspec",

    execute = function(ctx, args, stdin)
        -- Check if --json is already present
        local parsed, err = sift.args.parse(args, {
            flags = { json = { "--json" } },
            opts = { allow_unknown = true },
        })
        if not parsed then
            if err then return nil, err end
            return { status = "passthrough" }
        end

        local has_json = parsed.json or false

        -- Build command with --json injected if missing
        local parts = {ctx.command}
        for i = 1, #args do
            parts[#parts + 1] = args[i]
        end
        if not has_json then
            parts[#parts + 1] = "--json"
        end
        local cmd = table.concat(parts, " ")
        local output, stderr, exit_code = sift.exec(ctx, cmd, {silent = true})

        if exit_code ~= 0 then
            return {
                status = "handled",
                output = output .. stderr,
                exit_code = exit_code
            }
        end

        -- Convert JSON output via sift.json.shortest()
        local formats = {
            json = { max_string_len = 80 },
            toon = true
        }
        local optimized = sift.json.shortest(ctx, output, formats)

        return {
            status = "handled",
            output = optimized,
            exit_code = 0
        }
    end
}
