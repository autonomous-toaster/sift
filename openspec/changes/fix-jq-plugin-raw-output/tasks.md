## 1. Fix jq plugin raw output

- [x] 1.1 In `plugins/jq.lua`, change the `-r` output loop to type-check before iterating: if `type(decoded) == "table"`, use `ipairs`; otherwise, convert the scalar directly to string and add it as a single line.
