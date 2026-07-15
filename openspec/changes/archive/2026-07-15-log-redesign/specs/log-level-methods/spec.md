## ADDED Requirements

### Requirement: sift.log provides level methods

ALWAYS T1.1 SHALL register `sift.log` as a table with level methods `info`, `warn`, `error`, and `debug`, each accepting `(ctx, msg)`.

#### Scenario: Log at info level
- **WHEN** a plugin calls `sift.log.info(ctx, "loaded plugin")`
- **THEN** T1.1 SHALL print `[sift] INFO: loaded plugin` to stdout.

#### Scenario: Log at warn level
- **WHEN** a plugin calls `sift.log.warn(ctx, "deprecated API")`
- **THEN** T1.1 SHALL print `[sift] WARN: deprecated API` to stderr.

#### Scenario: Log at error level
- **WHEN** a plugin calls `sift.log.error(ctx, "command failed")`
- **THEN** T1.1 SHALL print `[sift] ERROR: command failed` to stderr.

#### Scenario: Log at debug level
- **WHEN** a plugin calls `sift.log.debug(ctx, "cache miss")`
- **THEN** T1.1 SHALL print `[sift] DEBUG: cache miss` to stdout.

### Requirement: sift.log is NOT callable

ALWAYS T1.1 SHALL NOT register a `__call` metatable on `sift.log`. Calling `sift.log(ctx, level, msg)` SHALL raise a Lua error.

#### Scenario: Callable usage errors
- **WHEN** a plugin calls `sift.log(ctx, "info", "msg")`
- **THEN** T1.1 SHALL raise a Lua error (table is not callable).
