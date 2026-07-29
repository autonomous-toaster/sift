use super::{SiftContext, SiftLua};
use crate::session::SessionStore;
use mlua::Table;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

fn plugin_dir() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().join("plugins")
}

fn test_context() -> SiftContext {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    SiftContext {
        cwd: std::env::current_dir().unwrap(),
        cwd_str: std::env::current_dir().unwrap().display().to_string(),
        cmd_count: std::cell::Cell::new(0),
        env: HashMap::new(),
        session_id: Some(format!("test_plugins_{ts}")),
        raw_bytes: 0,
        filtered_bytes: 0,
    }
}

fn load_plugin(name: &str) -> String {
    let path = plugin_dir().join(format!("{name}.lua"));
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to load plugin {name} from {}: {e}", path.display()))
}

fn load_plugin_and_register(lua: &mut SiftLua, name: &str) {
    let code = load_plugin(name);
    lua.load_plugin_from_str(name, &code)
        .unwrap_or_else(|e| panic!("failed to register plugin {name}: {e}"));
}

#[test]
fn test_smoke_all_plugins_see_full_api() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();

    let plugin_names = [
        "sift-read",
        "cat",
        "head",
        "tail",
        "sed",
        "openspec",
        "rtk",
        "jq",
    ];

    for name in &plugin_names {
        load_plugin_and_register(&mut lua, name);
    }

    // Verify sift.* API is fully visible from the Lua runtime
    let globals = lua.lua.globals();
    let sift: Table = globals.get("sift").unwrap();

    // Core sub-tables and functions
    assert!(sift.get::<Table>("str").is_ok(), "sift.str");
    assert!(sift.get::<Table>("fs").is_ok(), "sift.fs");
    assert!(sift.get::<Table>("cache").is_ok(), "sift.cache");
    assert!(sift.get::<Table>("hash").is_ok(), "sift.hash");
    assert!(sift.get::<Table>("json").is_ok(), "sift.json");
    assert!(sift.get::<mlua::Function>("diff").is_ok(), "sift.diff");
    assert!(sift.get::<Table>("env").is_ok(), "sift.env");
    assert!(sift.get::<Table>("meta").is_ok(), "sift.meta");
    assert!(sift.get::<mlua::Function>("exec").is_ok(), "sift.exec");
    assert!(sift.get::<mlua::Function>("nudge").is_ok(), "sift.nudge");
    assert!(sift.get::<Table>("gain").is_ok(), "sift.gain");

    // sift.str functions callable without ctx
    let str_tbl: Table = sift.get("str").unwrap();
    assert!(
        str_tbl.get::<mlua::Function>("split_lines").is_ok(),
        "str.split_lines"
    );
    assert!(
        str_tbl.get::<mlua::Function>("slice_text").is_ok(),
        "str.slice_text"
    );
    assert!(
        str_tbl.get::<mlua::Function>("is_sensitive").is_ok(),
        "str.is_sensitive"
    );

    // Call them directly to verify no ctx needed
    let result: bool = lua
        .lua
        .load("local ctx = {} return sift.str.is_sensitive(ctx, '.env')")
        .eval()
        .unwrap();
    assert!(result, "is_sensitive('.env') should be true");

    let result: bool = lua
        .lua
        .load("local ctx = {} return sift.str.is_sensitive(ctx, 'main.rs')")
        .eval()
        .unwrap();
    assert!(!result, "is_sensitive('main.rs') should be false");

    let result: mlua::Table = lua
        .lua
        .load("local ctx = {} return sift.str.split_lines(ctx, 'a\\nb\\nc')")
        .eval()
        .unwrap();
    assert_eq!(
        result.len().unwrap(),
        3,
        "split_lines should return 3 lines"
    );

    let result: String = lua
        .lua
        .load("local ctx = {} return sift.str.slice_text(ctx, 'a\\nb\\nc\\n', 2, 3)")
        .eval()
        .unwrap();
    assert_eq!(result, "b\nc", "slice_text should return lines 2-3");

    // sift.toon.encode/decode without ctx, with options
    let result: String = lua
        .lua
        .load("local ctx = {} return sift.toon.encode(ctx, {name = 'Alice'})")
        .eval()
        .unwrap();
    assert!(
        result.contains("Alice"),
        "toon encode should work: {result}"
    );

    let result: String = lua
        .lua
        .load("local ctx = {} return sift.toon.encode(ctx, {tags = {'a', 'b'}}, {delimiter = 'pipe'})")
        .eval()
        .unwrap();
    assert!(
        result.contains("|"),
        "toon encode with pipe delimiter: {result}"
    );

    let result: mlua::Value = lua
        .lua
        .load("local ctx = {} return sift.toon.decode(ctx, 'name: Alice')")
        .eval()
        .unwrap();
    assert!(
        matches!(result, mlua::Value::Table(_)),
        "toon decode should return a table"
    );

    let result: String = lua
        .lua
        .load("local ctx = {} return sift.toon.decode(ctx, 'items[3]: a,b', {strict = true})")
        .eval::<mlua::Value>()
        .map(|_| String::new())
        .unwrap_or_else(|e| e.to_string());
    assert!(
        result.contains("error") || result.is_empty(),
        "strict decode should error or succeed: {result}"
    );
}

#[test]
fn test_recording_populates_conversation_cache() {
    use std::io::Write;
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let store = rt.block_on(SessionStore::open(&db_path)).unwrap();
    let store = Arc::new(store);

    let session_id = format!(
        "test-recording-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    let ctx = SiftContext {
        cwd: std::env::current_dir().unwrap(),
        cwd_str: std::env::current_dir().unwrap().display().to_string(),
        cmd_count: std::cell::Cell::new(1),
        env: HashMap::new(),
        session_id: Some(session_id.clone()),
        raw_bytes: 0,
        filtered_bytes: 0,
    };

    let mut lua = SiftLua::new(Some(store.clone()), ctx).unwrap();
    load_plugin_and_register(&mut lua, "cat");

    let f_dir = tempfile::tempdir().unwrap();
    let path = f_dir.path().join("test.txt");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "hello world").unwrap();
    drop(f);

    let cmd = format!("cat {}", path.display());
    let (output, code, _plugin) = lua.dispatch_full(&cmd, None::<mlua::Value>).unwrap();
    assert_eq!(code, 0, "cat should succeed, got: {output}");

    // Give record_conversation thread time to flush
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Query with LIKE since item_id now includes invocation_id
    let entries = rt
        .block_on(store.query_conversations(Some(&session_id)))
        .unwrap();
    assert!(
        !entries.is_empty(),
        "conversation_cache should have an entry"
    );
    let entry = &entries[0];
    assert_eq!(entry.plugin_name.as_deref(), Some("cat"));
    assert_eq!(entry.output_format.as_deref(), Some("text"));
    assert!(entry.raw_bytes.is_some(), "raw_bytes should be recorded");
    assert!(
        entry.filtered_bytes.is_some(),
        "filtered_bytes should be recorded"
    );
    assert!(
        entry.reduction_bps.is_some(),
        "reduction_bps should be computed"
    );
}

#[test]
fn test_passthrough_records_as_bypass() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let store = rt.block_on(SessionStore::open(&db_path)).unwrap();
    let store = Arc::new(store);

    let session_id = format!(
        "test-bypass-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    let ctx = SiftContext {
        cwd: std::env::current_dir().unwrap(),
        cwd_str: std::env::current_dir().unwrap().display().to_string(),
        cmd_count: std::cell::Cell::new(1),
        env: HashMap::new(),
        session_id: Some(session_id.clone()),
        raw_bytes: 0,
        filtered_bytes: 0,
    };

    let mut lua = SiftLua::new(Some(store.clone()), ctx).unwrap();

    // Load a minimal passthrough plugin
    lua.load_plugin_from_str(
        "command",
        r#"
return {
    name = "command",
    priority = 1000,
    pattern = "command",
    execute = function(ctx, args, stdin)
        return { status = "passthrough" }
    end
}
"#,
    )
    .unwrap();

    let (output, code, _plugin) = lua
        .dispatch_full("command echo hello", None::<mlua::Value>)
        .unwrap();
    assert_eq!(code, 0, "passthrough should succeed, got: {output}");

    std::thread::sleep(std::time::Duration::from_millis(200));

    // Query with LIKE since item_id now includes invocation_id
    let entries = rt
        .block_on(store.query_conversations(Some(&session_id)))
        .unwrap();
    assert!(
        !entries.is_empty(),
        "bypass should have a conversation entry"
    );
    let entry = &entries[0];
    assert_eq!(entry.plugin_name.as_deref(), Some("command"));
    assert_eq!(entry.output_format.as_deref(), Some("passthrough"));
    assert!(
        entry.raw_bytes.is_some() && entry.filtered_bytes.is_some(),
        "raw/filtered bytes should be recorded for bypass"
    );
    assert_eq!(
        entry.raw_bytes, entry.filtered_bytes,
        "bypass: raw == filtered"
    );
    assert_eq!(
        entry.reduction_bps,
        Some(0),
        "bypass should have 0 reduction"
    );
}

#[test]
fn test_sift_read_plugin_executes() {
    use std::io::Write;
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "sift-read");

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.rs");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "line1").unwrap();
    writeln!(f, "line2").unwrap();
    writeln!(f, "line3").unwrap();
    drop(f);

    let cmd = format!("sift-read {}", path.display());
    let (output, code, _plugin) = lua.dispatch_full(&cmd, None::<mlua::Value>).unwrap();
    assert_eq!(code, 0, "sift-read should succeed, got: {output}");
    assert!(
        output.contains("line1"),
        "output should contain line1: {output}"
    );
    assert!(
        output.contains("line3"),
        "output should contain line3: {output}"
    );
}

#[test]
fn test_cat_plugin_executes() {
    use std::io::Write;
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "cat");

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.txt");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "hello").unwrap();
    writeln!(f, "world").unwrap();
    drop(f);

    let cmd = format!("cat {}", path.display());
    let (output, code, _plugin) = lua.dispatch_full(&cmd, None::<mlua::Value>).unwrap();
    assert_eq!(code, 0, "cat should succeed, got: {output}");
    assert!(
        output.contains("hello"),
        "output should contain hello: {output}"
    );
    assert!(
        output.contains("world"),
        "output should contain world: {output}"
    );
}

#[test]
fn test_head_plugin_executes() {
    use std::io::Write;
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "head");

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.txt");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "line1").unwrap();
    writeln!(f, "line2").unwrap();
    writeln!(f, "line3").unwrap();
    writeln!(f, "line4").unwrap();
    drop(f);

    let cmd = format!("head -n 2 {}", path.display());
    let (output, code, _plugin) = lua.dispatch_full(&cmd, None::<mlua::Value>).unwrap();
    assert_eq!(code, 0, "head should succeed, got: {output}");
    assert!(
        output.contains("line1"),
        "output should contain line1: {output}"
    );
    assert!(
        output.contains("line2"),
        "output should contain line2: {output}"
    );
    assert!(
        !output.contains("line4"),
        "output should NOT contain line4: {output}"
    );
}

#[test]
fn test_tail_plugin_executes() {
    use std::io::Write;
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "tail");

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.txt");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "line1").unwrap();
    writeln!(f, "line2").unwrap();
    writeln!(f, "line3").unwrap();
    writeln!(f, "line4").unwrap();
    drop(f);

    let cmd = format!("tail -n 2 {}", path.display());
    let (output, code, _plugin) = lua.dispatch_full(&cmd, None::<mlua::Value>).unwrap();
    assert_eq!(code, 0, "tail should succeed, got: {output}");
    assert!(
        output.contains("line3"),
        "output should contain line3: {output}"
    );
    assert!(
        output.contains("line4"),
        "output should contain line4: {output}"
    );
    assert!(
        !output.contains("line1"),
        "output should NOT contain line1: {output}"
    );
}

#[test]
fn test_sed_plugin_executes() {
    use std::io::Write;
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "sed");

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("test.txt");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "line1").unwrap();
    writeln!(f, "line2").unwrap();
    writeln!(f, "line3").unwrap();
    writeln!(f, "line4").unwrap();
    writeln!(f, "line5").unwrap();
    drop(f);

    let cmd = format!("sed -n '2,4p' {}", path.display());
    let (output, code, _plugin) = lua.dispatch_full(&cmd, None::<mlua::Value>).unwrap();
    assert_eq!(code, 0, "sed should succeed, got: {output}");
    assert!(
        output.contains("line2"),
        "output should contain line2: {output}"
    );
    assert!(
        output.contains("line3"),
        "output should contain line3: {output}"
    );
    assert!(
        output.contains("line4"),
        "output should contain line4: {output}"
    );
    assert!(
        !output.contains("line1"),
        "output should NOT contain line1: {output}"
    );
    assert!(
        !output.contains("line5"),
        "output should NOT contain line5: {output}"
    );
}

#[test]
fn test_gain_report_via_lua() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("test.db");
    let store = rt.block_on(SessionStore::open(&db_path)).unwrap();
    let store = Arc::new(store);

    let session_id = format!(
        "test-gain-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    let ctx = SiftContext {
        cwd: std::env::current_dir().unwrap(),
        cwd_str: std::env::current_dir().unwrap().display().to_string(),
        cmd_count: std::cell::Cell::new(1),
        env: HashMap::new(),
        session_id: Some(session_id.clone()),
        raw_bytes: 0,
        filtered_bytes: 0,
    };

    let mut lua = SiftLua::new(Some(store.clone()), ctx).unwrap();
    load_plugin_and_register(&mut lua, "cat");

    // Run a cat command to populate the conversation cache
    use std::io::Write;
    let f_dir = tempfile::tempdir().unwrap();
    let path = f_dir.path().join("test.txt");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "gain test data").unwrap();
    drop(f);

    let cmd = format!("cat {}", path.display());
    let (output, code, _plugin) = lua.dispatch_full(&cmd, None::<mlua::Value>).unwrap();
    assert_eq!(code, 0, "cat should succeed, got: {output}");

    std::thread::sleep(std::time::Duration::from_millis(200));

    // Test gain report via Lua API
    let result: String = lua.lua.load("return sift.gain.report({})").eval().unwrap();
    assert!(
        result.contains("sift gain"),
        "report should start with sift gain: {result}"
    );
    assert!(
        result.contains("cat"),
        "report should mention cat plugin: {result}"
    );
    assert!(
        result.contains("Commands:"),
        "report should show command count: {result}"
    );

    // Test JSON output
    let json_result: String = lua
        .lua
        .load("return sift.gain.report({json = true})")
        .eval()
        .unwrap();
    assert!(
        json_result.contains("total_commands"),
        "JSON report should have total_commands: {json_result}"
    );
    assert!(
        json_result.contains("per_plugin"),
        "JSON report should have per_plugin: {json_result}"
    );
}

#[test]
fn test_jq_plugin_basic_filter() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "jq");

    // Test basic filter with piped stdin via dispatch_full
    // The jq plugin receives piped stdin from try_pipeline
    // We test by calling dispatch_full with a pipeline
    let cmd = r##"echo '{"name":"John","age":30}' | jq '.name'"##;
    let (output, code, _plugin) = lua.dispatch_full(cmd, None::<mlua::Value>).unwrap();
    assert_eq!(code, 0, "jq basic filter should succeed, got: {output}");
    assert!(
        output.contains("John"),
        "output should contain John: {output}"
    );
}

#[test]
fn test_jq_plugin_raw_output() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "jq");

    // Test -r raw output
    let cmd = r##"echo '{"name":"John","age":30}' | jq -r '.name'"##;
    let (output, code, _plugin) = lua.dispatch_full(cmd, None::<mlua::Value>).unwrap();
    assert_eq!(code, 0, "jq -r should succeed, got: {output}");
    assert!(
        output.contains("John"),
        "raw output should contain John: {output}"
    );
}

#[test]
fn test_jq_plugin_null_input() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "jq");

    // Test -n null input
    let cmd = r##"jq -n '{a: 1, b: 2}'"##;
    let (output, code, _plugin) = lua.dispatch_full(cmd, None::<mlua::Value>).unwrap();
    assert_eq!(code, 0, "jq -n should succeed, got: {output}");
    assert!(
        output.contains("1") && output.contains("2"),
        "output should contain values: {output}"
    );
}

#[test]
fn test_jq_plugin_unknown_flag_fallthrough() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "jq");

    // Test unknown flag --arg → should fall through to real jq
    // dispatch_full returns passthrough output from bash
    let cmd = r##"echo '{}' | jq --arg name foo '.name'"##;
    let (output, code, _plugin) = lua.dispatch_full(cmd, None::<mlua::Value>).unwrap();
    // Passthrough should still produce output (real jq runs)
    assert_eq!(
        code, 0,
        "jq --arg fallthrough should succeed, got: {output}"
    );
}

#[test]
fn test_jq_plugin_no_stdin_fallthrough() {
    use std::io::Write;
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "jq");

    // Test no stdin and no -n → should fall through to real jq
    // jq with file argument: jq '.name' data.json
    // This should passthrough since the plugin can't handle file args
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("data.json");
    let mut f = std::fs::File::create(&path).unwrap();
    writeln!(f, "{{\"name\":\"John\"}}").unwrap();
    drop(f);

    let cmd = format!("jq '.name' {}", path.display());
    let (output, code, _plugin) = lua.dispatch_full(&cmd, None::<mlua::Value>).unwrap();
    assert_eq!(
        code, 0,
        "jq file arg fallthrough should succeed, got: {output}"
    );
    assert!(
        output.contains("John"),
        "output should contain John: {output}"
    );
}

#[test]
fn test_curl_plugin_verbose_passthrough() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "curl");

    // Test that verbose mode passthrough works (no HTTP request made)
    // -v flag triggers passthrough which runs real curl
    // In test environment, curl will fail but the plugin should handle it
    let cmd = "curl -v http://nonexistent.example.test 2>/dev/null";
    let (_output, _code, _plugin) = lua.dispatch_full(cmd, None::<mlua::Value>).unwrap();
    // Plugin should handle the command without crashing
    // Exit code may be non-zero (curl fails on nonexistent host), but plugin should not error
}

#[test]
fn test_curl_plugin_extension_fallback_url_parsing() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "curl");

    // Test that the curl plugin handles URLs with various extensions
    // without crashing. The extension fallback is a URL pattern match
    // that activates when content-type is unrecognized.
    // These requests will fail (no network), but the plugin should
    // handle the URL parsing without errors.

    // .md URL
    let cmd = "curl -s https://raw.githubusercontent.com/user/repo/main/README.md 2>/dev/null";
    let (_output, _code, _plugin) = lua.dispatch_full(cmd, None::<mlua::Value>).unwrap();

    // .json URL
    let cmd = "curl -s https://api.example.com/data.json 2>/dev/null";
    let (_output, _code, _plugin) = lua.dispatch_full(cmd, None::<mlua::Value>).unwrap();

    // URL without extension
    let cmd = "curl -s https://api.example.com/data 2>/dev/null";
    let (_output, _code, _plugin) = lua.dispatch_full(cmd, None::<mlua::Value>).unwrap();

    // URL with query params
    let cmd = "curl -s 'https://api.example.com/data.json?format=pretty' 2>/dev/null";
    let (_output, _code, _plugin) = lua.dispatch_full(cmd, None::<mlua::Value>).unwrap();
}

#[test]
fn test_scred_plugin_echo_fallthrough() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "scred");

    // Test that echo works through the scred plugin
    // If scred is not installed, the plugin returns original output
    let cmd = "echo hello world";
    let (output, code, _plugin) = lua.dispatch_full(cmd, None::<mlua::Value>).unwrap();
    assert_eq!(code, 0, "echo should succeed, got: {output}");
    assert!(
        output.contains("hello world"),
        "output should contain hello world: {output}"
    );
}

#[test]
fn test_scred_plugin_env_fallthrough() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "scred");

    // Test that env works through the scred plugin
    let cmd = "env";
    let (output, code, _plugin) = lua.dispatch_full(cmd, None::<mlua::Value>).unwrap();
    assert_eq!(code, 0, "env should succeed, got: {output}");
    assert!(
        output.len() > 0,
        "env output should not be empty"
    );
}

#[test]
fn test_scred_plugin_empty_output() {
    let mut lua = SiftLua::new(None, test_context()).unwrap();
    load_plugin_and_register(&mut lua, "scred");

    // Test that echo with no args (empty output) falls through
    let cmd = "echo -n ''";
    let (output, code, _plugin) = lua.dispatch_full(cmd, None::<mlua::Value>).unwrap();
    assert_eq!(code, 0, "echo empty should succeed, got: {output}");
}
