# Dependency Security Advisory Evidence - 2026-05-18

> Evidence-only artifact. This file records the Security Audit blocker separately from the already closed fmt/test/clippy/docs gate work. It does not change dependencies or CI policy.

## Source

- GitHub Actions run: https://github.com/chengjon/quantix-rust/actions/runs/26007862679
- Security job: https://github.com/chengjon/quantix-rust/actions/runs/26007862679/job/76442610183
- Command observed in CI: `cargo audit`
- Quality-gate boundary commit: `3676cb4`
- Security workstream commit when generated: `f06f977`

## Summary

- Total advisories: 18
- Vulnerabilities: 8
- Warnings: 10
- Default reachable: 15
- All-features only: 3
- Lockfile-only or unresolved from metadata: 0

## Advisory Table

| Advisory | Crate | Version | Class | Reachability | Solution | Primary path |
|---|---|---:|---|---|---|---|
| RUSTSEC-2025-0003 | `fast-float` | `0.2.0` | vulnerability | default | No fixed upgrade is available! | fast-float@0.2.0 <- polars-arrow@0.43.1 <- polars@0.43.1 <- quantix-cli@0.1.0 |
| RUSTSEC-2024-0421 | `idna` | `0.3.0` | vulnerability | default | Upgrade to >=1.0.0 | idna@0.3.0 <- cookie_store@0.20.0 <- reqwest@0.11.27 <- quantix-cli@0.1.0 |
| RUSTSEC-2026-0041 | `lz4_flex` | `0.11.5` | vulnerability | default | Upgrade to >=0.11.6, <0.12.0 OR >=0.12.1 | lz4_flex@0.11.5 <- clickhouse@0.12.2 <- quantix-cli@0.1.0 |
| RUSTSEC-2021-0041 | `parse_duration` | `2.1.1` | vulnerability | all-features-only | No fixed upgrade is available! | parse_duration@2.1.1 <- taos-ws@0.5.12 <- quantix-cli@0.1.0 |
| RUSTSEC-2024-0437 | `protobuf` | `2.28.0` | vulnerability | default | Upgrade to >=3.7.2 | protobuf@2.28.0 <- prometheus@0.13.4 <- quantix-cli@0.1.0 |
| RUSTSEC-2023-0071 | `rsa` | `0.9.10` | vulnerability | default | No fixed upgrade is available! | rsa@0.9.10 <- sqlx-mysql@0.7.4 <- sqlx@0.7.4 <- quantix-cli@0.1.0 |
| RUSTSEC-2024-0363 | `sqlx` | `0.7.4` | vulnerability | default | Upgrade to >=0.8.1 | sqlx@0.7.4 <- quantix-cli@0.1.0 |
| RUSTSEC-2023-0065 | `tungstenite` | `0.18.0` | vulnerability | all-features-only | Upgrade to >=0.20.1 | tungstenite@0.18.0 <- tokio-tungstenite@0.18.0 <- taos-ws@0.5.12 <- quantix-cli@0.1.0 |
| RUSTSEC-2021-0141 | `dotenv` | `0.15.0` | unmaintained | default | No solution in audit output | dotenv@0.15.0 <- quantix-cli@0.1.0 |
| RUSTSEC-2025-0119 | `number_prefix` | `0.4.0` | unmaintained | default | No solution in audit output | number_prefix@0.4.0 <- indicatif@0.17.11 <- quantix-cli@0.1.0 |
| RUSTSEC-2024-0436 | `paste` | `1.0.15` | unmaintained | default | No solution in audit output | paste@1.0.15 <- parquet@53.4.1 <- quantix-cli@0.1.0 |
| RUSTSEC-2025-0134 | `rustls-pemfile` | `1.0.4` | unmaintained | default | No solution in audit output | rustls-pemfile@1.0.4 <- reqwest@0.11.27 <- quantix-cli@0.1.0 |
| RUSTSEC-2024-0320 | `yaml-rust` | `0.4.5` | unmaintained | default | No solution in audit output | yaml-rust@0.4.5 <- config@0.13.4 <- quantix-cli@0.1.0 |
| RUSTSEC-2024-0379 | `fast-float` | `0.2.0` | unsound | default | No solution in audit output | fast-float@0.2.0 <- polars-arrow@0.43.1 <- polars@0.43.1 <- quantix-cli@0.1.0 |
| RUSTSEC-2026-0002 | `lru` | `0.12.5` | unsound | all-features-only | No solution in audit output | lru@0.12.5 <- ratatui@0.25.0 <- quantix-cli@0.1.0 |
| RUSTSEC-2026-0097 | `rand` | `0.8.5` | unsound | default | No solution in audit output | rand@0.8.5 <- quantix-cli@0.1.0 |
| RUSTSEC-2026-0097 | `rand` | `0.9.2` | unsound | default | No solution in audit output | rand@0.9.2 <- postgres-protocol@0.6.10 <- postgres-types@0.2.12 <- rust_decimal@1.40.0 <- quantix-cli@0.1.0 |
| NO-ID | `lz4_flex` | `0.11.5` | yanked | default | No solution in audit output | lz4_flex@0.11.5 <- clickhouse@0.12.2 <- quantix-cli@0.1.0 |

## Notes

- `all-features-only` means the advisory appears when optional features are resolved, not necessarily in the default dependency graph.
- `lockfile-only-or-unresolved` should be investigated before remediation; it may indicate a stale lockfile package or a feature/resolution path not captured by metadata.
- Advisories with no fixed upgrade must not be hidden by CI policy without an explicit risk-acceptance record.
