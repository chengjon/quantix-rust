# Audit Logs Directory

> 状态源说明：本文是代码审计证据，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 `FUNCTION_TREE.md` 的状态注册表为准。

This directory is reserved for long-running gate logs, especially `cargo build --release` logs captured by the hardened audit execution spec.

No release-build log is added by this post-review supplement because the original long-running build was already terminated before this directory contract existed. Future runs should write logs here using the naming shape `cargo-build-release-<timestamp>.log`.
