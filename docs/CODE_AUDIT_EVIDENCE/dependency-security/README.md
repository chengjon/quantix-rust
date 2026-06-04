# Dependency Security Evidence

This directory stores evidence for the dependency security audit workstream started on 2026-05-18. It is separate from the fmt/test/clippy/docs gate recovery.

Files:

- `advisories-2026-05-18.json`: structured advisory evidence parsed from GitHub Actions run 26007862679 and enriched with `cargo metadata --locked` dependency paths.
- `advisories-2026-05-18.md`: human-readable summary of the same advisory set.

Boundary:

- No dependency upgrades are represented here.
- No `cargo audit --ignore` or CI policy relaxation is represented here.
- The next step is remediation issue triage by dependency owner.
