## Task Reference

| Task ID | Description |
|---------|-------------|
| T1.1 | Implement `sift.args.parse()` Rust function |
| T1.2 | Wire `register_args()` into module system |
| T2.1 | Convert cat.lua to use `sift.args.parse()` |
| T2.2 | Convert head.lua to use `sift.args.parse()` |
| T2.3 | Convert tail.lua to use `sift.args.parse()` |
| T2.4 | Convert sed.lua to use `sift.args.parse()` |
| T2.5 | Convert sift-read.lua to use `sift.args.parse()` |
| T2.6 | Convert git-commit.lua to use `sift.args.parse()` |
| T2.7 | Convert curl.lua to use `sift.args.parse()` |
| T2.8 | Convert openspec.lua to use `sift.args.parse()` |

## ADDED Requirements

### Requirement: Declarative argument parsing

`sift.args.parse(args, spec)` SHALL accept a Lua table of string arguments and a declarative spec table, and return a parsed result table on success or `nil, error_string` on failure.

- T1.1 SHALL complete BEFORE T2.1 SHALL run (parser must exist before plugins use it)
- ALWAYS T1.1 SHALL validate the spec structure before parsing

#### Scenario: Successful parse with boolean flag and positional
- **WHEN** `sift.args.parse({"--fresh", "file.rs"}, {flags={fresh={"--fresh"}}, args={{name="path", required=true}}})` is called
- **THEN** it SHALL return `{fresh=true, path="file.rs"}`

#### Scenario: Missing required positional
- **WHEN** `sift.args.parse({}, {args={{name="path", required=true}}})` is called
- **THEN** it SHALL return `nil, "missing required argument: path"`

#### Scenario: Unknown flag with allow_unknown=false
- **WHEN** `sift.args.parse({"--unknown"}, {opts={allow_unknown=false}})` is called
- **THEN** it SHALL return `nil, nil` (plugin should passthrough)

#### Scenario: Unknown flag with allow_unknown=true
- **WHEN** `sift.args.parse({"--unknown", "val"}, {opts={allow_unknown=true}})` is called
- **THEN** it SHALL return `{}` (empty result, unknown flag skipped)

### Requirement: Flag type coercion

Flags SHALL support three types: `boolean` (default), `int`, and `str`. Values SHALL be coerced to the declared type.

- ALWAYS T1.1 SHALL coerce flag values to their declared type

#### Scenario: Integer flag
- **WHEN** `sift.args.parse({"-n", "10"}, {flags={n={"-n", type="int"}}})` is called
- **THEN** it SHALL return `{n=10}` (Lua number)

#### Scenario: Invalid integer
- **WHEN** `sift.args.parse({"-n", "abc"}, {flags={n={"-n", type="int"}}})` is called
- **THEN** it SHALL return `nil, "invalid integer: abc"`

#### Scenario: String flag
- **WHEN** `sift.args.parse({"-o", "out.md"}, {flags={output={"-o", type="str"}}})` is called
- **THEN** it SHALL return `{output="out.md"}`

### Requirement: Short count syntax

When `opts.short_count = true`, an argument matching `^-(\d+)$` SHALL be parsed as `n=<number>`.

- ALWAYS T1.1 SHALL check short count before combined flag splitting

#### Scenario: Short count
- **WHEN** `sift.args.parse({"-10", "file.rs"}, {flags={n={"-n", type="int"}}, args={{name="path"}}, opts={short_count=true}})` is called
- **THEN** it SHALL return `{n=10, path="file.rs"}`

### Requirement: Combined short flags

When a short flag argument contains multiple characters (e.g., `-vs`), it SHALL be split into individual boolean flags. Only boolean flags SHALL be combined.

- ALWAYS T1.1 SHALL only split combined flags when all characters are boolean flags

#### Scenario: Combined boolean flags
- **WHEN** `sift.args.parse({"-vs"}, {flags={v={"-v"}, s={"-s"}}})` is called
- **THEN** it SHALL return `{v=true, s=true}`

#### Scenario: Combined with unknown flag
- **WHEN** `sift.args.parse({"-vx"}, {flags={v={"-v"}}, opts={allow_unknown=false}})` is called
- **THEN** it SHALL return `nil, nil` (unknown flag `-x` in combined form)

### Requirement: Long flag with inline value

Long flags SHALL support `--flag=value` syntax for non-boolean types.

- ALWAYS T1.1 SHALL parse `--flag=value` as flag with value

#### Scenario: Long flag with equals
- **WHEN** `sift.args.parse({"--output=out.md"}, {flags={output={"--output", type="str"}}})` is called
- **THEN** it SHALL return `{output="out.md"}`

### Requirement: End-of-flags marker

`--` SHALL stop flag parsing. All subsequent arguments SHALL be treated as positional.

- ALWAYS T1.1 SHALL treat `--` as end-of-flags marker

#### Scenario: End-of-flags
- **WHEN** `sift.args.parse({"--", "--fresh", "file.rs"}, {flags={fresh={"--fresh"}}, args={{name="path"}}})` is called
- **THEN** it SHALL return `{path="--fresh"}` (--fresh treated as positional)

### Requirement: Plugin conversion

Each shipped plugin SHALL use `sift.args.parse()` instead of its manual parser. The plugin SHALL check the return value: on `nil, nil` it SHALL passthrough; on `nil, error` it SHALL return the error.

- T1.1 SHALL complete BEFORE T2.1 SHALL run
- T2.1 SHALL complete BEFORE T2.2 SHALL run (order within phase, but any order works)

#### Scenario: cat.lua with sift.args.parse
- **WHEN** cat.lua receives `{"file.rs"}` with spec `{args={{name="path", required=true}}, opts={allow_unknown=false}}`
- **THEN** it SHALL parse successfully and use `parsed.path`

#### Scenario: head.lua with short count
- **WHEN** head.lua receives `{"-5", "file.rs"}` with spec `{flags={n={"-n", type="int"}}, args={{name="path"}}, opts={short_count=true, allow_unknown=false}}`
- **THEN** it SHALL parse `n=5, path="file.rs"`

#### Scenario: git-commit.lua forbids -n
- **WHEN** git-commit.lua receives `{"-m", "msg", "-n"}` with spec `{flags={n={"-n"}, ["no-verify"]={"--no-verify"}}, opts={allow_unknown=true}}`
- **THEN** it SHALL detect `parsed.n = true` and return exit code 1

#### Scenario: curl.lua detects verbose
- **WHEN** curl.lua receives `{"-vs", "https://example.com"}` with spec `{flags={v={"-v"}, s={"-s"}}, opts={allow_unknown=true}}`
- **THEN** it SHALL parse `v=true, s=true` and run as-is
