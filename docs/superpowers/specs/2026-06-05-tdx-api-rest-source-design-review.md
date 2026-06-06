# Review: 2026-06-05-tdx-api-rest-source-design.md

**Type**: `.md` / **arch** | **Perspective**: architecture, consistency | **Date**: 2026-06-05 | **Reviewer**: Claude

---

## Executive Summary

Well-structured architecture decision document proposing `tdx-api` REST integration into Quantix. All referenced source files and key symbols (`TdxApiConfig`, `TdxApiClient`, `StockQuote`, `Kline`) exist and match claims. Two medium-severity issues found: missing `server-status` endpoint in the existing client, and a dual `TdxApiConfig` definition that the document does not address. The document is close to implementation-ready after addressing the findings.

## Document Metadata

| Field | Value |
|-------|-------|
| Source | `docs/superpowers/specs/2026-06-05-tdx-api-rest-source-design.md` |
| File Type | `.md` |
| Doc Type | arch (architecture design) |
| Sections | 11 |
| Referenced Files | 5 found / 0 missing |
| Referenced Symbols | 6 found / 0 missing |

## Evidence Verification

### Files Referenced

| File | Exists? | Location |
|------|---------|----------|
| `src/sources/tdx_api.rs` | yes | `/opt/claude/quantix-rust/src/sources/tdx_api.rs` |
| `src/cli/handlers/tdx_api_handler.rs` | yes | `/opt/claude/quantix-rust/src/cli/handlers/tdx_api_handler.rs` |
| `src/bridge/client.rs` | yes | `/opt/claude/quantix-rust/src/bridge/client.rs` |
| `tests/bridge_tdx_source_test.rs` | yes | `/opt/claude/quantix-rust/tests/bridge_tdx_source_test.rs` |
| `tdx-api/FUNCTION_TREE.md` | no | Not in this repository (external project) |

### Functions/Classes Referenced

| Symbol | Found? | Location |
|--------|--------|----------|
| `TdxApiConfig` | yes | `src/sources/tdx_api.rs:32` and `src/core/config.rs:66` (dual definition) |
| `TdxApiClient` | yes | `src/sources/tdx_api.rs:402` |
| `StockQuote` | yes | `src/sources/tdx.rs` (re-exported via `use`) |
| `Kline` | yes | `src/data/models.rs` |
| `from_env` | yes | `src/sources/tdx_api.rs:49` (config) and `:423` (client) |
| `from_app_config` | yes | `src/sources/tdx_api.rs:428` |

### Claims Verified

| Claim | Status | Evidence |
|-------|--------|----------|
| `TdxApiClient` exposes REST methods for quote, batch quote, k-line, minute, trades, code search, code lists, workday, market stats, index k-line, k-line history, pull tasks, health | confirmed | Grep found all corresponding `pub async fn` methods in `src/sources/tdx_api.rs` |
| `TdxApiClient` exposes pull task inspection and cancellation | confirmed | `get_task()` at line 991, `cancel_task()` at line 996 |
| `src/bridge/client.rs` has bridge methods for TDX quotes and k-line | confirmed | `fetch_tdx_quotes()` at line 79, `fetch_tdx_kline()` at line 88 |
| `tests/bridge_tdx_source_test.rs` validates quote and k-line model mapping | confirmed | Test functions at lines 8 and 51 |
| Strategy daemon fallback tests exercise primary source with TDX fallback | confirmed | `src/strategy/fallback_loader.rs` has `primary_source_id`, `fallback_used`, and TDX fallback logic |
| Health endpoint returns `status=healthy` (observed) vs documented `code/message` | confirmed | Current `health()` at line 1005 returns raw `serde_json::Value` without shape enforcement |
| `from_env` / `from_app_config` paths exist | confirmed | `from_env()` at line 423, `from_app_config()` at line 428 |
| `TdxApiConfig` has `base_url`, `timeout`, `max_retries` fields | confirmed | `src/sources/tdx_api.rs:32-36` and `src/core/config.rs:66-73` |

## Checklist Results

Architecture checklist:

| # | Check | Result | Notes |
|---|-------|--------|-------|
| A1 | Component boundaries | PASS | Runtime path vs optional MCP path clearly separated |
| A2 | Data flow | PASS | Four-layer flow (command -> source selection -> client -> REST -> model) is explicit |
| A3 | Coupling | PASS | REST client is independent of MCP; bridge layer separate from source layer |
| A4 | Interface contracts | PASS | Source scope table specifies endpoints and expected Quantix behavior |
| A5 | Scalability | PASS | Deferred capability list explicitly scopes growth |
| A6 | Terminology consistency | PASS | `tdx-api`, `TdxApiClient`, `TdxApiConfig` used consistently |
| A7 | Backward compatibility | FAIL | See Finding #1: dual `TdxApiConfig` not addressed |
| A8 | Implementation surface precision | FAIL | See Finding #2: `server-status` endpoint missing from client |
| A9 | Named entities verified | PASS | All referenced files, structs, and functions found in codebase |

Consistency checklist:

| # | Check | Result | Notes |
|---|-------|--------|-------|
| N1 | Terminology | PASS | Consistent use of `tdx-api` as service, `TdxApiClient` as client |
| N2 | Naming conventions | PASS | Follows Rust conventions and project patterns |
| N3 | Formatting | PASS | Uniform heading hierarchy, table formatting, code blocks |
| N4 | Cross-references | PASS | Internal references resolve; external `FUNCTION_TREE.md` noted as out-of-repo |
| N5 | Style consistency | PASS | Formal technical writing throughout |

9 items PASS, 2 items FAIL.

## Findings

### Medium Issues

| # | Section | Issue | Impact | Evidence | Recommendation |
|---|---------|-------|--------|----------|----------------|
| 1 | Configuration (line 120) | Document says "Use or extend the existing `TdxApiConfig` shape" but does not address that `TdxApiConfig` is defined in **two places**: `src/sources/tdx_api.rs:32` (runtime, with `Duration` timeout) and `src/core/config.rs:66` (serde config, with `u64` timeout_secs). The document recommends adding `enabled/disabled flag`, `max batch quote size`, `retry count`, `source priority`, and `health-check timeout` but does not specify which struct to modify or whether to unify them. | Implementation ambiguity: a developer could add fields to one struct but not the other, or create a third config layer. | Grep found `struct TdxApiConfig` at `src/sources/tdx_api.rs:32` (3 fields: `base_url: String`, `timeout: Duration`, `max_retries: u32`) and `src/core/config.rs:66` (3 fields: `base_url: String`, `timeout_secs: u64`, `max_retries: u32`). The document does not mention this duplication. | Add a subsection specifying: (a) which `TdxApiConfig` is the source of truth for new fields, (b) whether the two should be unified, and (c) the mapping between them when `from_app_config` bridges the two. |
| 2 | Source Scope (line 145) and Error Handling (line 173) | Document specifies `/api/server-status` as a required health endpoint ("server-status for upstream TDX connection state"), but the existing `TdxApiClient` has no `server_status()` method. The current `health()` (line 1005) only calls `/api/health` and returns raw JSON. | The document claims reuse of `TdxApiClient` but one of the two required health endpoints does not exist in it yet. This is an implementation gap the document should explicitly call out as new work. | Grep for `server.status` and `server-status` in `src/sources/tdx_api.rs` returned 0 matches. `health()` at line 1005 only calls `/api/health`. The document's Source Scope table at line 145 lists `/api/server-status` as a health capability. | Add `server_status()` to the implementation surface explicitly, noting it is a new method to be added to `TdxApiClient`, or clarify that the existing `health()` should be extended to call both endpoints. |

### Low Issues

| # | Section | Issue | Evidence | Recommendation |
|---|---------|-------|----------|----------------|
| 1 | Configuration (line 131) | Document recommends "max batch quote size, defaulting to the tdx-api documented limit of 50" but `batch_quote()` at `src/sources/tdx_api.rs:611` has no batch size cap or validation. | Read `src/sources/tdx_api.rs:611` — `batch_quote()` passes codes directly without size check. | Note this as explicit new logic to add, not existing behavior to preserve. |
| 2 | Testing (line 193) | Document proposes env var `QUANTIX_TDX_API_LIVE_BASE_URL` for live smoke checks, but current code uses `TDX_API_URL` (see `src/sources/tdx_api.rs:51` and `src/core/config.rs:76`). | Grep for `QUANTIX_TDX_API` returned 0 matches. `TDX_API_URL` is used in both config locations. | Use `TDX_API_URL` for consistency with existing env vars, or explicitly state the new var is separate from the runtime config var. |
| 3 | Current Baseline (line 66) | Document says `src/bridge/client.rs` has "bridge methods for TDX quotes and k-line" which is accurate, but these use the bridge HTTP protocol (`/api/v1/data/tdx/...`), not the direct `tdx-api` REST protocol (`/api/quote`, `/api/kline`). The two are architecturally distinct. | `src/bridge/client.rs:79` calls `/api/v1/data/tdx/quotes` with API key auth. `src/sources/tdx_api.rs:589` calls `/api/quote?code=...` directly. These are different service endpoints. | Clarify that the bridge layer is a separate HTTP service from direct `tdx-api` REST, and this design targets the direct REST path only. |

## Strengths

- All 5 in-repo referenced files and 6 key symbols verified as existing — the baseline assessment is accurate.
- Clear separation of runtime path vs MCP path with explicit "do not" directives in Non-Goals.
- Deferred capability list is specific and defensible (minute/trade data, async tasks, MCP wrapper).
- Error handling classification (7 failure modes) is thorough and maps well to Quantix's `QuantixError` pattern.
- Rollout plan is incremental and realistic (5 phases from spec to MCP decision).

## Recommendations

1. **Unify or document the dual `TdxApiConfig`** — Add a subsection in Configuration specifying which struct owns new fields and whether `src/core/config.rs::TdxApiConfig` should be the single serde source while `src/sources/tdx_api.rs::TdxApiConfig` becomes a derived runtime type. This is the most important clarification for the implementer.

2. **Add `server_status()` to implementation surface** — Explicitly list it as a new method in the Source Scope or Architecture section, since it does not exist in the current client.

3. **Use existing env var naming** — Change `QUANTIX_TDX_API_LIVE_BASE_URL` to `TDX_API_URL` (or explain the separation) to avoid confusion with the already-in-use runtime variable.

4. **Distinguish bridge vs direct paths** — Add one sentence clarifying that `src/bridge/client.rs` calls a separate HTTP service, and this design targets direct `tdx-api` REST endpoints only.

## Scoring

| Dimension | Score (1-5) | Evidence |
|-----------|-------------|----------|
| Technical Accuracy | 5 | All codebase references verified correct; REST probes and known issues well-documented |
| Completeness | 4 | Strong baseline and scope; missing dual-config and server-status details |
| Codebase Alignment | 5 | All 5 referenced files and 6 symbols confirmed; existing client methods accurately enumerated |
| Actionability | 4 | Clear rollout plan and acceptance criteria; config unification needs resolution |
| Terminology Consistency | 5 | Consistent naming throughout; clear distinction between runtime and MCP paths |
| **Overall** | **4.6** | Weighted 2x: Codebase Alignment, Terminology Consistency |

## Verdict

**APPROVE_WITH_NOTES** — Technically accurate and well-aligned with the codebase. The two medium findings (dual `TdxApiConfig` ambiguity and missing `server_status()` method) should be addressed in a revision before implementation planning, but neither blocks spec approval.
