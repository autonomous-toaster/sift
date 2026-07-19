## Context

`sift.toon.encode` and `sift.toon.decode` are registered in `register_json_toon()` in `api_reg_io.rs`. They take `ctx` as first parameter (unused) and call `encode_default`/`decode_default` — the simplest toon-format path. The toon-format crate (v0.4.6) provides `encode()` with `EncodeOptions` (delimiter, indent) and `decode()` with `DecodeOptions` (strict, no_coerce), plus `decode_strict`, `decode_no_coerce` convenience functions.

## Goals / Non-Goals

**Goals:**
- Expose `sift.toon.encode(data, options?)` with `delimiter` and `indent` options
- Expose `sift.toon.decode(str, options?)` with `strict` and `no_coerce` options
- Drop unused `ctx` parameter from both (pure functions)
- Default behavior identical to current when no options passed

**Non-Goals:**
- Exposing utility functions (`is_keyword`, `escape_string`, etc.)
- Exposing `encode_object`/`encode_array` as separate functions
- JSON stream encoding (`encode_json_stream` — behind feature flag)

## Decisions

1. **Single encode/decode entry points** — `sift.toon.encode(data, options?)` and `sift.toon.decode(str, options?)`. The options table maps to `EncodeOptions`/`DecodeOptions` internally. No separate functions for each variant.

2. **Options table mapping:**
   - `delimiter: "comma" | "pipe"` → `Delimiter::Comma | Delimiter::Pipe`
   - `indent: "tab" | "space2" | "space4"` → `Indent::Tab | Indent::Space(2) | Indent::Space(4)`
   - `strict: true` → calls `decode_strict`
   - `no_coerce: true` → calls `decode_no_coerce`
   - Both `strict` and `no_coerce` → error (mutually exclusive)

3. **Drop ctx** — `sift.toon.*` are pure functions like `sift.str.*`. No plugin passes `ctx` meaningfully.

## Risks / Trade-offs

1. **Backwards compatibility** — `sift.toon.encode(ctx, val)` → `sift.toon.encode(val)`. If any plugin passes `ctx` as first arg, it will be interpreted as `data` (a table). → Mitigation: no shipped plugins use `sift.toon.*` with `ctx` meaningfully. The `ctx` was always ignored.

2. **Mutual exclusion** — `strict` and `no_coerce` can't both be true. → Return error string if both set.