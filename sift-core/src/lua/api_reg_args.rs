//! API registration for `sift.args.*` — declarative argument parsing for Lua plugins.
//!
//! `sift.args.parse(args, spec)` replaces ad-hoc manual parsers in plugins
//! with a declarative spec table. Returns a parsed result table on success,
//! or `nil, error_string` on failure.

use super::SiftLua;
use anyhow::Result;
use mlua::{Lua, Table, Value};

// ---- Internal types for parsed spec ----

struct FlagEntry {
    name: String,
    flag_type: String,
}

struct PosSpec {
    name: String,
    required: bool,
    pos_type: String,
}

// ---- Internal helpers ----

fn coerce_value(lua: &Lua, raw: &str, ty: &str) -> Result<Value, String> {
    match ty {
        "int" => {
            let n: i64 = raw.parse().map_err(|_| format!("invalid integer: {raw}"))?;
            Ok(Value::Integer(n))
        }
        "str" => Ok(Value::String(
            lua.create_string(raw)
                .map_err(|e| format!("create string: {e}"))?,
        )),
        _ => Ok(Value::String(
            lua.create_string(raw)
                .map_err(|e| format!("create string: {e}"))?,
        )),
    }
}

fn lookup_flag<'a>(flag_map: &'a [(String, FlagEntry)], alias: &str) -> Option<&'a FlagEntry> {
    flag_map.iter().find(|(a, _)| a == alias).map(|(_, e)| e)
}

fn build_flag_map(flags: &Table) -> Result<Vec<(String, FlagEntry)>, mlua::Error> {
    let mut flag_map: Vec<(String, FlagEntry)> = Vec::new();
    for pair in flags.pairs::<String, Table>() {
        let (name, flag_tbl) = pair?;
        let mut flag_type = "boolean".to_string();
        let mut aliases: Vec<String> = Vec::new();

        for entry_pair in flag_tbl.pairs::<Value, Value>() {
            let (k, v) = entry_pair?;
            match k {
                Value::Integer(_) => {
                    if let Value::String(s) = v {
                        aliases.push(s.to_str()?.to_string());
                    }
                }
                Value::String(ref s) if s.to_str()? == "type" => {
                    if let Value::String(t) = v {
                        flag_type = t.to_str()?.to_string();
                    }
                }
                _ => {}
            }
        }

        for alias in aliases {
            flag_map.push((
                alias,
                FlagEntry {
                    name: name.clone(),
                    flag_type: flag_type.clone(),
                },
            ));
        }
    }
    Ok(flag_map)
}

fn build_pos_specs(args_spec: &Table) -> Result<Vec<PosSpec>, mlua::Error> {
    let mut pos_specs: Vec<PosSpec> = Vec::new();
    for pair in args_spec.pairs::<Value, Table>() {
        let (_, pos_tbl) = pair?;
        let name: String = pos_tbl
            .get("name")
            .map_err(|_| mlua::Error::external("positional arg missing 'name'"))?;
        let required: bool = pos_tbl.get("required").unwrap_or(true);
        let pos_type: String = pos_tbl.get("type").unwrap_or_else(|_| "str".into());
        pos_specs.push(PosSpec {
            name,
            required,
            pos_type,
        });
    }
    Ok(pos_specs)
}

fn coerce_or_err(lua: &Lua, val: &str, ty: &str) -> Result<Value, ParseError> {
    coerce_value(lua, val, ty).map_err(ParseError::Error)
}

fn set_result_or_extra(
    lua: &Lua,
    result: &Table,
    pos_specs: &[PosSpec],
    pos_idx: &mut usize,
    arg: &str,
) -> Result<(), ParseError> {
    if *pos_idx >= pos_specs.len() {
        // No positional specs defined: silently skip extra args
        return Ok(());
    }
    let spec = &pos_specs[*pos_idx];
    let coerced = coerce_or_err(lua, arg, &spec.pos_type)?;
    result.set(spec.name.as_str(), coerced)?;
    *pos_idx += 1;
    Ok(())
}

fn handle_value_flag(
    lua: &Lua,
    result: &Table,
    entry: &FlagEntry,
    val: &str,
) -> Result<(), ParseError> {
    let coerced = coerce_or_err(lua, val, &entry.flag_type)?;
    result.set(entry.name.as_str(), coerced)?;
    Ok(())
}

fn missing_value_err(_lua: &Lua, alias: &str) -> ParseError {
    ParseError::Error(format!("missing value for {alias}"))
}

fn check_required_pos(_lua: &Lua, pos_specs: &[PosSpec], pos_idx: usize) -> Result<(), ParseError> {
    for spec in pos_specs.iter().skip(pos_idx) {
        if spec.required {
            return Err(ParseError::Error(format!(
                "missing required argument: {}",
                spec.name
            )));
        }
    }
    Ok(())
}

// ---- Main parse function ----

#[allow(clippy::enum_variant_names)]
enum ParseError {
    Passthrough,
    Error(String),
}

impl From<mlua::Error> for ParseError {
    fn from(e: mlua::Error) -> Self {
        Self::Error(e.to_string())
    }
}

#[allow(clippy::too_many_lines)]
fn parse_args(
    lua: &Lua,
    args: &Table,
    spec: &Table,
) -> std::result::Result<(Value, Value), ParseError> {
    let flags_spec: Option<Table> = spec.get("flags").ok();
    let args_spec: Option<Table> = spec.get("args").ok();
    let opts: Option<Table> = spec.get("opts").ok();

    let allow_unknown: bool = opts
        .as_ref()
        .and_then(|o| o.get("allow_unknown").ok())
        .unwrap_or(false);
    let short_count: bool = opts
        .as_ref()
        .and_then(|o| o.get("short_count").ok())
        .unwrap_or(false);

    let flag_map = flags_spec.as_ref().map_or(Ok(Vec::new()), build_flag_map)?;
    let pos_specs = args_spec.as_ref().map_or(Ok(Vec::new()), build_pos_specs)?;

    let result = lua.create_table()?;
    let args_len: usize = args
        .len()?
        .try_into()
        .map_err(|_| ParseError::Error("args length overflow".into()))?;
    let mut i: usize = 1;
    let mut pos_idx: usize = 0;
    let mut end_of_flags = false;

    while i <= args_len {
        let arg: String = args.get(i)?;

        if end_of_flags {
            set_result_or_extra(lua, &result, &pos_specs, &mut pos_idx, &arg)?;
            i += 1;
            continue;
        }

        if arg == "--" {
            end_of_flags = true;
            i += 1;
            continue;
        }

        if arg.starts_with('-') && arg.len() > 1 {
            // Short count?
            if short_count {
                let rest = &arg[1..];
                if let Ok(n) = rest.parse::<i64>() {
                    result.set("n", n)?;
                    i += 1;
                    continue;
                }
            }

            // Combined short flags? (e.g., -vs → -v -s)
            if !arg.starts_with("--") && arg.len() > 2 {
                let rest = &arg[1..];
                let all_boolean = rest.chars().all(|ch| {
                    let alias = format!("-{ch}");
                    lookup_flag(&flag_map, &alias)
                        .map_or(allow_unknown, |e| e.flag_type == "boolean")
                });

                if all_boolean {
                    for ch in rest.chars() {
                        let alias = format!("-{ch}");
                        if let Some(entry) = lookup_flag(&flag_map, &alias) {
                            result.set(entry.name.as_str(), true)?;
                        } else if !allow_unknown {
                            return Err(ParseError::Passthrough);
                        }
                    }
                    i += 1;
                    continue;
                }
            }

            // Long flag: --flag or --flag=value
            if arg.starts_with("--") {
                let eq_pos = arg.find('=');
                let alias = eq_pos.map_or_else(|| arg.clone(), |pos| arg[..pos].to_string());
                let inline_value = eq_pos.map(|pos| arg[pos + 1..].to_string());

                if let Some(entry) = lookup_flag(&flag_map, &alias) {
                    if entry.flag_type == "boolean" {
                        result.set(entry.name.as_str(), true)?;
                    } else if let Some(ref val) = inline_value {
                        handle_value_flag(lua, &result, entry, val)?;
                    } else {
                        i += 1;
                        if i > args_len {
                            return Err(missing_value_err(lua, &alias));
                        }
                        let val: String = args.get(i)?;
                        handle_value_flag(lua, &result, entry, &val)?;
                    }
                    i += 1;
                    continue;
                }

                // Unknown long flag
                if !allow_unknown {
                    return Err(ParseError::Passthrough);
                }
                if inline_value.is_none() && i < args_len {
                    let next: String = args.get(i + 1)?;
                    if !next.starts_with('-') {
                        i += 1;
                    }
                }
                i += 1;
                continue;
            }

            // Short flag: -f or -f value
            let alias = arg.clone();
            if let Some(entry) = lookup_flag(&flag_map, &alias) {
                if entry.flag_type == "boolean" {
                    result.set(entry.name.as_str(), true)?;
                } else {
                    i += 1;
                    if i > args_len {
                        return Err(missing_value_err(lua, &alias));
                    }
                    let val: String = args.get(i)?;
                    handle_value_flag(lua, &result, entry, &val)?;
                }
                i += 1;
                continue;
            }

            // Unknown short flag
            if !allow_unknown {
                return Err(ParseError::Passthrough);
            }
            if i < args_len {
                let next: String = args.get(i + 1)?;
                if !next.starts_with('-') {
                    i += 1;
                }
            }
            i += 1;
            continue;
        }

        // Positional argument
        set_result_or_extra(lua, &result, &pos_specs, &mut pos_idx, &arg)?;
        i += 1;
    }

    // Check required positional args
    check_required_pos(lua, &pos_specs, pos_idx)?;

    Ok((Value::Table(result), Value::Nil))
}

impl SiftLua {
    /// Register `sift.args.parse()`.
    pub(super) fn register_args(&self, sift: &Table) -> Result<()> {
        let args_tbl = self.lua.create_table()?;

        let parse_fn =
            self.lua.create_function(
                |lua: &Lua, (args, spec): (Table, Table)| -> mlua::Result<(Value, Value)> {
                    match parse_args(lua, &args, &spec) {
                        Ok(ok) => Ok(ok),
                        Err(ParseError::Passthrough) => Ok((Value::Nil, Value::Nil)),
                        Err(ParseError::Error(msg)) => Ok((
                            Value::Nil,
                            Value::String(lua.create_string(&msg).map_err(|e| {
                                mlua::Error::external(format!("create string: {e}"))
                            })?),
                        )),
                    }
                },
            )?;

        args_tbl.set("parse", parse_fn)?;
        sift.set("args", args_tbl)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SiftLua;
    use crate::lua::SiftContext;
    use mlua::Table;
    use std::collections::HashMap;

    fn test_sift() -> (SiftLua, Table, mlua::Lua) {
        let lua = SiftLua::new(
            None,
            SiftContext {
                cwd: std::env::current_dir().unwrap(),
                cwd_str: std::env::current_dir().unwrap().display().to_string(),
                cmd_count: std::cell::Cell::new(0),
                env: HashMap::new(),
                session_id: None,
                raw_bytes: 0,
                filtered_bytes: 0,
            },
        )
        .unwrap();
        let lua_ref = lua.lua.clone();
        let sift: Table = lua_ref.globals().get("sift").unwrap();
        (lua, sift, lua_ref)
    }

    fn parse(
        sift: &Table,
        lua: &mlua::Lua,
        args: Vec<&str>,
        spec: Table,
    ) -> (mlua::Value, mlua::Value) {
        let parse_fn: mlua::Function = sift.get::<Table>("args").unwrap().get("parse").unwrap();
        let args_tbl = lua.create_table().unwrap();
        for (i, a) in args.iter().enumerate() {
            args_tbl.set(i + 1, *a).unwrap();
        }
        parse_fn.call((args_tbl, spec)).unwrap()
    }

    fn spec(
        lua: &mlua::Lua,
        flags: Option<Vec<(&str, Vec<&str>, Option<&str>)>>,
        args_spec: Option<Vec<(&str, bool, Option<&str>)>>,
        opts: Option<Vec<(&str, bool)>>,
    ) -> Table {
        let tbl = lua.create_table().unwrap();
        if let Some(f) = flags {
            let flags_tbl = lua.create_table().unwrap();
            for (name, aliases, ty) in f {
                let flag_tbl = lua.create_table().unwrap();
                for (i, alias) in aliases.iter().enumerate() {
                    flag_tbl.set(i + 1, *alias).unwrap();
                }
                if let Some(t) = ty {
                    flag_tbl.set("type", t).unwrap();
                }
                flags_tbl.set(name, flag_tbl).unwrap();
            }
            tbl.set("flags", flags_tbl).unwrap();
        }
        if let Some(a) = args_spec {
            let args_tbl = lua.create_table().unwrap();
            for (i, (name, required, ty)) in a.iter().enumerate() {
                let pos_tbl = lua.create_table().unwrap();
                pos_tbl.set("name", *name).unwrap();
                pos_tbl.set("required", *required).unwrap();
                if let Some(t) = ty {
                    pos_tbl.set("type", *t).unwrap();
                }
                args_tbl.set(i + 1, pos_tbl).unwrap();
            }
            tbl.set("args", args_tbl).unwrap();
        }
        if let Some(o) = opts {
            let opts_tbl = lua.create_table().unwrap();
            for (key, val) in o {
                opts_tbl.set(key, val).unwrap();
            }
            tbl.set("opts", opts_tbl).unwrap();
        }
        tbl
    }

    #[test]
    fn test_parse_boolean_flag() {
        let (_, sift, lua) = test_sift();
        let s = spec(
            &lua,
            Some(vec![("fresh", vec!["--fresh"], None)]),
            Some(vec![("path", true, None)]),
            None,
        );
        let (result, err): (mlua::Value, mlua::Value) =
            parse(&sift, &lua, vec!["--fresh", "file.rs"], s);
        assert!(
            matches!(err, mlua::Value::Nil),
            "expected no error, got {err:?}"
        );
        let tbl = result.as_table().unwrap();
        assert_eq!(tbl.get::<bool>("fresh").unwrap(), true);
        assert_eq!(tbl.get::<String>("path").unwrap(), "file.rs");
    }

    #[test]
    fn test_parse_missing_required() {
        let (_, sift, lua) = test_sift();
        let s = spec(&lua, None, Some(vec![("path", true, None)]), None);
        let (result, err): (mlua::Value, mlua::Value) = parse(&sift, &lua, vec![], s);
        assert!(matches!(result, mlua::Value::Nil), "expected nil result");
        let err_str: String = err.as_str().unwrap().to_string();
        assert!(
            err_str.contains("missing required argument"),
            "got: {err_str}"
        );
    }

    #[test]
    fn test_parse_unknown_flag_passthrough() {
        let (_, sift, lua) = test_sift();
        let s = spec(&lua, None, None, Some(vec![("allow_unknown", false)]));
        let (result, err): (mlua::Value, mlua::Value) = parse(&sift, &lua, vec!["--unknown"], s);
        assert!(matches!(result, mlua::Value::Nil));
        assert!(matches!(err, mlua::Value::Nil));
    }

    #[test]
    fn test_parse_unknown_flag_allow() {
        let (_, sift, lua) = test_sift();
        let s = spec(&lua, None, None, Some(vec![("allow_unknown", true)]));
        let (result, err): (mlua::Value, mlua::Value) =
            parse(&sift, &lua, vec!["--unknown", "val"], s);
        assert!(matches!(err, mlua::Value::Nil));
        let tbl = result.as_table().unwrap();
        assert_eq!(tbl.len().unwrap(), 0);
    }

    #[test]
    fn test_parse_int_flag() {
        let (_, sift, lua) = test_sift();
        let s = spec(&lua, Some(vec![("n", vec!["-n"], Some("int"))]), None, None);
        let (result, err): (mlua::Value, mlua::Value) = parse(&sift, &lua, vec!["-n", "10"], s);
        assert!(matches!(err, mlua::Value::Nil));
        let tbl = result.as_table().unwrap();
        assert_eq!(tbl.get::<i64>("n").unwrap(), 10);
    }

    #[test]
    fn test_parse_invalid_int() {
        let (_, sift, lua) = test_sift();
        let s = spec(&lua, Some(vec![("n", vec!["-n"], Some("int"))]), None, None);
        let (result, err): (mlua::Value, mlua::Value) = parse(&sift, &lua, vec!["-n", "abc"], s);
        assert!(matches!(result, mlua::Value::Nil));
        let err_str: String = err.as_str().unwrap().to_string();
        assert!(err_str.contains("invalid integer"), "got: {err_str}");
    }

    #[test]
    fn test_parse_str_flag() {
        let (_, sift, lua) = test_sift();
        let s = spec(
            &lua,
            Some(vec![("output", vec!["-o", "--output"], Some("str"))]),
            None,
            None,
        );
        let (result, err): (mlua::Value, mlua::Value) = parse(&sift, &lua, vec!["-o", "out.md"], s);
        assert!(matches!(err, mlua::Value::Nil));
        let tbl = result.as_table().unwrap();
        assert_eq!(tbl.get::<String>("output").unwrap(), "out.md");
    }

    #[test]
    fn test_parse_short_count() {
        let (_, sift, lua) = test_sift();
        let s = spec(
            &lua,
            Some(vec![("n", vec!["-n"], Some("int"))]),
            Some(vec![("path", true, None)]),
            Some(vec![("short_count", true)]),
        );
        let (result, err): (mlua::Value, mlua::Value) =
            parse(&sift, &lua, vec!["-10", "file.rs"], s);
        assert!(matches!(err, mlua::Value::Nil));
        let tbl = result.as_table().unwrap();
        assert_eq!(tbl.get::<i64>("n").unwrap(), 10);
        assert_eq!(tbl.get::<String>("path").unwrap(), "file.rs");
    }

    #[test]
    fn test_parse_combined_short_flags() {
        let (_, sift, lua) = test_sift();
        let s = spec(
            &lua,
            Some(vec![("v", vec!["-v"], None), ("s", vec!["-s"], None)]),
            None,
            None,
        );
        let (result, err): (mlua::Value, mlua::Value) = parse(&sift, &lua, vec!["-vs"], s);
        assert!(matches!(err, mlua::Value::Nil));
        let tbl = result.as_table().unwrap();
        assert_eq!(tbl.get::<bool>("v").unwrap(), true);
        assert_eq!(tbl.get::<bool>("s").unwrap(), true);
    }

    #[test]
    fn test_parse_combined_with_unknown() {
        let (_, sift, lua) = test_sift();
        let s = spec(
            &lua,
            Some(vec![("v", vec!["-v"], None)]),
            None,
            Some(vec![("allow_unknown", false)]),
        );
        let (result, err): (mlua::Value, mlua::Value) = parse(&sift, &lua, vec!["-vx"], s);
        assert!(matches!(result, mlua::Value::Nil));
        assert!(matches!(err, mlua::Value::Nil));
    }

    #[test]
    fn test_parse_long_flag_with_equals() {
        let (_, sift, lua) = test_sift();
        let s = spec(
            &lua,
            Some(vec![("output", vec!["--output"], Some("str"))]),
            None,
            None,
        );
        let (result, err): (mlua::Value, mlua::Value) =
            parse(&sift, &lua, vec!["--output=out.md"], s);
        assert!(matches!(err, mlua::Value::Nil));
        let tbl = result.as_table().unwrap();
        assert_eq!(tbl.get::<String>("output").unwrap(), "out.md");
    }

    #[test]
    fn test_parse_end_of_flags() {
        let (_, sift, lua) = test_sift();
        let s = spec(
            &lua,
            Some(vec![("fresh", vec!["--fresh"], None)]),
            Some(vec![("path", true, None)]),
            None,
        );
        let (result, err): (mlua::Value, mlua::Value) =
            parse(&sift, &lua, vec!["--", "--fresh"], s);
        assert!(matches!(err, mlua::Value::Nil));
        let tbl = result.as_table().unwrap();
        assert_eq!(tbl.get::<String>("path").unwrap(), "--fresh");
    }

    #[test]
    fn test_parse_extra_positional() {
        let (_, sift, lua) = test_sift();
        let s = spec(&lua, None, Some(vec![("path", true, None)]), None);
        let (result, err): (mlua::Value, mlua::Value) = parse(&sift, &lua, vec!["a", "b"], s);
        assert!(matches!(err, mlua::Value::Nil));
        let tbl = result.as_table().unwrap();
        assert_eq!(tbl.get::<String>("path").unwrap(), "a");
        // extra positional "b" is silently ignored
    }

    #[test]
    fn test_parse_optional_positional() {
        let (_, sift, lua) = test_sift();
        let s = spec(
            &lua,
            None,
            Some(vec![("path", true, None), ("offset", false, Some("int"))]),
            None,
        );
        let (result, err): (mlua::Value, mlua::Value) = parse(&sift, &lua, vec!["file.rs"], s);
        assert!(matches!(err, mlua::Value::Nil));
        let tbl = result.as_table().unwrap();
        assert_eq!(tbl.get::<String>("path").unwrap(), "file.rs");
        assert!(tbl.get::<i64>("offset").is_err());
    }

    #[test]
    fn test_parse_missing_value_for_flag() {
        let (_, sift, lua) = test_sift();
        let s = spec(&lua, Some(vec![("n", vec!["-n"], Some("int"))]), None, None);
        let (result, err): (mlua::Value, mlua::Value) = parse(&sift, &lua, vec!["-n"], s);
        assert!(matches!(result, mlua::Value::Nil));
        let err_str: String = err.as_str().unwrap().to_string();
        assert!(err_str.contains("missing value"), "got: {err_str}");
    }

    #[test]
    fn test_parse_long_flag_boolean() {
        let (_, sift, lua) = test_sift();
        let s = spec(
            &lua,
            Some(vec![("verbose", vec!["--verbose"], None)]),
            None,
            None,
        );
        let (result, err): (mlua::Value, mlua::Value) = parse(&sift, &lua, vec!["--verbose"], s);
        assert!(matches!(err, mlua::Value::Nil));
        let tbl = result.as_table().unwrap();
        assert_eq!(tbl.get::<bool>("verbose").unwrap(), true);
    }

    #[test]
    fn test_parse_multiple_flags_and_positional() {
        let (_, sift, lua) = test_sift();
        let s = spec(
            &lua,
            Some(vec![
                ("fresh", vec!["--fresh"], None),
                ("n", vec!["-n"], Some("int")),
            ]),
            Some(vec![("path", true, None)]),
            None,
        );
        let (result, err): (mlua::Value, mlua::Value) =
            parse(&sift, &lua, vec!["--fresh", "-n", "5", "file.rs"], s);
        assert!(matches!(err, mlua::Value::Nil));
        let tbl = result.as_table().unwrap();
        assert_eq!(tbl.get::<bool>("fresh").unwrap(), true);
        assert_eq!(tbl.get::<i64>("n").unwrap(), 5);
        assert_eq!(tbl.get::<String>("path").unwrap(), "file.rs");
    }
}
