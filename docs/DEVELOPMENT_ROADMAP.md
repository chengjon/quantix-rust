# Quantix Rust 开发路线图 (更新版 v3.0)

基于 `GAP_ANALYSIS.md` 分析结果、`MIGRATION_FROM_OPENSTOCK.md` 迁移计划以及实际完成进度，制定以下开发计划。

> **重要说明**: 本文档已根据 `ROADMAP_REVIEW.md` 审核意见更新，反映了项目当前实际完成状态。

---

## 当前完成状态

根据 README.md 和代码仓库分析，以下功能已完成：

```
已完成模块:
├── Phase 27: 风控模块 ✅
│   ├── src/risk/ 风险管理框架
│   ├── 规则引擎
│   └── 风控拦截器
├── Phase 29/29B/29C: 策略执行自动化 ✅
│   ├── src/execution/ 执行框架
│   ├── Paper Trading
│   └── QMT Bridge v1 (预览模式)
├── Phase 30: 异常检测 ✅
│   ├── src/anomaly/ 异常检测模块
│   └── 特征工程
├── Phase 9: 东方财富数据源 ✅
│   └── 基础行情数据接入
├── Phase 2: AI 决策模块 ✅
│   ├── src/ai/ LLM 多供应商适配
│   └── OpenAI/DeepSeek/Gemini/Anthropic/Ollama 支持
├── Phase 3: 新闻搜索模块 ✅
│   ├── src/news/ 多源新闻搜索
│   └── Tavily/SerpAPI/Bocha/Brave/SearXNG 支持
├── P0.2: 执行请求生命周期 ✅
│   ├── strategy request show/list 命令
│   └── 多维过滤和统计汇总
├── 算法交易 ✅
│   ├── src/execution/algo/ TWAP/VWAP 执行器
│   └── 状态机、切片计划、CLI 命令
├── Phase 4: 基本面分析 🔨
│   ├── src/fundamental/ 类型和提供商 trait
│   ├── EastMoneyFundamentalProvider
│   └── CLI handlers 已连线 (待: API 响应解析)
├── 舆情分析 🔨
│   ├── src/market/sentiment/ 类型和聚合器
│   └── CLI handlers 已连线 (待: 真实数据提供商)
└── Windows Bridge v1 ✅
    └── WSL2-Windows 桥接
```

---

## 版本规划总览

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                          Quantix Rust 开发路线图                              │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  v0.2.x          v0.3.0          v0.4.0          v0.5.0          v1.0.0      │
│  ───────         ──────          ──────          ──────          ──────      │
│  (已完成)        策略增强        风控完善        研究平台        企业级        │
│                  实盘完善                                        │
│                                                                              │
│              ┌─────┐        ┌─────┐        ┌─────┐        ┌─────┐           │
│              │ 因子 │        │ VaR │        │ ML  │        │ 灾备 │           │
│              │ 库   │        │ 计算 │        │ 框架 │        │ 安全 │           │
│              ├─────┤        ├─────┤        ├─────┤        ├─────┤           │
│  QMT实盘 →  │ 参数 │        │ 压力 │        │ AI  │        │ 合规 │           │
│  算法交易    │ 优化 │        │ 测试 │        │ 多Provider│   │ 风控 │           │
│              ├─────┤        ├─────┤        ├─────┤        ├─────┤           │
│  风控拦截 →  │ 情绪 │        │ 绩效 │        │ 基本 │        │ 多账 │           │
│  邮件通知    │ 分析 │        │ 归因 │        │ 面   │        │ 户   │           │
│              ├─────┤        ├─────┤        ├─────┤        ├─────┤           │
│  Web面板 →  │ 多策 │        │ 组合 │        │ 统计 │        │ 自动 │           │
│  (P2可选)    │ 略   │        │ 再平衡│       │ 套利 │        │ 报告 │           │
│              └─────┘        └─────┘        └─────┘        └─────┘           │
│                                                                              │
│  已完成         4-5周          3-4周          5-7周          4-6周           │
│                                                                              │
│  ★ OpenStock迁移功能已标注                                                    │
│  ✅ 已完成功能已标注                                                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 1: v0.3.0 - 实盘完善 & 策略增强 (P0/P1)

**目标**: 完善实盘交易能力，提升策略研发和管理能力

**周期**: 4-5 周

### 1.1 ✅→🔨 QMT 真实下单接口 (从预览到实盘)

**优先级**: P0 (最高)
**预估工时**: 2-3 天 (框架已完成，仅需 adapter)
**状态**: Phase 29C 执行自动化已完成，需扩展真实下单能力

```
任务清单:
├── [✅] 执行框架 (已完成)
│   ├── src/execution/executor.rs
│   ├── 订单管理
│   └── 状态机
├── [✅] QMT 真实下单 Adapter (2026-03-27 完成)
│   ├── src/execution/qmt_live_adapter.rs
│   ├── 股票买入/卖出真实下单
│   ├── 撤单接口
│   └── 查询接口 (订单/成交/持仓)
├── [ ] 订单状态回调增强
│   ├── 委托成功/失败
│   ├── 成交回报
│   └── 撤单确认
└── [ ] 测试覆盖
    └── 模拟环境集成测试
```

**已有基础**: `src/execution/qmt_bridge.rs`, `src/execution/runtime_store.rs`
**产出**: 扩展 `src/execution/qmt_executor.rs`

### 1.2 算法交易 (TWAP/VWAP)

**优先级**: P0
**预估工时**: 4-5 天

```
任务清单:
├── [ ] 算法交易框架
│   ├── AlgorithmExecutor trait
│   ├── 算法上下文 (Context)
│   └── 算法状态机
├── [ ] TWAP 实现
│   ├── 时间切片
│   ├── 均匀下单
│   └── 随机化抖动
├── [ ] VWAP 实现
│   ├── 成交量预测
│   ├── 按量下单
│   └── 参与率控制
├── [ ] POV (参与率)
│   ├── 实时成交量追踪
│   └── 动态调整
├── [ ] 算法监控
│   ├── 执行进度
│   ├── 滑点统计
│   └── 算法日志
└── [ ] CLI 命令
    ├── quantix execution algo list
    └── quantix execution algo run --type twap
```

**产出**: `src/execution/algo/`

### 1.3 Web 监控面板

**优先级**: P2 (降级，CLI + TUI 已满足当前需求)
**预估工时**: 5-7 天

```
任务清单:
├── [ ] 后端 API
│   ├── 系统状态 API
│   ├── 持仓查询 API
│   ├── PnL 查询 API
│   ├── 告警列表 API
│   └── WebSocket 实时推送
├── [ ] 前端框架
│   ├── React/Vue 脚手架
│   ├── 路由配置
│   └── 状态管理
├── [ ] 核心页面
│   ├── Dashboard (总览)
│   ├── Positions (持仓)
│   ├── Orders (订单)
│   ├── PnL (盈亏)
│   └── Alerts (告警)
├── [ ] 实时更新
│   ├── WebSocket 连接
│   ├── 数据刷新
│   └── 状态同步
└── [ ] 部署配置
    ├── 静态资源打包
    └── Nginx 配置模板
```

**产出**: `web/dashboard/`

### 1.4 ✅→🔨 实时风险拦截 (增强)

**优先级**: P0
**预估工时**: 1-2 天 (已与现有 risk 模块整合)
**状态**: Phase 27 风控模块已完成，需与执行层集成

```
任务清单:
├── [✅] 风控框架 (已完成)
│   ├── src/risk/manager.rs
│   ├── src/risk/rules.rs
│   └── 规则引擎
├── [→] 交易前风控检查集成
│   ├── 与 ExecutionRuntime 集成
│   ├── 单笔限额检查
│   ├── 日内累计限额
│   └── 黑名单拦截
├── [→] 实时敞口监控
│   ├── 单股票敞口
│   ├── 行业敞口
│   └── 市场敞口
├── [ ] 异常订单检测
│   ├── 价格异常 (偏离过大)
│   ├── 数量异常 (异常大单)
│   └── 频率异常 (频繁下单)
└── [ ] 拦截日志
    ├── 拦截记录
    └── 拦截统计
```

**已有基础**: `src/risk/`
**产出**: `src/risk/interceptor.rs`

### 1.5 ★ 邮件通知服务 (OpenStock 迁移)

**优先级**: P0 (从 OpenStock 迁移)
**预估工时**: 3-4 天

```
来源: MIGRATION_FROM_OPENSTOCK.md Section 2

任务清单:
├── [ ] SMTP 发送器
│   ├── lettre 库集成
│   ├── Gmail SMTP 配置
│   └── 连接池管理
├── [ ] 通知事件类型
│   ├── PriceAlert (价格告警)
│   ├── VolumeAlert (成交量异常)
│   ├── StrategySignal (策略信号)
│   └── DailySummary (每日汇总)
├── [ ] HTML 邮件模板
│   ├── handlebars 模板引擎
│   ├── 价格上涨提醒模板
│   ├── 价格下跌提醒模板
│   └── 成交量异常模板
├── [ ] 多渠道分发
│   ├── Email 发送
│   ├── Webhook 发送 (已有基础)
│   └── 统一分发接口
├── [ ] 与现有通知系统集成
│   ├── 扩展 NotificationChannel
│   ├── 集成到 NotificationService
│   └── 与 Monitor 服务联动
└── [ ] 环境配置
    ├── SMTP_* 环境变量
    └── 收件人配置
```

**依赖**: `src/monitoring/notification.rs` (已有)
**产出**: `src/notification/email/`

### 1.6 因子库系统

**优先级**: P1
**预估工时**: 7-10 天

```
任务清单:
├── [ ] 因子定义框架
│   ├── Factor trait
│   ├── 因子元数据
│   └── 因子依赖声明
├── [ ] 因子计算引擎
│   ├── 批量计算
│   ├── 增量计算
│   └── 并行优化
├── [ ] 因子存储
│   ├── SQLite 存储
│   ├── 历史快照
│   └── 查询接口
├── [ ] 因子检验
│   ├── IC 计算
│   ├── IR 计算
│   ├── 分组回测
│   └── 因子衰减分析
├── [ ] 内置因子库
│   ├── 动量因子
│   ├── 价值因子
│   ├── 成长因子
│   ├── 质量因子
│   └── 波动因子
├── [ ] CLI 命令
│   ├── quantix factor list
│   ├── quantix factor calc
│   └── quantix factor test
└── [ ] 文档
    ├── 因子开发指南
    └── 因子库 API
```

**产出**: `src/factor/`

### 1.7 ★ 情绪分析模块 (OpenStock 迁移)

**优先级**: P1 (从 OpenStock 迁移)
**预估工时**: 5-7 天

```
来源: MIGRATION_FROM_OPENSTOCK.md Section 1

任务清单:
├── [ ] 数据类型定义
│   ├── SentimentSource (Reddit/X/News/Polymarket)
│   ├── SentimentTrend (Rising/Falling/Stable)
│   ├── SourceInsight (单源情绪)
│   ├── StockSentiment (聚合情绪)
│   └── SourceAlignment (来源一致性)
├── [ ] Adanos API 客户端
│   ├── HTTP 客户端封装
│   ├── 多数据源并行请求
│   ├── 错误处理与重试
│   └── 响应解析
├── [ ] 情绪聚合器
│   ├── 多源数据聚合
│   ├── 平均热度计算
│   ├── 看涨比例计算
│   └── 一致性判断算法
├── [ ] 策略信号生成
│   ├── BullishAlignment → Long 信号
│   ├── BearishAlignment → Short 信号
│   ├── WideDivergence → Watch 信号
│   └── 与 Strategy trait 集成
├── [ ] CLI 命令
│   └── quantix sentiment <symbol> [--days N]
├── [ ] 数据存储
│   ├── 历史情绪数据存储
│   └── 情绪趋势追踪
└── [ ] 环境配置
    ├── ADANOS_API_KEY
    └── ADANOS_BASE_URL
```

**产出**: `src/sentiment/`

**信号生成逻辑**:
```
BullishAlignment + bullish_average >= 65% → Strong Long
BearishAlignment + bullish_average <= 35% → Moderate Short
WideDivergence → Watch (高波动机会)
```

### 1.8 策略参数优化

**优先级**: P1
**预估工时**: 5-7 天

```
任务清单:
├── [ ] 参数空间定义
│   ├── 参数类型 (int/float/enum)
│   ├── 参数范围
│   └── 参数约束
├── [ ] 优化算法
│   ├── 网格搜索 (Grid Search)
│   ├── 随机搜索 (Random Search)
│   └── 贝叶斯优化 (可选)
├── [ ] 评估指标
│   ├── 收益率
│   ├── Sharpe Ratio
│   ├── 最大回撤
│   └── 自定义指标
├── [ ] Walk-Forward 分析
│   ├── 滚动窗口
│   ├── 样本外验证
│   └── 稳定性评分
├── [ ] 结果可视化
│   ├── 参数热力图
│   ├── 收益分布图
│   └── 优化报告
└── [ ] CLI 命令
    ├── quantix strategy optimize
    └── quantix strategy walk-forward
```

**产出**: `src/strategy/optimizer.rs`

### 1.9 多策略组合管理

**优先级**: P1
**预估工时**: 4-5 天

```
任务清单:
├── [ ] 策略组合模型
│   ├── PortfolioStrategy
│   ├── 策略权重配置
│   └── 组合信号聚合
├── [ ] 资金分配
│   ├── 等权分配
│   ├── 风险平价
│   └── 自定义分配
├── [ ] 策略相关性
│   ├── 收益相关性计算
│   ├── 信号相关性
│   └── 相关性矩阵
├── [ ] 动态调整
│   ├── 定期再平衡
│   ├── 表现驱动调整
│   └── 调整日志
└── [ ] 组合报告
    ├── 组合绩效
    └── 策略贡献度
```

**产出**: `src/strategy/portfolio.rs`

### 1.10 基本面数据接入

**优先级**: P1
**预估工时**: 2-3 天 (东财数据源 Phase 9 已接入)
**状态**: 基础行情数据已有，需扩展财务数据

```
任务清单:
├── [✅] 行情数据源 (已完成)
│   └── 东方财富行情 API
├── [ ] 财务数据扩展
│   ├── 资产负债表
│   ├── 利润表
│   ├── 现金流量表
│   └── 财务指标
├── [ ] 数据模型
│   ├── 财务报表结构
│   └── 计算指标
├── [ ] 数据存储
│   ├── PostgreSQL 表设计
│   ├── 历史快照
│   └── 增量更新
└── [ ] CLI 命令
    ├── quantix data financial
    └── quantix data indicators
```

**已有基础**: `src/sources/eastmoney/`
**产出**: `src/data/fundamental/`

---

## Phase 2: v0.4.0 - 风控完善 (P1)

**目标**: 建立完整的风险管理体系

**周期**: 3-4 周

### 2.1 VaR 计算

**优先级**: P1
**预估工时**: 4-5 天

```
任务清单:
├── [ ] 历史模拟法
│   ├── 历史收益分布
│   ├── 分位数计算
│   └── 置信区间
├── [ ] 参数法
│   ├── 协方差矩阵
│   ├── 正态分布假设
│   └── Cornish-Fisher 展开
├── [ ] 蒙特卡洛模拟
│   ├── 收益分布拟合
│   ├── 随机模拟
│   └── 并行计算
├── [ ] CVaR (条件风险价值)
│   ├── 尾部风险计算
│   └── ES (Expected Shortfall)
├── [ ] VaR 报告
│   ├── 日度 VaR
│   ├── VaR 趋势
│   └── 回测验证
└── [ ] CLI 命令
    ├── quantix risk var
    └── quantix risk cvar
```

**产出**: `src/risk/var.rs`

### 2.2 压力测试

**优先级**: P1
**预估工时**: 4-5 天

```
任务清单:
├── [ ] 情景定义
│   ├── 历史情景
│   ├── 假设情景
│   └── 自定义情景
├── [ ] 内置情景库
│   ├── 2008 金融危机
│   ├── 2015 股灾
│   ├── 2020 疫情
│   └── 流动性枯竭
├── [ ] 情景分析
│   ├── 组合价值冲击
│   ├── 敞口变化
│   └── 流动性影响
├── [ ] 敏感性分析
│   ├── 因子敏感性
│   ├── 参数敏感性
│   └── 敏感性矩阵
└── [ ] 报告生成
    ├── 压力测试报告
    └── 风险预警
```

**产出**: `src/risk/stress_test.rs`

### 2.3 绩效归因

**优先级**: P1
**预估工时**: 4-5 天

```
任务清单:
├── [ ] Brinson 归因
│   ├── 配置效应
│   ├── 选择效应
│   └── 交互效应
├── [ ] 因子归因
│   ├── 风险因子暴露
│   ├── 因子收益
│   └── 特质收益
├── [ ] 行业归因
│   ├── 行业配置贡献
│   └── 行业内选股贡献
├── [ ] 归因报告
│   ├── 多期归因
│   ├── 归因可视化
│   └── PDF 报告生成
└── [ ] CLI 命令
    ├── quantix analyze attribution
    └── quantix analyze brinson
```

**产出**: `src/analysis/attribution.rs`

### 2.4 组合再平衡

**优先级**: P1
**预估工时**: 3-4 天

```
任务清单:
├── [ ] 再平衡触发
│   ├── 定期触发 (日/周/月)
│   ├── 阈值触发 (偏离度)
│   └── 信号触发
├── [ ] 再平衡优化
│   ├── 目标权重计算
│   ├── 交易成本考虑
│   └── 税务优化 (可选)
├── [ ] 再平衡执行
│   ├── 交易清单生成
│   ├── 批量下单
│   └── 执行监控
└── [ ] 再平衡报告
    ├── 调整明细
    └── 成本分析
```

**产出**: `src/portfolio/rebalance.rs`

---

## Phase 3: v0.5.0 - 研究平台 (P2)

**目标**: 提供专业级量化研究能力，集成 AI 能力

**周期**: 5-7 周

### 3.1 ★ AI 多 Provider (OpenStock 迁移)

**优先级**: P1 (从 OpenStock 迁移)
**预估工时**: 4-5 天

```
来源: MIGRATION_FROM_OPENSTOCK.md Section 3

任务清单:
├── [ ] Provider Trait 定义
│   ├── AIProvider trait
│   ├── GenerateOptions (temperature/max_tokens)
│   └── GenerateResponse (text/tokens_used/model)
├── [ ] Gemini Provider
│   ├── 原生 REST API 调用
│   ├── 请求/响应解析
│   └── API Key 管理
├── [ ] OpenAI 兼容 Provider
│   ├── MiniMax 实现
│   ├── Siray 实现
│   └── DeepSeek (扩展)
├── [ ] 统一客户端
│   ├── 环境变量配置
│   ├── 自动降级逻辑
│   ├── 超时处理
│   └── 重试机制
├── [ ] 预定义提示词
│   ├── 股票分析提示词
│   ├── 新闻摘要提示词
│   ├── 市场情绪解读
│   └── 自定义提示词模板
├── [ ] CLI 命令
│   ├── quantix ai analyze <symbol>
│   └── quantix ai summarize
└── [ ] 环境配置
    ├── AI_PROVIDER (gemini/minimax/siray)
    ├── GEMINI_API_KEY
    ├── MINIMAX_API_KEY
    └── SIRAY_API_KEY
```

**产出**: `src/ai/`

**自动降级逻辑**:
```
Primary (Gemini) → Fallback (MiniMax) → Fallback (Siray)
```

### 3.2 机器学习框架

**优先级**: P2
**预估工时**: 7-10 天

```
任务清单:
├── [ ] 特征工程
│   ├── 特征提取
│   ├── 特征选择
│   ├── 特征标准化
│   └── 特征存储
├── [ ] 模型训练
│   ├── 训练框架 (Linfa/SmartCore)
│   ├── 交叉验证
│   ├── 超参数调优
│   └── 模型持久化
├── [ ] 模型评估
│   ├── 分类评估 (AUC/F1)
│   ├── 回归评估 (RMSE/MAE)
│   └── 样本外测试
├── [ ] 在线推理
│   ├── 模型加载
│   ├── 实时预测
│   └── 预测缓存
├── [ ] ML 策略模板
│   ├── 预测型策略
│   ├── 分类型策略
│   └── 强化学习 (可选)
└── [ ] CLI 命令
    ├── quantix ml train
    ├── quantix ml predict
    └── quantix ml evaluate
```

**产出**: `src/ml/`

### 3.3 统计套利

**优先级**: P2
**预估工时**: 5-7 天

```
任务清单:
├── [ ] 配对交易
│   ├── 协整检验
│   ├── 配对筛选
│   ├── 价差计算
│   └── 开平仓信号
├── [ ] 配对管理
│   ├── 配对池维护
│   ├── 配对有效性监控
│   └── 自动换仓
├── [ ] 统计套利组合
│   ├── 多配对组合
│   ├── 风险对冲
│   └── 资金分配
└── [ ] CLI 命令
    ├── quantix strategy pairs
    └── quantix strategy cointegration
```

**产出**: `src/strategy/pairs_trading.rs`

### 3.4 事件研究

**优先级**: P2
**预估工时**: 4-5 天

```
任务清单:
├── [ ] 事件定义
│   ├── 事件类型
│   ├── 事件窗口
│   └── 估计窗口
├── [ ] 事件分析
│   ├── 正常收益估计
│   ├── 异常收益计算
│   └── CAR (累积异常收益)
├── [ ] 事件驱动策略
│   ├── 事件信号生成
│   └── 策略回测
└── [ ] 事件数据库
    ├── 事件存储
    └── 事件查询
```

**产出**: `src/analysis/event_study.rs`

### 3.5 择时模型

**优先级**: P2
**预估工时**: 4-5 天

```
任务清单:
├── [ ] 趋势择时
│   ├── 均线系统
│   ├── 突破系统
│   └── 趋势强度
├── [ ] 均值回归择时
│   ├── 布林带
│   ├── RSI 极值
│   └── Z-Score
├── [ ] 波动率择时
│   ├── 波动率 regime
│   └── VIX 类指标
├── [ ] 板块轮动
│   ├── 强势板块识别
│   └── 轮动信号
└── [ ] 择时评估
    ├── 择时收益
    └── 择时准确性
```

**产出**: `src/strategy/timing/`

---

## Phase 4: v1.0.0 - 企业级 (P2/P3)

**目标**: 达到生产级稳定性和合规要求

**周期**: 4-6 周

### 4.1 多账户管理

**优先级**: P2
**预估工时**: 5-7 天

```
任务清单:
├── [ ] 账户组管理
│   ├── 账户组定义
│   ├── 账户聚合视图
│   └── 跨账户查询
├── [ ] 资金调度
│   ├── 资金划转
│   ├── 资金利用率
│   └── 调度日志
├── [ ] 订单路由
│   ├── 智能拆单
│   ├── 最优执行场所
│   └── 路由规则
├── [ ] 多券商适配
│   ├── 券商适配器抽象
│   ├── 统一订单接口
│   └── 状态同步
└── [ ] CLI 命令
    ├── quantix account list
    ├── quantix account group
    └── quantix account transfer
```

**产出**: `src/account/`

### 4.2 合规风控

**优先级**: P3
**预估工时**: 4-5 天

```
任务清单:
├── [ ] 监管规则
│   ├── 涨跌停限制
│   ├── 停牌处理
│   ├── ST 限制
│   └── 交易时间限制
├── [ ] 合规检查
│   ├── 内幕交易检测
│   ├── 关联交易检测
│   └── 异常交易检测
├── [ ] 审计日志
│   ├── 操作日志
│   ├── 数据变更日志
│   └── 日志归档
└── [ ] 合规报告
    ├── 定期报告
    └── 专项报告
```

**产出**: `src/compliance/`

### 4.3 灾备与高可用

**优先级**: P3
**预估工时**: 5-7 天

```
任务清单:
├── [ ] 数据备份
│   ├── 数据库备份
│   ├── 配置备份
│   └── 自动备份调度
├── [ ] 故障恢复
│   ├── 数据恢复
│   ├── 服务恢复
│   └── 恢复演练
├── [ ] 高可用架构
│   ├── 主备切换
│   ├── 健康检查
│   └── 自动故障转移
└── [ ] 容灾预案
    ├── 预案文档
    └── 演练记录
```

**产出**: `scripts/backup/`, `scripts/ha/`

### 4.4 安全机制

**优先级**: P3
**预估工时**: 4-5 天

```
任务清单:
├── [ ] 身份认证
│   ├── 用户管理
│   ├── 密码策略
│   └── Token 认证
├── [ ] 权限控制
│   ├── RBAC 模型
│   ├── 权限配置
│   └── 访问控制
├── [ ] 数据安全
│   ├── 敏感数据加密
│   ├── 传输加密 (TLS)
│   └── 密钥管理
└── [ ] 安全审计
    ├── 登录审计
    └── 操作审计
```

**产出**: `src/security/`

### 4.5 自动化报告

**优先级**: P2
**预估工时**: 3-4 天

```
任务清单:
├── [ ] 报告模板
│   ├── 日报模板
│   ├── 周报模板
│   └── 月报模板
├── [ ] 报告生成
│   ├── 数据聚合
│   ├── 模板渲染
│   └── PDF 生成
├── [ ] 报告推送
│   ├── 邮件推送 (复用 1.5)
│   ├── 钉钉推送
│   └── 企业微信推送
└── [ ] 报告归档
    ├── 历史报告
    └── 报告检索
```

**产出**: `src/report/`

---

## ★ OpenStock 迁移功能汇总

| 功能 | 阶段 | 优先级 | 工时 | 依赖 |
|------|------|--------|------|------|
| **邮件通知服务** | Phase 1.5 | P0 | 3-4天 | notification.rs |
| **情绪分析模块** | Phase 1.7 | P1 | 5-7天 | 策略系统 |
| **AI 多 Provider** | Phase 3.1 | P1 | 4-5天 | HTTP 客户端 |

**迁移总工时**: 12-16 天

---

## 里程碑时间线

```
2026-Q2 (4-6月)
├── Week 1-2:   Phase 1.1-1.2 (QMT实盘完善 + 算法交易)
├── Week 3:     Phase 1.4 (风控拦截集成)
├── Week 4-5:   Phase 1.5 (★邮件通知) + Phase 1.6 (因子库开始)
├── Week 6-8:   Phase 1.6-1.7 (因子库 + ★情绪分析)
├── Week 9-10:  Phase 1.8-1.9 (参数优化 + 多策略)
├── Week 11:    Phase 1.10 (基本面数据)
└── Week 12:    Phase 1.3 (Web面板 - 可选)
    → v0.3.0 发布 (实盘完善 + 策略增强 + 邮件/情绪)

2026-Q3 (7-9月)
├── Week 13-15: Phase 2.1-2.2 (VaR + 压力测试)
├── Week 16-18: Phase 2.3-2.4 (绩效归因 + 再平衡)
├── Week 19-20: Phase 3.1 (★AI 多 Provider)
├── Week 21-24: Phase 3.2-3.4 (ML框架 + 统计套利 + 事件)
└── Week 25:    Phase 3.5 (择时模型)
    → v0.4.0 发布 (风控完善)
    → v0.5.0 发布 (研究平台 + AI)

2026-Q4 (10-12月)
├── Week 26-29: Phase 4.1-4.2 (多账户 + 合规)
├── Week 30-33: Phase 4.3-4.5 (灾备 + 安全 + 报告)
└── Week 34-38: 集成测试 + 性能优化 + 文档完善
    → v1.0.0 发布 (企业级)
```

---

## 资源需求

### 人力

| 角色 | 人数 | 职责 | 备注 |
|------|------|------|------|
| 后端开发 | 1-2 | 核心功能开发 | Rust 经验 |
| 测试 | 1-2 | 测试用例 + 自动化 | 需加强自动化测试覆盖 |
| 运维 | 0.5 | 部署 + 监控 (兼职) | |

> **注意**: 前端开发人员已移除。CLI + TUI 已满足当前交互需求。Web 面板降级为 P2 可选功能。

### 基础设施

| 资源 | 用途 |
|------|------|
| 开发服务器 | 本地开发测试 |
| 测试环境 | 集成测试 |
| 数据库 | PostgreSQL + ClickHouse |
| 行情源 | 东方财富/AKShare API |
| 实盘模拟 | QMT 模拟账户 |
| ★ Adanos API | 情绪数据源 |
| ★ LLM API | Gemini/MiniMax/Siray |

---

## 新增依赖 (OpenStock 迁移)

```toml
# Cargo.toml 新增
[dependencies]

# 情绪分析
reqwest = { version = "0.12", features = ["json"] }

# 邮件通知
lettre = "0.11"
handlebars = "5.1"

# AI Provider
async-trait = "0.1"
```

---

## 新增环境变量 (OpenStock 迁移)

```bash
# .env 新增

# === 情绪分析 ===
ADANOS_API_KEY=your_key
ADANOS_BASE_URL=https://api.adanos.org

# === 邮件通知 ===
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your_email@gmail.com
SMTP_PASS=your_app_password
EMAIL_FROM_NAME=Quantix
NOTIFICATION_RECIPIENTS=user1@example.com,user2@example.com

# === AI Provider ===
AI_PROVIDER=gemini
GEMINI_API_KEY=your_key
MINIMAX_API_KEY=your_key
SIRAY_API_KEY=your_key
```

---

## 风险与依赖

### 技术风险

| 风险 | 影响 | 缓解措施 |
|------|------|----------|
| QMT 接口变更 | 实盘功能受阻 | 抽象层隔离，快速适配 |
| 第三方 API 限流 | 数据获取不稳定 | 本地缓存 + 多源备份 |
| ★ Adanos API 可用性 | 情绪分析受阻 | 多源备份 (雪球/东财) |
| ★ LLM API 成本 | AI 分析受限 | 多 Provider + 降级 |
| 性能瓶颈 | 实时性下降 | 性能测试 + 优化 |
| **Windows 桥接依赖** | QMT 功能仅 Windows 可用 | 保留跨平台抽象层 |
| **Python 数据层耦合** | Schema 同步成本 | 接口层解耦 |
| **Rust ML 生态薄弱** | ML 实现成本高 | 考虑 Python 集成或外部服务 |

### 外部依赖

| 依赖 | 状态 | 备选方案 |
|------|------|----------|
| QMT 客户端 | 需安装 (仅 Windows) | - |
| 东方财富 API | 公开 | AKShare 备选 |
| ★ Adanos API | 需注册 | 雪球/东财股吧爬虫 |
| ★ LLM API | 需注册 | 多 Provider 互备 |
| PostgreSQL | 稳定 | - |
| ClickHouse | 可选 | PostgreSQL 替代 |
| Python quantix | 数据层依赖 | 接口层解耦 |

---

## 现有模块整合说明

### 风控模块 (Phase 27 已完成)

```
src/risk/
├── manager.rs      # 风险管理器
├── rules.rs        # 规则定义
├── limits.rs       # 限额管理
└── position.rs     # 持仓风险

整合要点:
├── 新增拦截器与现有 RiskManager 集成
├── 复用现有规则引擎
└── 扩展实时监控能力
```

### 执行模块 (Phase 29/29B/29C 已完成)

```
src/execution/
├── executor.rs        # 执行器 trait
├── qmt_executor.rs    # QMT 执行器
├── qmt_bridge.rs      # QMT 桥接
├── runtime_store.rs   # 运行时存储
├── paper_executor.rs  # Paper Trading
└── order.rs           # 订单模型

整合要点:
├── QMT 真实下单扩展现有 QmtExecutor
├── 算法交易继承 Executor trait
└── 风控拦截集成到执行流程
```

### 异常检测模块 (Phase 30 已完成)

```
src/anomaly/
├── detector.rs    # 异常检测器
├── features.rs    # 特征工程
└── filter.rs      # 过滤器

整合要点:
├── 与风控拦截器联动
└── 异常信号触发风控动作
```

---

## 验收标准

### v0.3.0 (实盘完善 + 策略增强 + 邮件/情绪)

- [ ] QMT 真实下单成功率 > 99%
- [ ] 算法交易执行偏差 < 0.5%
- [ ] 风控拦截与执行层集成完成
- [ ] ★ 邮件通知送达率 > 95%
- [ ] 因子库包含 20+ 常用因子
- [ ] 参数优化支持 3 种算法
- [ ] ★ 情绪分析支持 4 个数据源
- [ ] ★ 情绪信号可被策略使用

### v0.4.0 (风控完善)

- [ ] VaR 计算误差 < 5%
- [ ] 压力测试覆盖 5+ 历史情景
- [ ] 绩效归因报告自动化

### v0.5.0 (研究平台 + AI)

- [ ] ★ AI 分析支持 3+ Provider
- [ ] ★ Provider 自动降级正常
- [ ] ML 模型训练流程完整
- [ ] 统计套利配对交易可用

### v1.0.0 (企业级)

- [ ] 系统可用性 > 99.9%
- [ ] 数据备份 RPO < 1小时
- [ ] 安全审计 100% 覆盖

---

*文档版本: 3.0*
*创建日期: 2026-03-27*
*最后更新: 2026-03-27*
*更新内容: 根据 ROADMAP_REVIEW.md 审核意见更新，反映实际完成状态*
