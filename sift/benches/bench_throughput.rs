//! Measures plugin processing throughput (pure processing, no bash I/O).
//! Uses a Lua plugin that transforms text without spawning bash.
//! This is the "pure processing" the user wants optimized.

#![allow(clippy::unwrap_used, clippy::pedantic, missing_docs)]

use sift_core::lua::{SiftContext, SiftLua};
use std::collections::HashMap;
use std::time::Instant;

fn main() {
    let ctx = SiftContext {
        cwd: std::env::current_dir().unwrap(),
        cwd_str: std::env::current_dir().unwrap().display().to_string(),
        cmd_count: std::cell::Cell::new(0),
        env: HashMap::new(),
        session_id: None,
        raw_bytes: 0,
        filtered_bytes: 0,
    };

    // Phase 1: Cold start — create SiftLua + load plugins
    let start = Instant::now();
    let mut lua = SiftLua::new(None, ctx.clone()).unwrap();
    // Load a text-processing plugin (no bash execution)
    lua.load_plugin_from_str(
        "processor",
        r#"
        return {
            name = "processor",
            priority = 0,
            pattern = "processor",
            execute = function(ctx, args, stdin)
                local input = ""
                if stdin ~= nil then
                    input = tostring(stdin)
                end
                -- Simulate text processing: uppercase + line count
                local lines = 0
                for _ in input:gmatch("[^\n]+") do
                    lines = lines + 1
                end
                local result = string.upper(input) .. "\n[processed " .. lines .. " lines]"
                return { status = "handled", output = result, exit_code = 0 }
            end
        }
    "#,
    )
    .unwrap();
    let cold_start = start.elapsed();

    // Phase 2: Warmup
    let input_data = "line1\nline2\nline3\nline4\nline5\n";
    for _ in 0..100 {
        let _ = lua.dispatch_full("processor arg1", None).unwrap();
    }

    // Phase 3: Measure throughput
    const ITERATIONS: usize = 10000;
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let _ = lua.dispatch_full("processor arg1", None).unwrap();
    }
    let elapsed = start.elapsed();
    let throughput = (ITERATIONS as f64) / elapsed.as_secs_f64();

    println!("cold_start_ns={}", cold_start.as_nanos());
    println!("dispatches={}", ITERATIONS);
    println!("elapsed_ns={}", elapsed.as_nanos());
    println!("throughput_cps={:.0}", throughput);
    println!("METRIC throughput_cps={:.0}", throughput);
    println!("METRIC cold_start_ns={}", cold_start.as_nanos());
}
