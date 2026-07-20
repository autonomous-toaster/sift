use super::{SiftContext, SiftLua};
use mlua::{Lua, Table};
use std::collections::HashMap;

fn test_context() -> SiftContext {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    SiftContext {
        cwd: std::env::current_dir().unwrap(),
        cwd_str: std::env::current_dir().unwrap().display().to_string(),
        cmd_count: 0,
        env: HashMap::new(),
        session_id: Some(format!("test-cache-{ts}")),
        raw_bytes: 0,
        filtered_bytes: 0,
    }
}

fn test_ctx(lua: &Lua) -> Table {
    let ctx = lua.create_table().unwrap();
    // Use unique session_id per test to avoid parallel test interference
    let session_id = format!(
        "test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    );
    ctx.set("session_id", session_id).unwrap();
    ctx.set("cmd_count", 0).unwrap();
    ctx.set("cwd", "/tmp").unwrap();
    ctx.set("command", "test").unwrap();
    ctx
}

fn cache_call_bool(cache: &mlua::Table, name: &str, args: impl mlua::IntoLuaMulti + Send) -> bool {
    let f: mlua::Function = cache.get(name).unwrap();
    let val: mlua::Value = f.call(args).unwrap();
    match val {
        mlua::Value::Boolean(b) => b,
        _ => panic!("expected bool from {name}, got {val:?}"),
    }
}

fn cache_call_void(cache: &mlua::Table, name: &str, args: impl mlua::IntoLuaMulti + Send) {
    let f: mlua::Function = cache.get(name).unwrap();
    let _: mlua::Value = f.call(args).unwrap();
}

#[test]
fn test_cleanup_cache() {
    let session_id = "test-cleanup";
    let base = std::path::PathBuf::from("/tmp/sift").join(session_id);
    let cache_dir = base.join("cache");
    let objects_dir = base.join("objects");
    let _ = std::fs::remove_dir_all(&base);

    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::create_dir_all(&objects_dir).unwrap();
    let meta = serde_json::json!({"created_at": 1_000_000_000_000u64, "size": 10});
    std::fs::write(cache_dir.join("abc123"), meta.to_string()).unwrap();
    std::fs::write(objects_dir.join("sha256-abc123.txt"), "content").unwrap();

    std::fs::write(objects_dir.join("sha256-orphan.txt"), "orphan").unwrap();

    crate::lua::exec::cleanup_cache(session_id, 1);

    assert!(
        !cache_dir.join("abc123").exists(),
        "expired cache entry should be deleted"
    );
    assert!(
        !objects_dir.join("sha256-orphan.txt").exists(),
        "orphan object should be deleted"
    );

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_cleanup_cache_preserves_fresh() {
    let session_id = "test-cleanup-fresh";
    let base = std::path::PathBuf::from("/tmp/sift").join(session_id);
    let cache_dir = base.join("cache");
    let objects_dir = base.join("objects");
    let _ = std::fs::remove_dir_all(&base);

    std::fs::create_dir_all(&cache_dir).unwrap();
    std::fs::create_dir_all(&objects_dir).unwrap();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let meta = serde_json::json!({"created_at": now, "size": 10});
    std::fs::write(cache_dir.join("abc123"), meta.to_string()).unwrap();
    std::fs::write(objects_dir.join("sha256-abc123.txt"), "content").unwrap();

    crate::lua::exec::cleanup_cache(session_id, 86_400_000);

    assert!(
        cache_dir.join("abc123").exists(),
        "fresh cache entry should be preserved"
    );
    assert!(
        objects_dir.join("sha256-abc123.txt").exists(),
        "referenced object should be preserved"
    );

    let _ = std::fs::remove_dir_all(&base);
}

#[test]
fn test_range_cache_add_and_has() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: mlua::Table = lua.lua.globals().get("sift").unwrap();
    let cache: mlua::Table = sift.get("cache").unwrap();
    let ctx = test_ctx(&lua.lua);
    let hash = "test-range-hash";

    cache_call_void(&cache, "add_range", (ctx.clone(), hash, 1u64, 4u64));

    assert!(
        cache_call_bool(&cache, "has_range", (ctx.clone(), hash, 1u64, 4u64)),
        "[1,4] should be contained"
    );
    assert!(
        !cache_call_bool(&cache, "has_range", (ctx.clone(), hash, 1u64, 5u64)),
        "[1,5] should NOT be contained"
    );
    assert!(
        !cache_call_bool(&cache, "has_range", (ctx.clone(), hash, 3u64, 5u64)),
        "[3,5] should NOT be contained"
    );

    cache_call_void(&cache, "add_range", (ctx.clone(), hash, 1u64, 5u64));

    assert!(
        cache_call_bool(&cache, "has_range", (ctx.clone(), hash, 1u64, 5u64)),
        "[1,5] should be contained after merge"
    );
    assert!(
        cache_call_bool(&cache, "has_range", (ctx, hash, 3u64, 5u64)),
        "[3,5] should be contained by [1,5]"
    );
}

#[test]
fn test_range_cache_merge_adjacent() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: mlua::Table = lua.lua.globals().get("sift").unwrap();
    let cache: mlua::Table = sift.get("cache").unwrap();
    let ctx = test_ctx(&lua.lua);
    let hash = "test-merge-adj";

    cache_call_void(&cache, "add_range", (ctx.clone(), hash, 1u64, 3u64));
    cache_call_void(&cache, "add_range", (ctx.clone(), hash, 4u64, 6u64));

    assert!(
        cache_call_bool(&cache, "has_range", (ctx, hash, 1u64, 6u64)),
        "[1,6] should be contained after merge"
    );
}

#[test]
fn test_range_cache_full_vs_range() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: mlua::Table = lua.lua.globals().get("sift").unwrap();
    let cache: mlua::Table = sift.get("cache").unwrap();
    let ctx = test_ctx(&lua.lua);
    let hash = "test-full-vs-range";

    cache_call_void(&cache, "store_file", (ctx.clone(), hash, "test content"));

    assert!(
        cache_call_bool(&cache, "has_file", (ctx.clone(), hash)),
        "has_file should return true for full read"
    );
    assert!(
        !cache_call_bool(&cache, "has_range", (ctx, hash, 5u64, 10u64)),
        "has_range should return false"
    );
}

#[test]
fn test_sift_read_empty_diff_regression() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();

    let plugin = r#"
        return {
            name = "test-read",
            priority = 0,
            pattern = "test-read",
            execute = function(ctx, args, stdin)
                local content = "line1\nline2\nline3\nline4\nline5\n"
                local hash = sift.hash.sha256(ctx, content)
                local offset = tonumber(args[2])
                local limit = tonumber(args[3])
                local range_start = offset or 1
                local range_end = limit and (offset or 1) + limit - 1 or 5

                if sift.cache.has_file(ctx, hash) then
                    return { status = "unchanged", message = "unchanged" }
                end
                if offset and sift.cache.has_range(ctx, hash, range_start, range_end) then
                    return { status = "unchanged", message = "unchanged range" }
                end

                local old_hash = sift.cache.get_path_hash(ctx, args[1])
                if old_hash then
                    local old_content = sift.cache.load_file(ctx, old_hash)
                    if old_content then
                        local diff = sift.diff(ctx, old_content, content)
                        if #diff > 0 and #diff < #content * 0.9 then
                            return { status = "handled", output = diff, exit_code = 0 }
                        end
                    end
                end

                sift.cache.store_content(ctx, hash, content)
                sift.cache.set_path_hash(ctx, args[1], hash)
                if offset then
                    local lines = {}
                    for line in content:gmatch("([^\n]*)\n?") do
                        table.insert(lines, line)
                    end
                    local sliced = {}
                    for i = range_start, math.min(range_end, #lines) do
                        table.insert(sliced, lines[i])
                    end
                    content = table.concat(sliced, "\n")
                    sift.cache.add_range(ctx, hash, range_start, range_end)
                end
                return { status = "handled", output = content, exit_code = 0 }
            end
        }
    "#;
    lua.load_plugin_from_str("test-read", plugin).unwrap();

    let (output, code, _) = lua
        .dispatch(
            "test-read",
            &["test.txt".into(), "1".into(), "4".into()],
            None::<mlua::Value>,
            false,
        )
        .unwrap();
    assert_eq!(code, 0);
    assert!(
        output.contains("line1"),
        "first read should return content, got: {output}"
    );
    assert!(
        output.contains("line4"),
        "first read should include line4, got: {output}"
    );

    let (output, code, _) = lua
        .dispatch(
            "test-read",
            &["test.txt".into(), "1".into(), "5".into()],
            None::<mlua::Value>,
            false,
        )
        .unwrap();
    assert_eq!(code, 0);
    assert!(
        output.contains("line1"),
        "second read should return content, got: {output}"
    );
    assert!(
        output.contains("line5"),
        "second read should include line5, got: {output}"
    );
    assert!(
        !output.is_empty(),
        "output should NOT be empty (regression: empty diff bug)"
    );
}
