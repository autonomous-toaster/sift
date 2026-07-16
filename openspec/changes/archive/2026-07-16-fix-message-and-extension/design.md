# Design

## 1. sift-read message format

**File**: `plugins/sift-read.lua`

Change the "unchanged" return block:

```lua
if offset or limit then
    if range_start == range_end then
        msg = string.format("[sift] %s line %d unchanged", raw_path, range_start)
    else
        msg = string.format("[sift] %s lines %d-%d unchanged", raw_path, range_start, range_end)
    end
    return { status = "unchanged", message = msg }
end
```

## 2. Pi extension fixes

**File**: `integrations/pi/sift.ts`

### 2a. Shell quoting

Replace `JSON.stringify(path)` with single-quote wrapping:

```typescript
function shQuote(s: string): string {
  return "'" + s.replace(/'/g, "'\\''") + "'";
}
```

### 2b. Session ID propagation

- Module-level `currentSessionId` variable
- Get from `session_start` event handler via `ctx.sessionManager.getSessionId()`
- Pass to `spawnHook` via closure
- Use in read tool execute function

### 2c. Read tool via `createReadTool`

Replace `execSync`-based read tool with `createReadTool` + custom `ReadOperations`:

```typescript
const readTool = createReadTool(cwd, {
  operations: {
    readFile: async (absolutePath) => {
      const result = execSync(`sift -c ${shQuote(`sift-read ${shQuote(absolutePath)}`)}`, {
        env: { ...process.env, AI_SESSION: currentSessionId },
        encoding: "utf-8",
        maxBuffer: 10 * 1024 * 1024,
      });
      return Buffer.from(result.toString());
    },
    access: async (absolutePath) => {
      await fsAccess(absolutePath);
    },
  },
});
```

This delegates file reading to sift while keeping pi's truncation, line range, and image detection logic intact.
