-- reset.lua — clear sift cache for current session (priority 1000)
-- Callable as: sift -c "reset" or typing "reset" in REPL mode.
-- Use "command reset" to bypass and run the real bash reset.
return {
    name = "reset",
    priority = 1000,
    pattern = "reset",

    execute = function(ctx, args, stdin)
        sift.cache.reset(ctx)
        return {
            status = "handled",
            output = "[sift] ok\n",
            exit_code = 0
        }
    end
}
