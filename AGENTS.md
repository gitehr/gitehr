# Agent Instructions

- read all of the files in `spec/` for agent context on the project.
- Keep GitEHR's plugin boundary strict: `gitehr` owns only generic `$PATH` dispatch and its public CLI/repository contracts. Feature-specific parsing, domain rules, schemas, and integrations belong in independently installable `gitehr-<name>` plugins, never in the core CLI.
- Plugins must compose GitEHR only through documented public interfaces such as `gitehr journal add`; do not depend on `cli/src` modules or write repository files directly. Keep a reusable feature engine (for example `clincalc`) independent of both the plugin and GitEHR.
