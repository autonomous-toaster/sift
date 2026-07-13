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

        local output, exit_code = sift.exec("git status --porcelain=v2 2>&1")
        sift.meta.raw_bytes = #output

        if exit_code ~= 0 then
            return { status = "passthrough" }
        end

        -- Check if working tree is clean (empty output)
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
