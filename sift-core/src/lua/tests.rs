use super::{SiftContext, SiftLua};
use crate::lua::exec::{exec_command, find_real_bash, save_output};
use mlua::{Lua, Table};
use std::collections::HashMap;

fn test_context() -> SiftContext {
    SiftContext {
        cwd: std::env::current_dir().unwrap(),
        cmd_count: 0,
        env: HashMap::new(),
        session_id: None,
        raw_bytes: 0,
        filtered_bytes: 0,
    }
}

fn test_ctx(lua: &Lua) -> Table {
    let ctx = lua.create_table().unwrap();
    ctx.set("session_id", "test").unwrap();
    ctx.set("cmd_count", 0).unwrap();
    ctx.set("cwd", "/tmp").unwrap();
    ctx.set("command", "test").unwrap();
    ctx
}

#[test]
fn test_sift_lua_creation() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    assert!(sift.get::<mlua::Function>("exec").is_ok());
    assert!(sift.get::<mlua::Table>("log").is_ok());
    assert!(sift.get::<mlua::Table>("hash").is_ok());
    assert!(sift.get::<mlua::Table>("json").is_ok());
    assert!(sift.get::<mlua::Table>("meta").is_ok());
}

#[test]
fn test_sift_hash_sha256() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let hash: Table = sift.get("hash").unwrap();
    let sha256: mlua::Function = hash.get("sha256").unwrap();
    let result: String = sha256.call((test_ctx(&lua.lua), "hello")).unwrap();
    assert_eq!(
        result,
        "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
    );
}

#[test]
fn test_sift_token_count() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let token_count: mlua::Function = sift.get("token_count").unwrap();
    let result: isize = token_count
        .call((test_ctx(&lua.lua), "hello world"))
        .unwrap();
    assert_eq!(result, 2);
}

#[test]
fn test_sift_str_split_lines() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let str_tbl: Table = sift.get("str").unwrap();
    let split_lines: mlua::Function = str_tbl.get("split_lines").unwrap();

    // With trailing newline
    let result: Table = split_lines
        .call(("a\nb\nc\n",))
        .unwrap();
    assert_eq!(result.get::<String>(1).unwrap(), "a");
    assert_eq!(result.get::<String>(2).unwrap(), "b");
    assert_eq!(result.get::<String>(3).unwrap(), "c");
    assert_eq!(result.get::<String>(4).unwrap(), "");

    // Without trailing newline
    let result2: Table = split_lines
        .call(("a\nb\nc",))
        .unwrap();
    assert_eq!(result2.get::<String>(1).unwrap(), "a");
    assert_eq!(result2.get::<String>(2).unwrap(), "b");
    assert_eq!(result2.get::<String>(3).unwrap(), "c");
    assert!(result2.get::<String>(4).is_err());

    // Empty string
    let result3: Table = split_lines
        .call(("",))
        .unwrap();
    assert_eq!(result3.get::<String>(1).unwrap(), "");
}

#[test]
fn test_sift_str_slice_text() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let str_tbl: Table = sift.get("str").unwrap();
    let slice_text: mlua::Function = str_tbl.get("slice_text").unwrap();

    // Slice within bounds
    let result: String = slice_text
        .call(("a\nb\nc\nd", 2u64, 3u64))
        .unwrap();
    assert_eq!(result, "b\nc");

    // Slice past end
    let result2: String = slice_text
        .call(("a\nb", 5u64, 10u64))
        .unwrap();
    assert_eq!(result2, "");

    // Single line
    let result3: String = slice_text
        .call(("hello", 1u64, 1u64))
        .unwrap();
    assert_eq!(result3, "hello");
}

#[test]
fn test_sift_str_is_sensitive() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let str_tbl: Table = sift.get("str").unwrap();
    let is_sensitive: mlua::Function = str_tbl.get("is_sensitive").unwrap();

    // Sensitive paths
    assert!(is_sensitive
        .call::<bool>((".env.production",))
        .unwrap());
    assert!(is_sensitive
        .call::<bool>(("/path/to/key.pem",))
        .unwrap());
    assert!(is_sensitive
        .call::<bool>(("/path/to/.ssh/id_rsa",))
        .unwrap());

    // Non-sensitive paths
    assert!(!is_sensitive
        .call::<bool>(("/path/to/main.rs",))
        .unwrap());
    assert!(!is_sensitive
        .call::<bool>((test_ctx(&lua.lua), "/path/to/readme.md"))
        .unwrap());
}

#[test]
fn test_plugin_load_and_dispatch() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    let plugin_code = r#"
        return {
            name = "test-cmd",
            priority = 0,
            pattern = "test-cmd",
            execute = function(ctx, args, stdin)
                return { status = "handled", output = "test: " .. (args[1] or "none"), exit_code = 0 }
            end
        }
    "#;
    lua.load_plugin_from_str("test", plugin_code).unwrap();
    let (output, exit_code, plugin) = lua
        .dispatch("test-cmd", &["arg1".to_string()], None::<mlua::Value>)
        .unwrap();
    assert_eq!(output, "test: arg1");
    assert_eq!(exit_code, 0);
    assert_eq!(plugin, "test-cmd");
}

#[test]
fn test_plugin_dispatch_fallback() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    let plugin_code = r#"
        return {
            name = "__default__",
            priority = -1000,
            pattern = "__default__",
            execute = function(ctx, args, stdin)
                return { status = "handled", output = "fallback", exit_code = 0 }
            end
        }
    "#;
    lua.load_plugin_from_str("default", plugin_code).unwrap();
    let (output, _exit_code, _plugin) = lua.dispatch("unknown-cmd", &[], None::<mlua::Value>).unwrap();
    assert_eq!(output, "fallback");
}

#[test]
fn test_plugin_priority_ordering() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    let low = r#"return { name = "test", priority = -100, pattern = "test", execute = function() return { status = "handled", output = "low", exit_code = 0 } end }"#;
    let high = r#"return { name = "test", priority = 100, pattern = "test", execute = function() return { status = "handled", output = "high", exit_code = 0 } end }"#;
    lua.load_plugin_from_str("low", low).unwrap();
    lua.load_plugin_from_str("high", high).unwrap();
    let (output, _exit_code, _plugin) = lua.dispatch("test", &[], None::<mlua::Value>).unwrap();
    assert_eq!(output, "high");
}

#[test]
fn test_sift_fs_read() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let fs: Table = sift.get("fs").unwrap();
    let fs_read: mlua::Function = fs.get("read").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.txt");
    std::fs::write(&path, "hello world").unwrap();
    let content: String = fs_read
        .call((
            test_ctx(&lua.lua),
            path.display().to_string(),
            mlua::Value::Nil,
        ))
        .unwrap();
    assert_eq!(content, "hello world");
}

#[test]
fn test_sift_json_encode() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let json: Table = sift.get("json").unwrap();
    let encode: mlua::Function = json.get("encode").unwrap();
    let tbl = lua.lua.create_table().unwrap();
    tbl.set("name", "test").unwrap();
    let encoded: String = encode.call((test_ctx(&lua.lua), tbl)).unwrap();
    assert!(encoded.contains("name"));
}

#[test]
fn test_find_real_bash_exists() {
    let bash = find_real_bash();
    assert!(bash.exists(), "real bash should exist at {bash:?}");
}

#[test]
fn test_sift_fs_write_and_read() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let fs: Table = sift.get("fs").unwrap();
    let fs_write: mlua::Function = fs.get("write").unwrap();
    let fs_read: mlua::Function = fs.get("read").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.txt").display().to_string();
    fs_write
        .call::<()>((test_ctx(&lua.lua), path.clone(), "hello world"))
        .unwrap();
    let content: String = fs_read
        .call((test_ctx(&lua.lua), path, mlua::Value::Nil))
        .unwrap();
    assert_eq!(content, "hello world");
}

#[test]
fn test_sift_fs_stat() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let fs: Table = sift.get("fs").unwrap();
    let fs_stat: mlua::Function = fs.get("stat").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.txt");
    std::fs::write(&path, "hello").unwrap();
    let result: Table = fs_stat
        .call((test_ctx(&lua.lua), path.display().to_string()))
        .unwrap();
    let is_file: bool = result.get("is_file").unwrap();
    assert!(is_file);
}

#[test]
fn test_sift_fs_exists() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let fs: Table = sift.get("fs").unwrap();
    let fs_exists: mlua::Function = fs.get("exists").unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.txt");
    std::fs::write(&path, "hello").unwrap();
    assert!(fs_exists
        .call::<bool>((test_ctx(&lua.lua), path.display().to_string()))
        .unwrap());
    assert!(!fs_exists
        .call::<bool>((
            test_ctx(&lua.lua),
            dir.path().join("nonexistent").display().to_string()
        ))
        .unwrap());
}

#[test]
fn test_sift_json_shortest_raw_wins_small() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let json: Table = sift.get("json").unwrap();
    let shortest: mlua::Function = json.get("shortest").unwrap();

    let formats = lua.lua.create_table().unwrap();
    let json_opts = lua.lua.create_table().unwrap();
    json_opts.set("max_string_len", 80).unwrap();
    formats.set("json", json_opts).unwrap();
    formats.set("toon", true).unwrap();

    let small = r#"{"name":"test","value":42}"#;
    let result: String = shortest.call((test_ctx(&lua.lua), small, formats)).unwrap();
    assert_eq!(result, small, "raw should win for small JSON");
}

#[test]
fn test_sift_json_shortest_non_json() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let json: Table = sift.get("json").unwrap();
    let shortest: mlua::Function = json.get("shortest").unwrap();

    let formats = lua.lua.create_table().unwrap();
    formats.set("toon", true).unwrap();

    let result: String = shortest
        .call((test_ctx(&lua.lua), "not json", formats))
        .unwrap();
    assert_eq!(result, "not json");
}

#[test]
fn test_sift_json_shortest_empty() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let json: Table = sift.get("json").unwrap();
    let shortest: mlua::Function = json.get("shortest").unwrap();

    let formats = lua.lua.create_table().unwrap();
    formats.set("toon", true).unwrap();

    let result: String = shortest.call((test_ctx(&lua.lua), "{}", formats)).unwrap();
    assert_eq!(result, "{}");
}

#[test]
fn test_sift_json_shortest_tiny_json() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let json: Table = sift.get("json").unwrap();
    let shortest: mlua::Function = json.get("shortest").unwrap();

    let formats = lua.lua.create_table().unwrap();
    formats.set("toon", true).unwrap();

    let result: String = shortest.call((test_ctx(&lua.lua), "42", formats)).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_sift_json_shortest_large_json_toon_wins() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let json: Table = sift.get("json").unwrap();
    let shortest: mlua::Function = json.get("shortest").unwrap();

    let formats = lua.lua.create_table().unwrap();
    formats.set("toon", true).unwrap();

    let mut items = Vec::new();
    for i in 0..100 {
        items.push(serde_json::json!({
            "name": format!("item-{}", i),
            "value": i,
            "description": "a long string that takes up many tokens in json because of quotes and commas"
        }));
    }
    let large = serde_json::json!({"items": items});
    let large_str = serde_json::to_string(&large).unwrap();
    assert!(large_str.len() > 2000, "large JSON should be >2000 chars");

    let result: String = shortest
        .call((test_ctx(&lua.lua), large_str, formats))
        .unwrap();
    assert!(
        !result.is_empty(),
        "shortest should return non-empty output"
    );
    let is_toon = result.contains(':') && result.contains("  ");
    assert!(
        is_toon,
        "large JSON should produce TOON output, got: {}",
        &result[..200.min(result.len())]
    );
}

#[test]
fn test_sift_toon_encode() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let toon: Table = sift.get("toon").unwrap();
    let encode: mlua::Function = toon.get("encode").unwrap();
    let tbl = lua.lua.create_table().unwrap();
    tbl.set("name", "test").unwrap();
    let encoded: String = encode.call((test_ctx(&lua.lua), tbl)).unwrap();
    assert!(encoded.contains("name"));
}

#[test]
fn test_sift_env() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let env: Table = sift.get("env").unwrap();
    let env_set: mlua::Function = env.get("set").unwrap();
    let env_get: mlua::Function = env.get("get").unwrap();
    env_set
        .call::<()>((test_ctx(&lua.lua), "SIFT_TEST", "val"))
        .unwrap();
    let result: Option<String> = env_get.call((test_ctx(&lua.lua), "SIFT_TEST")).unwrap();
    assert_eq!(result, Some("val".to_string()));
}

#[test]
fn test_sift_meta() {
    let ctx = SiftContext {
        cwd: std::env::current_dir().unwrap(),
        cmd_count: 42,
        env: HashMap::new(),
        session_id: Some("test-session".to_string()),
        raw_bytes: 100,
        filtered_bytes: 50,
    };
    let lua = SiftLua::new(None, ctx).unwrap();
    let sift: Table = lua.lua.globals().get("sift").unwrap();
    let meta: Table = sift.get("meta").unwrap();
    let session_id: String = meta.get("session_id").unwrap();
    assert_eq!(session_id, "test-session");
    let cmd_count: i64 = meta.get("cmd_count").unwrap();
    assert_eq!(cmd_count, 42);
}

#[test]
fn test_exec_command() {
    let (stdout, stderr, code) = exec_command("echo hello", "test", 0, None).unwrap();
    assert!(
        stdout.contains("hello"),
        "stdout should contain hello, got: {stdout}"
    );
    assert!(stderr.is_empty(), "stderr should be empty, got: {stderr}");
    assert_eq!(code, 0);
}

#[test]
fn test_exec_command_with_stderr() {
    let (stdout, stderr, code) = exec_command("echo out && echo err >&2", "test", 0, None).unwrap();
    assert!(
        stdout.contains("out"),
        "stdout should contain out, got: {stdout}"
    );
    assert!(
        stderr.contains("err"),
        "stderr should contain err, got: {stderr}"
    );
    assert_eq!(code, 0);
}

#[test]
fn test_exec_command_exit_code() {
    let (_stdout, _stderr, code) = exec_command("exit 42", "test", 0, None).unwrap();
    assert_eq!(code, 42, "exit code should be 42, got {code}");
}

#[test]
fn test_save_output() {
    let session_id = "test-save";
    save_output("echo test", session_id, 1, "test content");
    let tmp_dir = std::path::PathBuf::from("/tmp/sift").join(session_id);
    let has_files = std::fs::read_dir(&tmp_dir).is_ok();
    assert!(has_files, "should have saved output files");
    let _ = std::fs::remove_dir_all(&tmp_dir);
}

#[test]
fn test_dispatch_full_simple() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    let plugin_code = r#"
        return {
            name = "test-cmd",
            priority = 0,
            pattern = "test-cmd",
            execute = function(ctx, args, stdin)
                return { status = "handled", output = "ok", exit_code = 0 }
            end
        }
    "#;
    lua.load_plugin_from_str("test", plugin_code).unwrap();
    let (output, exit_code, _plugin) = lua.dispatch_full("test-cmd arg1", None::<mlua::Value>).unwrap();
    assert_eq!(output, "ok");
    assert_eq!(exit_code, 0);
}

#[test]
fn test_dispatch_full_empty() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let (output, exit_code, _plugin) = lua.dispatch_full("", None::<mlua::Value>).unwrap();
    assert_eq!(output, "");
    assert_eq!(exit_code, 0);
}

#[test]
fn test_split_pipeline_simple() {
    let segments = super::api::split_pipeline("echo abc | cat");
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0], "echo abc");
    assert_eq!(segments[1], "cat");
}

#[test]
fn test_split_pipeline_logical_or() {
    let segments = super::api::split_pipeline("false || echo ok");
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0], "false || echo ok");
}

#[test]
fn test_split_pipeline_no_pipe() {
    let segments = super::api::split_pipeline("cat foo.rs");
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0], "cat foo.rs");
}

#[test]
fn test_dispatch_full_cd_prefix() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    let plugin_code = r#"
        return {
            name = "test-cmd",
            priority = 0,
            pattern = "test-cmd",
            execute = function(ctx, args, stdin)
                return { status = "handled", output = "cd-dispatched", exit_code = 0 }
            end
        }
    "#;
    lua.load_plugin_from_str("test", plugin_code).unwrap();
    let (output, exit_code, _plugin) = lua.dispatch_full("cd /tmp && test-cmd", None::<mlua::Value>).unwrap();
    assert_eq!(output, "cd-dispatched");
    assert_eq!(exit_code, 0);
}

#[test]
fn test_dispatch_full_pipeline_fallback() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    let default_code = r#"
        return {
            name = "__default__",
            priority = -1000,
            pattern = "__default__",
            execute = function(ctx, args, stdin)
                return { status = "handled", output = "fallback", exit_code = 0 }
            end
        }
    "#;
    lua.load_plugin_from_str("default", default_code).unwrap();
    let (output, exit_code, _plugin) = lua.dispatch_full("echo hello | grep hello", None::<mlua::Value>).unwrap();
    assert_eq!(output, "fallback");
    assert_eq!(exit_code, 0);
}

#[test]
fn test_dispatch_full_popd() {
    let lua = SiftLua::new(None, test_context()).unwrap();
    let (output, exit_code, _plugin) = lua.dispatch_full("popd", None::<mlua::Value>).unwrap();
    assert_eq!(output, "");
    assert_eq!(exit_code, 0);
}

#[test]
fn test_dispatch_unchanged_nudge() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    let plugin_code = r#"
        return {
            name = "test-cmd",
            priority = 0,
            pattern = "test-cmd",
            execute = function(ctx, args, stdin)
                sift.nudge(ctx, "bypass: 'command cat foo.rs'")
                return { status = "unchanged", message = "[sift] foo.rs unchanged since last read" }
            end
        }
    "#;
    lua.load_plugin_from_str("test", plugin_code).unwrap();
    let (output, exit_code, _plugin) = lua.dispatch("test-cmd", &[], None::<mlua::Value>).unwrap();
    assert!(
        output.contains("[sift] foo.rs unchanged since last read"),
        "output: {output}"
    );
    assert!(
        output.contains("bypass: 'command cat foo.rs'"),
        "output: {output}"
    );
    assert_eq!(exit_code, 0);
}

#[test]
fn test_stdin_reader_file_redirect() {
    use std::io::Write;
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test_stdin.txt");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "line1").unwrap();
    writeln!(f, "line2").unwrap();
    writeln!(f, "line3").unwrap();
    drop(f);

    let mut lua = SiftLua::new(None, test_context()).unwrap();
    let plugin_code = r#"
        return {
            name = "stdin-cat",
            priority = 0,
            pattern = "stdin-cat",
            execute = function(ctx, args, stdin)
                local result = {}
                if stdin ~= nil then
                    local line = stdin:readline()
                    while line ~= nil do
                        table.insert(result, line)
                        line = stdin:readline()
                    end
                end
                return { status = "handled", output = table.concat(result, "\n"), exit_code = 0 }
            end
        }
    "#;
    lua.load_plugin_from_str("stdin-cat", plugin_code).unwrap();

    // Use dispatch_full with < file redirect
    let cmd = format!("stdin-cat < {}", path.display());
    let (output, exit_code, _plugin) = lua.dispatch_full(&cmd, None::<mlua::Value>).unwrap();
    assert_eq!(exit_code, 0, "output: {output}");
    assert!(output.contains("line1"), "output: {output}");
    assert!(output.contains("line2"), "output: {output}");
    assert!(output.contains("line3"), "output: {output}");
}

#[test]
fn test_stdin_reader_pipeline() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    let plugin_code = r#"
        return {
            name = "stdin-cap",
            priority = 0,
            pattern = "stdin-cap",
            execute = function(ctx, args, stdin)
                local result = {}
                if stdin ~= nil then
                    local line = stdin:readline()
                    while line ~= nil do
                        table.insert(result, string.upper(line))
                        line = stdin:readline()
                    end
                end
                return { status = "handled", output = table.concat(result, "\n"), exit_code = 0 }
            end
        }
    "#;
    lua.load_plugin_from_str("stdin-cap", plugin_code).unwrap();

    // Pipeline: echo pipes to stdin-cap
    let (output, exit_code, _plugin) = lua.dispatch_full("echo hello | stdin-cap", None::<mlua::Value>).unwrap();
    assert_eq!(exit_code, 0, "output: {output}");
    assert!(output.contains("HELLO"), "output: {output}");
}

mod tests_cache;
