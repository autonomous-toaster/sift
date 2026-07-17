-- reset.lua — clear sift cache for current session (priority 1000)
-- Callable as: sift -c "reset" or typing "reset" in REPL mode.
-- Use "command reset" to bypass and run the real bash reset.
-- Clears both in-memory cache and file-based content store.
-- Returns "(cleared)" or "(nothing to clear)" based on cache state.
return {
    name = "reset",
    priority = 1000,
    pattern = "reset",

    execute = function(ctx, args, stdin)
        local had_cache = sift.cache.has_any(ctx)
        sift.cache.reset(ctx)
        sift.cache.clear_all(ctx)
        if had_cache then
            return {
                status = "handled",
                output = "[sift] ok (cleared)\n",
                exit_code = 0
            }
        else
            return {
                status = "handled",
                output = "[sift] ok (nothing to clear)\n",
                exit_code = 0
            }
        end
    end
}
