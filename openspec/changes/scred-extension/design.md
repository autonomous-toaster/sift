## Context

sift has a Lua plugin system with `sift.ext.*` for Rust extensions (mime, xberg, html, markdown). The `sift.exec()` function supports an optional `transform` callback that processes stdout chunks in real-time from a background thread.

scred provides `RedactionStream` — a streaming redactor with 512B lookahead window that detects and redacts secrets using pure Rust SIMD pattern matching (244+ patterns). Currently `RedactionStream::feed()` calls `detect_all()` directly with no pattern filtering.

The current `plugins/scred.lua` only intercepts `echo`/`env`/`printenv` — a narrow approach that misses most leak vectors.

## Goals / Non-Goals

**Goals:**
- Embed `scred-redactor` as `sift.ext.scred` Rust extension in sift's Lua VM
- Expose streaming transform API: `sift.ext.scred.create_transform(opts?) → (transform_fn, finalize_fn)`
- Expose one-shot API: `sift.ext.scred.redact(text, opts?) → string`
- Support pattern selection via scred's native `PatternSelector` (names, globs, regex, ALL)
- Create user bash plugin that redacts ALL command output via the transform
- Handle errors gracefully (poisoned mutex → passthrough)

**Non-Goals:**
- No changes to scred's detection patterns themselves
- No pre-execution environment scrubbing (separate concern)
- No changes to existing plugin dispatch logic

## Decisions

### 1. Streaming transform as primary API

The bash plugin uses `sift.ext.scred.create_transform()` which returns two Lua functions sharing an `Arc<Mutex<RedactionStream>>`:

```lua
local transform, finalize = sift.ext.scred.create_transform({redact = "ALL"})
-- transform(chunk) → redacted_chunk  (called from bg thread via sift.exec)
-- finalize() → final_chunk, stats     (called by plugin after exec)
```

The transform function is passed to `sift.exec()` as `{transform = transform, silent = true}`. The background thread calls it per-chunk. After execution, the plugin calls `finalize()` to flush the lookahead buffer.

### 2. Pattern selection via scred's native PatternSelector

The `redact` option accepts a string in scred's native `PatternSelector` format:

```lua
-- Default: redact ALL patterns
sift.ext.scred.create_transform()

-- By exact pattern name (comma-separated)
sift.ext.scred.create_transform({redact = "aws-access-key,github-token"})

-- By glob pattern
sift.ext.scred.create_transform({redact = "aws-*,github-*"})

-- By regex
sift.ext.scred.create_transform({redact = "regex:^sk-"})
```

The string is parsed into `PatternSelector` using scred's own parsing logic. The `RedactionStream` stores the selector and uses it in `feed()` to filter matches before redacting.

### 3. RedactionStream modification in scred

`RedactionStream` needs a `PatternSelector` field. In `feed()`, after `detect_all()`, filter matches through the selector before `redact_in_place()`:

```rust
// Current:
let detection = detect_all(&combined);
redact_in_place(&mut redacted, &detection.matches);

// New:
let detection = detect_all(&combined);
let filtered: Vec<_> = detection.matches.into_iter()
    .filter(|m| selector.allows(m.pattern_type))
    .collect();
redact_in_place(&mut redacted, &filtered);
```

The `allows()` method maps the numeric pattern type ID to a pattern name using a built-in lookup table, then checks if the name matches the `PatternSelector`.

### 4. Error handling

The transform runs in a background thread. If `Mutex::lock()` fails (poisoned), return the chunk unmodified:

```rust
let feed_fn = lua.create_function(move |_, chunk: String| {
    let result = match feed_stream.lock() {
        Ok(mut s) => s.feed(chunk.as_bytes()),
        Err(_) => chunk.into_bytes(),  // passthrough on error
    };
    Ok(String::from_utf8_lossy(&result).to_string())
})?;
```

### 5. Stats exposure

`finalize()` returns `StreamingStats` as a Lua table:
```lua
local final, stats = finalize()
-- stats = {bytes_read = 1000, bytes_written = 980, patterns_found = 3}
```

### 6. Feature gate

`scred` feature in `sift-core/Cargo.toml`. When disabled, `sift.ext.scred` is `nil` and the bash plugin falls through to the built-in `__default__`.

## Risks / Trade-offs

- **Per-chunk overhead**: Each chunk goes through SIMD pattern matching. For typical output (<1MB) this is negligible. For very large output (100MB+), the 512B lookahead adds latency.
- **Mutex contention**: The `Arc<Mutex<RedactionStream>>` is locked per-chunk. In practice, the background thread is the only consumer, so contention is zero.
- **scred dependency**: Adds ~200KB to binary size (pattern tables). Optional feature, so users without secrets don't pay the cost.
- **scred source modification**: Requires a small change to `RedactionStream` in the scred repo. The change is backward-compatible (default selector = All).
