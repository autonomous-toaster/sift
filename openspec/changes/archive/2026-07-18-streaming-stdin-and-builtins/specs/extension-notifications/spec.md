## Task Reference

| Task ID | Description |
|---------|-------------|
| T3.1 | Add cache hit detection to reset.lua |
| T3.2 | Add ctx.ui.notify() calls to resetCache in sift.ts |
| T3.3 | Remove session_shutdown, session_tree, session_fork, session_switch reset handlers |
| T3.4 | Keep only session_compact reset handler |

## ADDED Requirements

### Requirement: Cache hit detection in reset plugin

The reset plugin SHALL detect whether cache entries existed before clearing and return different output: `"[sift] ok (cleared)\n"` if entries were found, `"[sift] ok (nothing to clear)\n"` if none.

T3.1 SHALL complete BEFORE T3.2 SHALL run.

#### Scenario: Cache had entries

- **WHEN** T3.1 runs `sift -c reset` and cache had entries
- **THEN** output SHALL contain `"(cleared)"`

#### Scenario: Cache was empty

- **WHEN** T3.1 runs `sift -c reset` and cache was empty
- **THEN** output SHALL contain `"(nothing to clear)"`

### Requirement: Extension notifies user on cache reset

The pi extension SHALL call `ctx.ui.notify()` on cache reset: `"sift: cache reset"` (info) on success, `"sift: cache reset failed: <error>"` (error) on failure.

T3.2 SHALL complete AFTER T3.1 SHALL complete.

#### Scenario: Successful reset with cache

- **WHEN** T3.2 runs resetCache and output contains `"(cleared)"`
- **THEN** `ctx.ui.notify("sift: cache reset", "info")` SHALL be called

#### Scenario: Successful reset without cache

- **WHEN** T3.2 runs resetCache and output contains `"(nothing to clear)"`
- **THEN** `ctx.ui.notify("sift: cache reset (nothing to clear)", "info")` SHALL be called

#### Scenario: Reset failure

- **WHEN** T3.2 runs resetCache and execSync throws
- **THEN** `ctx.ui.notify("sift: cache reset failed: <error>", "error")` SHALL be called

### Requirement: Only session_compact resets cache

The extension SHALL only reset the sift cache on `session_compact`. Handlers for `session_shutdown`, `session_tree`, `session_fork`, and `session_switch` SHALL be removed.

T3.3 SHALL complete BEFORE T3.4 SHALL run.

#### Scenario: Compact triggers reset

- **WHEN** T3.4 handles `session_compact`
- **THEN** resetCache SHALL be called

#### Scenario: Shutdown does not trigger reset

- **WHEN** T3.3 removes the `session_shutdown` handler
- **THEN** `session_shutdown` SHALL NOT call resetCache

#### Scenario: Tree navigation does not trigger reset

- **WHEN** T3.3 removes the `session_tree` handler
- **THEN** `session_tree` SHALL NOT call resetCache
