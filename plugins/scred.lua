-- scred.lua — Redact secrets from echo/env/printenv output (priority 0)
-- Executes the command, pipes output through scred binary for secret redaction.
-- Falls through to passthrough if scred is not installed or fails.
return {
    name = "scred",
    priority = 0,
    pattern = { "echo", "env", "printenv" },
    append_prompt = "Output from echo, env, and printenv is redacted by scred. " ..
        "Use 'command' prefix to bypass.",

    execute = function(ctx, args, stdin)
        -- Use original command to preserve shell semantics (variable expansion, etc.)
        local cmd = ctx.original_cmd

        -- Execute the command silently, capture output
        local out, stderr, code = sift.exec(ctx, cmd, { silent = true })
        if code ~= 0 or #out == 0 then
            return { status = "passthrough" }
        end

        -- Pipe output through scred for secret redaction (silent)
        local redacted, _, scred_code = sift.exec(ctx, "scred", { stdin = out, silent = true })
        if scred_code == 0 then
            sift.nudge(ctx, "raw: command " .. ctx.original_cmd)
            return {
                status = "handled",
                output = redacted,
                exit_code = 0
            }
        end

        -- scred not installed or failed — return original output
        return {
            status = "handled",
            output = out,
            exit_code = 0
        }
    end
}
