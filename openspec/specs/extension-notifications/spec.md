## Purpose

Define the behavior of the sift cache reset in the pi extension, including user notifications via `ctx.ui.notify()` and restricting cache reset to only the `session_compact` lifecycle event.

## Requirements

### Requirement: Cache hit detection in reset plugin

The reset plugin SHALL detect whether cache entries existed before clearing and return different output: `"[sift] ok (cleared)\n"` if entries were found, `"[sift] ok (nothing to clear)\n"` if none.

#### Scenario: Cache had entries

- **WHEN** `sift -c reset` runs and cache had entries
- **THEN** output SHALL contain `"(cleared)"`

#### Scenario: Cache was empty

- **WHEN** `sift -c reset` runs and cache was empty
- **THEN** output SHALL contain `"(nothing to clear)"`

### Requirement: Extension notifies user on cache reset

The pi extension SHALL call `ctx.ui.notify()` on cache reset: `"sift: cache reset"` (info) on success, `"sift: cache reset failed: <error>"` (error) on failure.

#### Scenario: Successful reset with cache

- **WHEN** resetCache runs and output contains `"(cleared)"`
- **THEN** `ctx.ui.notify("sift: cache reset", "info")` SHALL be called

#### Scenario: Successful reset without cache

- **WHEN** resetCache runs and output contains `"(nothing to clear)"`
- **THEN** `ctx.ui.notify("sift: cache reset (nothing to clear)", "info")` SHALL be called

#### Scenario: Reset failure

- **WHEN** resetCache runs and execSync throws
- **THEN** `ctx.ui.notify("sift: cache reset failed: <error>", "error")` SHALL be called

### Requirement: Only session_compact resets cache

The extension SHALL only reset the sift cache on `session_compact`. Handlers for `session_shutdown`, `session_tree`, `session_fork`, and `session_switch` SHALL be removed.

#### Scenario: Compact triggers reset

- **WHEN** `session_compact` is handled
- **THEN** resetCache SHALL be called

#### Scenario: Shutdown does not trigger reset

- **WHEN** `session_shutdown` handler is removed
- **THEN** `session_shutdown` SHALL NOT call resetCache

#### Scenario: Tree navigation does not trigger reset

- **WHEN** `session_tree` handler is removed
- **THEN** `session_tree` SHALL NOT call resetCache