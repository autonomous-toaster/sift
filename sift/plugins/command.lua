-- command.lua — bypass plugin (priority 1000)
-- Matches "command" prefix, returns passthrough to run the real binary.
return {
    name = "command",
    priority = 1000,
    pattern = "command",

    execute = function(ctx, args, stdin)
        -- "command cat foo" → args = {"cat", "foo"}
        -- Return passthrough → sift runs the real binary directly
        return { status = "passthrough" }
    end
}
