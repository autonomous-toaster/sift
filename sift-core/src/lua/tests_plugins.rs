use super::{SiftContext, SiftLua};
use mlua::Table;
use std::collections::HashMap;
use std::path::PathBuf;

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
        cmd_count: 0,
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
        "sift-read", "cat", "head", "tail", "sed", "openspec", "rtk",
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

    // sift.str functions callable without ctx
    let str_tbl: Table = sift.get("str").unwrap();
    assert!(str_tbl.get::<mlua::Function>("split_lines").is_ok(), "str.split_lines");
    assert!(str_tbl.get::<mlua::Function>("slice_text").is_ok(), "str.slice_text");
    assert!(str_tbl.get::<mlua::Function>("is_sensitive").is_ok(), "str.is_sensitive");

    // Call them directly to verify no ctx needed
    let result: bool = lua
        .lua
        .load("return sift.str.is_sensitive('.env')")
        .eval()
        .unwrap();
    assert!(result, "is_sensitive('.env') should be true");

    let result: bool = lua
        .lua
        .load("return sift.str.is_sensitive('main.rs')")
        .eval()
        .unwrap();
    assert!(!result, "is_sensitive('main.rs') should be false");

    let result: mlua::Table = lua
        .lua
        .load("return sift.str.split_lines('a\\nb\\nc')")
        .eval()
        .unwrap();
    assert_eq!(result.len().unwrap(), 3, "split_lines should return 3 lines");

    let result: String = lua
        .lua
        .load("return sift.str.slice_text('a\\nb\\nc\\n', 2, 3)")
        .eval()
        .unwrap();
    assert_eq!(result, "b\nc", "slice_text should return lines 2-3");
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
    assert!(output.contains("line1"), "output should contain line1: {output}");
    assert!(output.contains("line3"), "output should contain line3: {output}");
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
    assert!(output.contains("hello"), "output should contain hello: {output}");
    assert!(output.contains("world"), "output should contain world: {output}");
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
    assert!(output.contains("line1"), "output should contain line1: {output}");
    assert!(output.contains("line2"), "output should contain line2: {output}");
    assert!(!output.contains("line4"), "output should NOT contain line4: {output}");
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
    assert!(output.contains("line3"), "output should contain line3: {output}");
    assert!(output.contains("line4"), "output should contain line4: {output}");
    assert!(!output.contains("line1"), "output should NOT contain line1: {output}");
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
    assert!(output.contains("line2"), "output should contain line2: {output}");
    assert!(output.contains("line3"), "output should contain line3: {output}");
    assert!(output.contains("line4"), "output should contain line4: {output}");
    assert!(!output.contains("line1"), "output should NOT contain line1: {output}");
    assert!(!output.contains("line5"), "output should NOT contain line5: {output}");
}