## ADDED Requirements

### Requirement: Auto-detect and compress JSON curl responses

The `curl` plugin SHALL intercept `curl` commands. If `-v` or `--verbose` is NOT in the arguments, the plugin SHALL add `-w "\n%{content_type}"` to detect the response content type. If the content type contains `json`, the plugin SHALL compress the response body using `sift.json.shortest()` with TOON format, store the raw JSON via `sift.store()`, and emit a nudge with the raw path. If `-v` or `--verbose` IS in the arguments, the plugin SHALL run the command as-is and return the full output. The plugin SHALL always propagate curl's exit code.

#### Scenario: JSON response without -v
- **WHEN** agent runs `curl https://httpbin.org/anything`
- **THEN** plugin adds `-w "\n%{content_type}"` internally
- **AND** detects `application/json` content type
- **AND** returns compressed JSON via `sift.json.shortest()`
- **AND** stores raw JSON and emits nudge

#### Scenario: Non-JSON response without -v
- **WHEN** agent runs `curl https://httpbin.org/html`
- **THEN** plugin adds `-w "\n%{content_type}"` internally
- **AND** detects `text/html` content type
- **AND** returns body as-is (no compression)

#### Scenario: curl with -v flag
- **WHEN** agent runs `curl -v https://httpbin.org/anything`
- **THEN** plugin runs command as-is
- **AND** returns full verbose output (no compression)

#### Scenario: curl error exit code propagated
- **WHEN** agent runs `curl https://nonexistent.example.com`
- **THEN** plugin returns curl's non-zero exit code
- **AND** returns curl's error output

#### Scenario: curl with -o output file
- **WHEN** agent runs `curl -o /tmp/out https://httpbin.org/anything`
- **THEN** plugin adds `-w "\n%{content_type}"` to the command
- **AND** stdout is empty (body written to file), content type is the only output
- **AND** plugin returns output as-is (no compression on empty body)
