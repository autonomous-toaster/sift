--[[
cargo_test.lua — Example sift plugin for `cargo test` output optimization

This plugin demonstrates:
  - Using sift.exec(ctx, cmd) to run a command with JSON output
  - Using sift.jq.query(ctx, data, filter) to filter and transform JSON data
  - Using sift.toon.encode(ctx, val) for token-optimized output
  - Using sift.meta to report token reduction metrics

Install: copy to plugins/cargo_test.lua or ~/.config/sift/plugins/cargo_test.lua
--]]

return {
    name = "cargo_test",
    priority = 0,
    pattern = "cargo",

    execute = function(ctx, args, stdin)
        -- Only handle "cargo test" commands
        if args[1] ~= "test" then
            return { status = "passthrough" }
        end

        -- Run cargo test with JSON output for machine parsing
        local cmd = "cargo test --message-format=json 2>&1"
        local raw_output, stderr, exit_code = sift.exec(ctx, cmd)

        -- Record raw output size for token tracking
        sift.meta.raw_bytes = #raw_output

        -- Parse JSON lines and extract test results using jq
        local results = sift.jq.query(ctx, raw_output, [[
            [.[] | select(.type == "test") | {
                name: .name,
                status: .event,
                outcome: if .event == "ok" then "passed" else "failed" end
            }]
        ]])

        -- Count passed/failed
        local passed = sift.jq.query(ctx, results, '[.[] | select(.outcome == "passed")] | length')
        local failed = sift.jq.query(ctx, results, '[.[] | select(.outcome == "failed")] | length')

        -- Build summary as Lua table
        local summary = {
            passed = tonumber(passed) or 0,
            failed = tonumber(failed) or 0,
            total = (tonumber(passed) or 0) + (tonumber(failed) or 0)
        }

        -- Encode as TOON for token-optimized output
        local output = sift.toon.encode(ctx, summary)

        -- Add failure details if any
        if summary.failed > 0 then
            local failures = sift.jq.query(ctx, results, '[.[] | select(.outcome == "failed") | .name]')
            output = output .. "\n" .. "FAILED: " .. failures
        end

        return {
            status = "handled",
            output = output,
            exit_code = exit_code
        }
    end
}
