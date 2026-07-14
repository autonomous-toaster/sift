-- git_status.lua — git status plugin (priority 0)
-- Fingerprints output, returns "working tree clean" on match.
return {
    name = "git_status",
    priority = 0,
    pattern = "git",

    execute = function(ctx, args, stdin)
        if args[1] ~= "status" then
            return { status = "passthrough" }
        end

        -- Extend original command with --porcelain=v2 instead of hardcoding
        -- ctx.command = "git", args = {"status"}
        local parts = {ctx.command}
        for i = 1, #args do
            parts[#parts + 1] = args[i]
        end
        local cmd = table.concat(parts, " ") .. " --porcelain=v2"
        local output, stderr, exit_code = sift.exec(cmd)
        sift.meta.raw_bytes = #output + #stderr

        if exit_code ~= 0 then
            return { status = "passthrough" }
        end

        -- Check if working tree is clean (empty stdout)
        if output == "" or output:match("^%s*$") then
            return {
                status = "unchanged",
                fingerprint = "git:status:clean",
                message = "[sift] working tree clean"
            }
        end

        return {
            status = "handled",
            output = output,
            exit_code = exit_code
        }
    end
}
