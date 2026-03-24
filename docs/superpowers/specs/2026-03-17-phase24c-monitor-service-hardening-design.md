# Phase 24C Monitor Service Hardening Design

**Date:** 2026-03-17
**Status:** Approved in-session
**Depends On:** Phase 24B green baseline (`master` @ `e9fb061`)

> This document is the source of truth for the next monitor slice: harden the WSL2 `systemd --user` service integration so service installation no longer depends on ephemeral build paths and so service lifecycle commands behave more safely.

---

## Goal

Make the existing Phase 24B monitor service integration operationally stable for WSL2 `systemd --user` usage by:

1. Decoupling the installed service from `cargo run` and temporary `target` paths
2. Storing the service binary path in dedicated service config
3. Making install fail early when the configured binary is invalid
4. Making uninstall refuse to remove a still-running service
5. Improving `monitor service status` to show a concise structured summary

This phase is a hardening pass. It should not change monitor loop behavior, event semantics, or quote/stop evaluation logic.

## Bottom-Up Scope

### User jobs

The minimum user jobs are:

1. Configure which stable `quantix` binary the service should run
2. Install a user service that survives Rust rebuilds and temporary target cleanup
3. See whether the service is installed, enabled, and actively running
4. Avoid uninstalling the service while its process is still active
5. Keep all runtime monitor behavior delegated to the existing Phase 24B loop

### Exact CLI boundary

Only implement:

```bash
quantix monitor service-config show
quantix monitor service-config set --quantix-bin /absolute/path/to/quantix

quantix monitor service install
quantix monitor service uninstall
quantix monitor service start
quantix monitor service stop
quantix monitor service status
quantix monitor service enable
quantix monitor service disable
```

Rules:

- `service-config set` only accepts an absolute path
- `service install` must fail if the configured binary path does not exist or is not executable
- `service install` writes:
  - `~/.quantix/monitor/service.json`
  - `~/.local/bin/quantix-monitor-run`
  - `~/.config/systemd/user/quantix-monitor.service`
- the unit must point to the wrapper script, not directly to the current executable
- `service uninstall` must fail if the service is still active
- `service status` must show a structured summary first, then optionally include raw `systemctl` output

### Explicitly deferred

Phase 24C does not include:

- cross-platform service managers for macOS or Windows
- auto-detection or search heuristics for the `quantix` binary path
- package-manager installation workflows
- automatic binary upgrades
- multiple service profiles
- desktop notifications
- monitor loop behavior changes

## Approaches Considered

### Option A: Keep the unit pointing directly to `current_exe()`

Pros:

- no new config file
- smallest code diff

Cons:

- breaks whenever the installed path is a temporary build artifact
- ties service installation to the command used for installation
- not suitable for stable WSL2 service usage

### Option B: Point the unit directly at a configured binary path

Pros:

- stable executable path
- simpler than adding a wrapper script

Cons:

- pushes env and future startup concerns into the unit file
- makes later hardening or startup prechecks harder

### Option C: Use dedicated service config plus a stable wrapper script

Pros:

- clear separation of concerns
- service config owns the binary path
- wrapper script is a stable `ExecStart` target
- easy to extend later without bloating the unit

Cons:

- one extra file
- slightly more installation logic

## Recommendation

Choose **Option C**.

Add a dedicated `service.json` for service-specific configuration, write a stable wrapper script under `~/.local/bin`, and make the systemd unit point at that script. Keep monitor runtime configuration in the existing `config.json`; do not duplicate runtime fields into the service config.

## Architecture

### File boundaries

- `src/monitor/service_config.rs`
  - read/write `service.json`
  - validate `quantix_bin_path`

- `src/monitor/systemd.rs`
  - render wrapper script
  - render unit file
  - install/uninstall/start/stop/status/enable/disable wrappers

- `src/cli/mod.rs`
  - add `monitor service-config` CLI surface

- `src/cli/handlers.rs`
  - wire `service-config`
  - adapt `service install/status/uninstall`

- tests
  - parser tests for `service-config`
  - config tests for `service.json`
  - systemd tests for wrapper script + unit rendering + uninstall safety

### Dedicated service config

Path:

- default: `~/.quantix/monitor/service.json`

Shape:

```json
{
  "quantix_bin_path": "/absolute/path/to/quantix"
}
```

Rules:

- path must be absolute
- path must exist
- path must be executable
- service config should not carry monitor loop settings like interval/group

### Wrapper script

Path:

- `~/.local/bin/quantix-monitor-run`

Responsibilities:

- be the stable `ExecStart` target
- execute the configured `quantix` binary with:
  - `monitor daemon run`
- preserve the environment variables already required by Phase 24B

The script should stay thin. It should not try to discover the binary path dynamically.

### Unit rendering

Path:

- `~/.config/systemd/user/quantix-monitor.service`

Rules:

- `ExecStart` points to the wrapper script
- keep `Restart=on-failure`
- keep the current monitor-related `Environment=` lines
- do not inline the configured `quantix` binary path into the unit directly

## Install And Uninstall Semantics

### `service install`

Order:

1. load `service.json`
2. validate `quantix_bin_path`
3. write wrapper script
4. write unit
5. run `systemctl --user daemon-reload`

Failure policy:

- any step failure aborts install
- no silent partial success
- if later steps fail, clean up files written earlier in the same install attempt where reasonable

### `service uninstall`

Rules:

- if service is active, return a clear error telling the user to run `monitor service stop` first
- only after confirming it is not active:
  - remove unit
  - remove wrapper script
  - run `daemon-reload`

## Status Output

`monitor service status` should report:

- `installed`
- `enabled`
- `active`
- `unit_path`
- `wrapper_path`
- `quantix_bin_path`

After the structured summary, it may append raw `systemctl --user status` text for detail.

## Error Handling

Hard errors:

- missing `service.json`
- invalid binary path
- relative binary path
- non-executable binary
- `systemctl --user daemon-reload` failure
- uninstall attempted while service is active

Soft errors:

- none in the installation path; this phase should fail loudly rather than guess

## Testing Strategy

1. Parser tests
   - `monitor service-config show`
   - `monitor service-config set --quantix-bin`

2. Service-config tests
   - round-trip persistence
   - reject relative paths
   - reject missing binary
   - reject non-executable binary

3. Systemd tests
   - wrapper script content is correct
   - unit points to wrapper script, not `current_exe`
   - `daemon-reload` args are correct
   - `status` summary formatting

4. Uninstall safety tests
   - active service blocks uninstall
   - inactive service allows uninstall

5. Docs/hygiene tests
   - README and user manual mention `service-config`
   - docs no longer imply `install` depends on transient `cargo run` paths

## Non-Goals

Phase 24C is not:

- a packaging/distribution system
- a binary installer
- a service supervisor beyond `systemd --user`
- a redesign of the monitor loop
