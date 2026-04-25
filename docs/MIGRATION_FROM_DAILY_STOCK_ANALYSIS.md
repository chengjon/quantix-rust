# daily_stock_analysis → quantix-rust 功能迁移方案

**源项目**: daily_stock_analysis (Python) - AI驱动的股票分析系统
**目标项目**: quantix-rust (Rust) - 量化交易CLI工具
**创建日期**: 2026-03-27
**状态**: ✅ 方案已确认

---

## 一、功能对比矩阵

| 功能领域 | daily_stock_analysis | quantix-rust | 迁移优先级 |
|----------|---------------------|--------------|-----------|
| AI决策分析 | ✅ LLM多模型 | ❌ 无 | **P0** |
| 新闻搜索 | ✅ 6种搜索源 | ❌ 无 | **P0** |
| 多渠道通知 | ✅ 7+渠道 | 部分（3种） | **P0** |
| 舆情分析 | ✅ Reddit/X/Polymarket | ❌ 无 | P1 |
| Agent策略问股 | ✅ 11种策略 | 部分（无对话） | P1 |
| 基本面数据 | ✅ AkShare适配器 | ❌ 无 | P1 |
| 智能导入 | ✅ 图片/文件/剪贴板 | ❌ 无 | P2 |
| 大盘复盘 | ✅ 日度复盘 | ✅ market命令 | ✅已有 |
| 技术指标 | ✅ 基础指标 | ✅ 12种指标 | ✅已有 |
| 回测引擎 | ✅ AI回测验证 | ✅ 完整回测 | ✅已有 |
| 风控系统 | ❌ 无 | ✅ 完整风控 | ✅已有 |
| 策略执行 | ❌ 无 | ✅ paper/mock_live + guarded qmt_live | ✅已有 |
| 异常检测 | ❌ 无 | ✅ Isolation Forest | ✅已有 |

---

## 二、吸收策略：增强而非替换

quantix-rust 已有完整的交易执行链路，吸收重点是用 AI 能力**增强决策层**。

### 核心原则
1. **CLI优先**: 保持命令行工具定位，不引入Web UI
2. **模块化**: 新功能作为独立模块，可选择性启用
3. **配置驱动**: 通过配置文件和环境变量控制
4. **渐进式**: 分阶段实施，每阶段独立可用

---

## 三、分阶段实施方案

### Phase 1: 通知模块增强 (1周)
**优先级**: P0 | **依赖**: 无 | **状态**: 📋 待实施

**目标**: 扩展现有 `monitoring/notification` 模块

```
现有结构:
src/monitoring/notification/
├── notification_service.rs  # 已有基础通知
├── desktop_sender.rs        # 桌面通知
├── webhook_sender.rs        # HTTP POST
└── log_sender.rs            # 日志

新增结构:
src/monitoring/notification/
├── providers/               # 新增Provider目录
│   ├── mod.rs
│   ├── wechat.rs           # 企业微信
│   ├── feishu.rs           # 飞书
│   ├── telegram.rs         # Telegram
│   ├── discord.rs          # Discord
│   ├── slack.rs            # Slack
│   ├── dingtalk.rs         # 钉钉
│   └── pushplus.rs         # PushPlus
├── renderer.rs             # 消息渲染（Markdown→图片）
└── markdown_to_image.rs    # MD转图片（用于不支持MD的渠道）
```

**CLI 命令**:
```bash
quantix notify send --channel telegram --message "测试"
quantix notify send --channel wechat,feishu --file report.md
quantix notify test --channel all
```

**验收标准**:
- [ ] 支持5+通知渠道
- [ ] CLI可测试各渠道连通性
- [ ] 支持Markdown消息
- [ ] 支持消息模板

---

### Phase 2: AI 决策模块 (2周)
**优先级**: P0 | **依赖**: Phase 1 | **状态**: 📋 待实施

**目标**: 新建独立 AI 模块，与现有策略层集成

```
新建结构:
src/ai/
├── mod.rs
├── llm_adapter.rs          # LLM统一适配器（OpenAI协议）
├── providers/              # 各LLM实现
│   ├── mod.rs
│   ├── openai.rs           # OpenAI/DeepSeek兼容
│   ├── gemini.rs           # Google Gemini
│   ├── anthropic.rs        # Claude
│   └── ollama.rs           # 本地模型
├── prompt_templates.rs      # Prompt模板系统
├── decision_engine.rs       # 决策引擎
├── conversation.rs          # 对话上下文管理
└── skill_registry.rs        # 策略技能注册

配置文件:
config/ai.toml              # AI配置

与现有策略层集成:
src/strategy/
├── chat.rs                 # 新增：对话式策略执行
└── skills/                 # 新增：策略技能包
    ├── mod.rs
    ├── chan_theory.rs      # 缠论
    ├── wave_theory.rs      # 波浪理论
    ├── divergence.rs       # 背离
    └── support_resistance.rs
```

**CLI 命令**:
```bash
quantix ai ask "分析 600519 的买卖点"
quantix ai decision --code 600519 --model deepseek-chat
quantix strategy ask "用缠论分析茅台" --skill chan_theory
```

**验收标准**:
- [ ] 支持3+ LLM提供商
- [ ] 对话上下文保持
- [ ] 决策仪表盘生成
- [ ] 策略技能注册表

---

### Phase 3: 新闻搜索模块 (1.5周)
**优先级**: P0 | **依赖**: 无 | **状态**: 📋 待实施

**目标**: 新建独立新闻模块

```
新建结构:
src/news/
├── mod.rs
├── provider.rs             # NewsProvider trait
├── providers/
│   ├── mod.rs
│   ├── tavily.rs           # Tavily（高质量AI友好）
│   ├── serpapi.rs          # SerpAPI（全渠道）
│   ├── bocha.rs            # 博查（中文优化）
│   ├── brave.rs            # Brave（隐私优先）
│   └── searxng.rs          # SearXNG（自建）
├── aggregator.rs           # 多源聚合+fallback
├── cache.rs                # 新闻缓存
└── types.rs                # 数据类型

配置文件:
config/news.toml            # 新闻源配置
```

**CLI 命令**:
```bash
quantix news search --code 600519 --days 3
quantix news search --code AAPL --provider tavily
quantix news trend --date 2026-03-27
```

**验收标准**:
- [ ] 支持3+搜索源
- [ ] 自动fallback机制
- [ ] 新闻缓存
- [ ] 股票代码关联搜索

---

### Phase 4: 基本面 + 舆情 (1周)
**优先级**: P1 | **依赖**: Phase 3 | **状态**: 📋 待实施

**目标**: 扩展现有 `market/` 模块

```
扩展结构:
src/fundamental/            # 新增独立模块
├── mod.rs
├── provider.rs             # FundamentalProvider trait
├── akshare.rs              # AkShare适配器
├── eastmoney.rs            # 东财适配器
├── types.rs                # 数据类型
├── valuation.rs            # 估值指标 PE/PB/PS
├── earnings.rs             # 财报数据
├── institution.rs          # 机构持仓
├── capital_flow.rs         # 资金流向
├── dragon_tiger.rs         # 龙虎榜
└── dividend.rs             # 分红信息

src/market/sentiment/       # 新增：舆情分析（美股）
├── mod.rs
├── adanos.rs               # Adanos API
└── aggregator.rs           # 情绪聚合
```

**CLI 命令**:
```bash
quantix fundamental show --code 600519
quantix fundamental valuation --code 000001
quantix fundamental dividend --code 600519 --years 3
quantix sentiment show --code AAPL  # 美股舆情
```

**验收标准**:
- [ ] 估值/财报数据可查询
- [ ] 龙虎榜数据可获取
- [ ] 美股情绪数据可展示
- [ ] 数据缓存机制

---

### Phase 5: 智能导入 (可选)
**优先级**: P2 | **依赖**: Phase 2 | **状态**: 📋 可选

```
新建结构:
src/import/
├── mod.rs
├── image_extractor.rs      # 图片识别（LLM Vision）
├── csv_parser.rs           # CSV解析
├── excel_parser.rs         # Excel解析
├── clipboard.rs            # 剪贴板解析
└── code_resolver.rs        # 代码/名称联想
```

**CLI 命令**:
```bash
quantix import from-image --file screenshot.png
quantix import from-csv --file stocks.csv
quantix import from-clipboard
```

---

## 四、模块依赖关系图

```
                    ┌─────────────────────────────────────┐
                    │           CLI Layer                 │
                    │         (cli/handlers)              │
                    └──────────────┬──────────────────────┘
                                   │
       ┌───────────────┬───────────┼───────────┬───────────────┐
       │               │           │           │               │
       ▼               ▼           ▼           ▼               ▼
┌─────────────┐ ┌───────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────┐
│   新增 AI    │ │ 新增 News │ │ 策略层  │ │ 执行层  │ │ 扩展 Market │
│   src/ai/   │ │ src/news/ │ │strategy │ │execution│ │ fundamental │
└──────┬──────┘ └─────┬─────┘ └────┬────┘ └────┬────┘ └──────┬──────┘
       │              │            │           │             │
       └──────────────┼────────────┼───────────┼─────────────┘
                      │            │           │
                      ▼            ▼           ▼
              ┌─────────────────────────────────────┐
              │     扩展 Notification (monitoring)  │
              │     src/monitoring/notification/    │
              └──────────────────┬──────────────────┘
                                 │
                                 ▼
                    ┌─────────────────────────┐
                    │       核心层 core/       │
                    │   config / error / db   │
                    └─────────────────────────┘
```

---

## 五、配置文件设计

### 5.1 新增 `config/ai.toml`

```toml
[ai]
default_model = "openai/deepseek-chat"
fallback_models = ["openai/gemini-2.5-flash"]
temperature = 0.7
max_tokens = 4096

[ai.providers.deepseek]
api_key_env = "DEEPSEEK_API_KEY"
base_url = "https://api.deepseek.com/v1"
models = ["deepseek-chat", "deepseek-reasoner"]

[ai.providers.gemini]
api_key_env = "GEMINI_API_KEY"
models = ["gemini-2.5-flash", "gemini-2.5-pro"]

[ai.providers.openai]
api_key_env = "OPENAI_API_KEY"
base_url_env = "OPENAI_BASE_URL"
models = ["gpt-4o", "gpt-4o-mini"]

[ai.providers.anthropic]
api_key_env = "ANTHROPIC_API_KEY"
models = ["claude-3-5-sonnet", "claude-3-opus"]

[ai.providers.ollama]
base_url = "http://localhost:11434"
models = ["llama3", "qwen2"]
```

### 5.2 新增 `config/news.toml`

```toml
[news]
default_provider = "tavily"
max_age_days = 3
cache_ttl_seconds = 3600
strategy_profile = "short"  # short / detailed

[news.providers.tavily]
api_key_env = "TAVILY_API_KEY"
enabled = true
priority = 1

[news.providers.serpapi]
api_key_env = "SERPAPI_API_KEY"
enabled = true
priority = 2

[news.providers.bocha]
api_key_env = "BOCHA_API_KEY"
enabled = true
priority = 3

[news.providers.brave]
api_key_env = "BRAVE_API_KEY"
enabled = false
priority = 4

[news.providers.searxng]
base_url_env = "SEARXNG_BASE_URL"
enabled = false
priority = 5
```

### 5.3 扩展 `config/notification.toml`

```toml
[notification]
enabled_channels = ["telegram", "wechat"]
retry_count = 3
retry_delay_seconds = 5
quiet_hours_start = "23:00"
quiet_hours_end = "07:00"

[notification.channels.telegram]
enabled = true
bot_token_env = "TELEGRAM_BOT_TOKEN"
chat_id_env = "TELEGRAM_CHAT_ID"

[notification.channels.wechat]
enabled = true
webhook_url_env = "WECHAT_WEBHOOK_URL"
msg_type = "markdown"

[notification.channels.feishu]
enabled = false
webhook_url_env = "FEISHU_WEBHOOK_URL"

[notification.channels.discord]
enabled = false
webhook_url_env = "DISCORD_WEBHOOK_URL"

[notification.channels.slack]
enabled = false
bot_token_env = "SLACK_BOT_TOKEN"
channel_id_env = "SLACK_CHANNEL_ID"

[notification.channels.dingtalk]
enabled = false
webhook_url_env = "DINGTALK_WEBHOOK_URL"

[notification.channels.email]
enabled = false
sender_env = "EMAIL_SENDER"
password_env = "EMAIL_PASSWORD"
receivers_env = "EMAIL_RECEIVERS"
smtp_host = "smtp.gmail.com"
smtp_port = 587

[notification.channels.pushplus]
enabled = false
token_env = "PUSHPLUS_TOKEN"
```

---

## 六、环境变量清单

### 6.1 AI/LLM 配置

```env
# 主模型配置
LITELLM_MODEL=openai/deepseek-chat
LITELLM_FALLBACK_MODELS=openai/gemini-2.5-flash,anthropic/claude-3-5-sonnet

# API Keys
GEMINI_API_KEY=
DEEPSEEK_API_KEY=
ANTHROPIC_API_KEY=
OPENAI_API_KEY=
OPENAI_BASE_URL=

# Ollama 本地模型
OLLAMA_API_BASE=http://localhost:11434

# 多渠道模式（高级）
LLM_CHANNELS=deepseek,gemini
LLM_DEEPSEEK_API_KEY=
LLM_DEEPSEEK_BASE_URL=https://api.deepseek.com/v1
LLM_DEEPSEEK_MODELS=deepseek-chat
```

### 6.2 新闻搜索配置

```env
# 新闻搜索 API Keys（支持多Key轮询）
TAVILY_API_KEYS=
SERPAPI_API_KEYS=
BOCHA_API_KEYS=
BRAVE_API_KEYS=
MINIMAX_API_KEYS=
SEARXNG_BASE_URLS=

# 新闻策略
NEWS_MAX_AGE_DAYS=3
NEWS_STRATEGY_PROFILE=short
```

### 6.3 通知渠道配置

```env
# 企业微信
WECHAT_WEBHOOK_URL=

# 飞书
FEISHU_WEBHOOK_URL=

# Telegram
TELEGRAM_BOT_TOKEN=
TELEGRAM_CHAT_ID=

# Discord
DISCORD_WEBHOOK_URL=

# Slack
SLACK_BOT_TOKEN=
SLACK_CHANNEL_ID=

# 邮件
EMAIL_SENDER=
EMAIL_PASSWORD=
EMAIL_RECEIVERS=

# 钉钉
DINGTALK_WEBHOOK_URL=

# PushPlus
PUSHPLUS_TOKEN=
```

### 6.4 舆情配置（美股）

```env
SOCIAL_SENTIMENT_API_KEY=
SOCIAL_SENTIMENT_API_URL=https://api.adanos.org
```

---

## 七、新增 Rust 依赖

```toml
# Cargo.toml 新增依赖

[dependencies]
# 模板引擎（用于Prompt模板和消息渲染）
tera = "1.19"

# 流式处理（LLM流式输出）
futures = "0.3"  # 已有 futures-util，需升级

# 图片处理（可选，用于MD转图片）
image = { version = "0.25", optional = true }

[features]
default = ["postgresql", "tdengine-rest"]
# 新增特性
ai-vision = ["image"]  # AI视觉能力（图片识别）
news-full = []         # 完整新闻源支持
notification-full = ["image"]  # 完整通知支持（含MD转图片）
```

**reqwest 升级**:
```toml
# 现有
reqwest = { version = "0.11", features = ["json", "cookies"] }

# 升级为
reqwest = { version = "0.12", features = ["json", "cookies", "rustls-tls", "stream"] }
```

---

## 八、实施时间表

| 阶段 | 内容 | 时长 | 依赖 | 状态 |
|------|------|------|------|------|
| **Phase 1** | 通知模块扩展 | 1周 | 无 | 📋 待实施 |
| **Phase 2** | AI 决策模块 | 2周 | Phase 1 | 📋 待实施 |
| **Phase 3** | 新闻搜索模块 | 1.5周 | 无 | 📋 待实施 |
| **Phase 4** | 基本面+舆情 | 1周 | Phase 3 | 📋 待实施 |
| **Phase 5** | 智能导入（可选） | 1周 | Phase 2 | 📋 可选 |
| **总计** | | **6.5周** | | |

### 详细进度

#### Phase 1: 通知模块 (1周)
```
Day 1-2: NotificationProvider trait + 企业微信/飞书
Day 3-4: Telegram/Discord/Slack
Day 5: 钉钉/PushPlus + 测试
Day 6-7: CLI集成 + 文档
```

#### Phase 2: AI 决策模块 (2周)
```
Week 1:
  Day 1-2: LLM Adapter 核心（OpenAI兼容协议）
  Day 3-4: DeepSeek/Gemini/Anthropic 实现
  Day 5: Ollama 本地模型支持

Week 2:
  Day 1-2: Prompt模板系统
  Day 3-4: 决策引擎 + 对话管理
  Day 5: 策略技能注册
  Day 6-7: CLI集成 + 测试
```

#### Phase 3: 新闻搜索 (1.5周)
```
Day 1-2: NewsProvider trait + Tavily
Day 3-4: SerpAPI + 博查
Day 5-6: 聚合器 + 缓存 + Fallback
Day 7-8: CLI集成 + 测试
Day 9-10: 文档
```

#### Phase 4: 基本面 + 舆情 (1周)
```
Day 1-2: FundamentalProvider trait + AkShare适配
Day 3-4: 估值/财报/龙虎榜数据
Day 5: 舆情API集成（美股）
Day 6-7: CLI集成 + 测试
```

---

## 九、风险与缓解措施

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| LLM API 延迟 | 用户体验差 | 异步处理 + 流式输出 + 进度提示 |
| 通知渠道API变更 | 功能失效 | 抽象层 + 多渠道fallback + 定期测试 |
| 新闻API限流 | 数据缺失 | 缓存 + 多源轮询 + 降级策略 |
| LLM成本 | 运营成本 | 配置多档模型 + 本地Ollama备选 |
| 上下文长度 | 内存/质量 | 限制历史长度 + 摘要压缩 |

---

## 十、参考资源

### daily_stock_analysis 源码参考
- `src/analyzer.py` - AI分析核心
- `src/agent/llm_adapter.py` - LLM适配器
- `src/notification.py` - 通知模块
- `src/search_service.py` - 新闻搜索
- `src/services/social_sentiment_service.py` - 舆情分析
- `data_provider/fundamental_adapter.py` - 基本面数据

### API 文档
- [OpenAI API](https://platform.openai.com/docs)
- [DeepSeek API](https://platform.deepseek.com/docs)
- [Gemini API](https://ai.google.dev/docs)
- [Tavily API](https://docs.tavily.com)
- [Adanos Sentiment API](https://api.adanos.org)

---

## 十一、总结

### 必须迁移 (P0)
1. **AI 决策模块** - 核心差异化能力
2. **新闻搜索** - 信息获取基础
3. **多渠道通知** - 用户触达关键

### 建议迁移 (P1)
4. **舆情分析** - 增强美股分析
5. **Agent 对话增强** - 提升交互体验
6. **基本面数据** - 完善分析维度

### 可选迁移 (P2)
7. **智能导入** - 便利性功能

### 不迁移
- Web UI（quantix-rust 定位为 CLI）
- 桌面端（非目标平台）
- 已有功能（技术指标、回测、风控等）
