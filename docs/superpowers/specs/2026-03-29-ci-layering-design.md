# CI 分层设计（2026-03-29）

> 状态源说明：本文是 CI 分层设计说明，不作为功能状态注册表。
> 当前功能状态、已设计/待实现项、证据和边界，以根目录 [`FUNCTION_TREE.md`](../../../FUNCTION_TREE.md) 的状态注册表行为准。

## 背景

当前 `.github/workflows/ci.yml` 在 `pull_request` 上会默认执行：

- `lint`
- `test`（含 Postgres / ClickHouse 服务）
- `security`
- `build`（Linux / macOS / Windows 三平台）
- `docs`

这使 PR 路径偏重，反馈速度慢，并且把多平台构建、文档部署准备、依赖过期检查等更适合主分支的任务混在了日常提交流程里。

## 目标

建立分层 CI：

- PR 默认保留必要质量门槛
- 主分支承担更重的完整验证
- `main` 保留最重的报告型或发布型任务

## 方案

### 1. PR 路径

`pull_request` 默认执行：

- `lint`
- `test`
- `security`

其中：

- `lint` 保持格式、clippy、文档链接检查
- `test` 继续保留服务依赖与 doc tests
- `security` 在 PR 上保留 `cargo audit`

### 2. Push 路径

对 `main` / `develop` 的 `push` 执行：

- `lint`
- `test`
- `security`
- `build`
- `docs`

其中：

- `build` 仅在 `push` 路径跑三平台矩阵
- `docs` 生成可在 `push` 保留，Pages deploy 仍只在 `main` 做

### 3. Main 重型路径

仅在 `push` 到 `main` 时执行：

- `coverage`
- `outdated`
- `bench`

实现方式优先：

- 从现有 `test` job 中拆出 `coverage`
- 从现有 `security` job 中拆出 `outdated`
- 保持 `bench` 仅在 `main` 跑

## 具体改法

1. 给 `test` job 增加条件，避免在 PR 上生成 coverage。
2. 给 `security` job 增加条件，避免在 PR 上执行 `cargo outdated --exit-code 1`。
3. 给 `build` job 增加条件，仅在 `push` 到 `main` / `develop` 时执行。
4. `docs` job 保持生成文档，但 deploy 继续限制为 `push main`。
5. 如有必要，将 `coverage` 和 `outdated` 从现有 job 中拆成独立 job，减少条件分支耦合。

## 推荐实现

推荐拆成独立 job，而不是在现有 job 里塞大量 `if:`：

- `security_audit`
- `dependency_outdated`
- `coverage`

理由：

- 可读性更好
- 触发条件更直观
- 后续继续细分 nightly / manual 触发时更容易扩展

## 非目标

- 本轮不重写整个 workflow 结构
- 本轮不引入新的 reusable workflow
- 本轮不新增 schedule 触发器
- 本轮不调整具体测试命令与缓存策略

## 验证

改动后至少验证：

1. YAML 语法有效
2. `pull_request` 下只会运行 `lint`、`test`、`security audit`
3. `push develop` / `push main` 会运行 `build`
4. `push main` 会运行 `coverage`、`outdated`、`bench`
5. docs deploy 仍只在 `push main` 执行
