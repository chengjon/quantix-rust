这份 Quantix CLI 命令手册内容非常丰富，覆盖了从数据导入、分析、策略、执行、风控到监控的完整链路。
你列出的 7 大方向（基础设施、数据、存储、分析回测、交易研究、风控模拟、工程治理）与当前实现的命令基本吻合，但当前命令树在组织与命名上存在一些可以优化的地方。

下面我以量化管理 / 交易专家的视角，给出系统性评估与具体优化建议。

一、整体分类合理性评估
原分类	合理性	问题与建议
01 基础设施与 CLI	✅ 合理	task 标记为实验性（P0 不支持），建议隐藏或移到 dev 子命令下
02 数据采集与数据源	⚠️ 部分缺失	当前没有显式的 collector、websocket 命令，只有 data query/export，采集能力偏弱
03 存储与同步	❌ 命令中几乎未体现	无 etl、sync、clickhouse 等命令，这部分功能可能未落地或放到了 risk sync 等侧边
04 分析、指标与回测	✅ 较完整	analyze indicators、analyze backtest、strategy run 等覆盖良好
05 交易辅助与研究	✅ 完整	watchlist、screener、market、fundamental、news 都有
06 风控、止损、模拟交易	✅ 很完整	risk、stop、trade、monitor 覆盖齐全
07 工程治理	❌ 缺失	无 git-nexus、worktree、docs、test 等工程命令
👉 结论：
当前实现偏向 “量化研究员 + 交易员” 日常使用的命令，缺失 数据采集、存储同步、工程治理三大部分，但这可能是项目阶段性取舍，不算错误。

二、具体命令的缺失与多余
✅ 建议增加的命令（按优先级）
方向	建议命令	理由
数据采集	data collect --from akshare --code ...	当前只能导出/查询，缺少主动拉取
数据采集	data subscribe --ws --code ...	实盘/高频需要 WebSocket
存储同步	storage migrate	数据库 schema 升级
存储同步	storage etl --from tdx --to pg	K 线聚合、清洗
回测增强	analyze benchmark --compare SPY,399300	对比基准
工程治理	dev doc	生成当前命令文档（自举）
工程治理	dev test --cmd account list	命令级冒烟测试
交易辅助	trade slippage --model fixed	模拟交易滑点配置
⚠️ 可能多余或重复的命令
命令	问题	建议
account split	订单拆分预览，与 algo plan 功能重叠	合并或重命名为 account preview-split
analyze candle-pattern	参数过多（11 个），使用复杂	拆分为 --from-csv / --from-code 子命令
task add	明确标注 Foundation P0 不支持	移到 task dev-add 或隐藏
execution bridge qmt-*	8 个 QMT 子命令，可合并为 qmt --action	改为 execution qmt account/asset/order
三、命令组织优化建议
1. 减少层级深度
当前最多 3 级（如 account group add-account），可接受，但部分命令名过长：

analyze screener preset-list → screener presets

monitor service-config set → monitor service config

2. 统一命名风格
列表：统一用 list（已基本统一 ✅）

查看详情：统一用 show（已有，但 account show / strategy show 不一致？ 检查发现都用了 show ✅）

创建：create / add / register 混用
建议：create 用于新建实体（group、alert），add 用于向集合添加（watchlist add），register 保留（账户注册）

3. 按领域聚合
当前已经是按领域聚合（account、risk、trade），很好。
但 execution 与 trade 边界模糊：

trade buy/sell 是模拟交易

execution bridge qmt-live 是实盘
建议：trade 只做模拟，execution 只做实盘桥接。

4. 隐藏实验性命令
task 系列命令应在 --help 中标注 (experimental)，甚至默认不显示，需要 --show-experimental 才出现。

四、具体重构方案（可落地）
建议的新命令树结构（部分）
bash
quantix data collect --from akshare --code 600519 --start 20240101
quantix data subscribe --ws wss://... --code 600519

quantix storage etl --source tdx --target pg --kline 1d
quantix storage migrate

quantix dev doc --format html
quantix dev test --scope account

# 合并 execution bridge
quantix execution qmt account
quantix execution qmt order --id xxx
quantix execution qmt cancel --order-id xxx

# 简化 analyze candle-pattern
quantix analyze pattern --code 600519 --type hammer
quantix analyze pattern --from-csv ./data.csv

# 移除 task.add（或标记 hidden）
quantix task list  # 只读，不暴露 add
五、总结
维度	评分	说明
功能完整性	7/10	缺采集、存储、工程治理
命名一致性	8/10	少量混用 add/create/register
层级合理性	8/10	3 级可接受，部分命令过长
可用性（用户体验）	7/10	部分命令参数过多，实验性命令未隐藏
最终建议：

短期：隐藏 task 系列，合并 execution bridge，简化 candle-pattern。

中期：增加 data collect、storage etl 两个核心命令。

长期：引入 dev test 与 dev doc 提升工程可维护性。


以下是一份优化后的 CLI 命令设计文档（Markdown），仅供参考：

Quantix CLI 优化设计文档
一、设计原则
原则	说明
单一职责	一个命令只做一件事
命名一致	list/show/create/remove 统一语义
层级适度	最多 3 级，2 级优先
实验隔离	不稳定命令默认隐藏或标注
模拟/实盘分离	trade 仅模拟，execution 仅实盘
二、保留的良好设计（无需改动）
命令路径	说明
account list/show/register/remove/update	账户 CRUD，命名清晰
account group create/list/show/remove	账户组管理
ai analyze/ask/decide	AI 能力
algo create/cancel/pause/resume/status	算法交易
analyze indicators/backtest	指标与回测
fundamental *	基本面全套
import from-*	智能导入
market *	市场分析
news *	新闻搜索
notify *	通知
risk rule/status/pnl/position	风控核心
stop set/list/remove	止盈止损
watchlist *	自选池
init/menu/status	基础设施
三、需要重构的命令
3.1 analyze candle-pattern → 拆分
当前问题：11 个选项，过于复杂

bash
# 旧命令（过于复杂）
quantix analyze candle-pattern --candle 10,11,12,13 --code 600519 --tdx-root /path --market sh --day-file ./day --start 20240101 --end 20241231 --type 1d --limit 20 --reference 100 --previous-close
优化后：

bash
# 从股票代码分析
quantix analyze pattern --code 600519 --period 1d --days 20

# 从 CSV 文件分析
quantix analyze pattern --from-csv ./kline.csv

# 从通达信 day 文件分析
quantix analyze pattern --from-tdx-day ./day --code 600519

# 显式 K 线序列
quantix analyze pattern --candle 10,11,12,13 --reference 100
3.2 execution bridge qmt-* → 合并
当前问题：8 个独立子命令，命名冗余

旧命令	新命令
execution bridge qmt-account	execution qmt account
execution bridge qmt-asset	execution qmt asset
execution bridge qmt-positions	execution qmt positions
execution bridge qmt-order --order-id	execution qmt order show --id
execution bridge qmt-cancel --order-id	execution qmt order cancel --id
execution bridge qmt-live --request-id	execution qmt order submit --request-id
execution bridge qmt-preview --request-id	execution qmt order preview --request-id
execution bridge qmt-query --order-id	execution qmt order status --id
execution bridge status	execution status
优化后结构：

bash
quantix execution qmt account          # 账户状态
quantix execution qmt asset            # 资产
quantix execution qmt positions        # 持仓
quantix execution qmt order list       # 订单列表
quantix execution qmt order show --id  # 订单详情
quantix execution qmt order cancel --id
quantix execution qmt order submit --request-id --yes
quantix execution qmt order preview --request-id
quantix execution status               # bridge 状态
3.3 account split 与 algo plan 重叠
当前问题：两个命令都做订单拆分预览，功能重叠

旧命令	建议
account split	删除，功能由 algo plan 覆盖
algo plan	保留，增加 --target-id 支持账户/组
3.4 monitor service-config → 简化
bash
# 旧
quantix monitor service-config set --quantix-bin /path
quantix monitor service-config show

# 新
quantix monitor service config --bin /path
quantix monitor service config show
3.5 strategy service-config 同理
bash
# 旧
quantix strategy service-config set --quantix-bin /path
quantix strategy service-config show

# 新
quantix strategy service config --bin /path
quantix strategy service config show
四、需要新增的命令
方向	命令	示例	优先级
数据采集	data collect --from akshare --code 600519	从 AkShare 拉取日线	P0
数据采集	data subscribe --ws --code 600519	WebSocket 订阅实时行情	P1
存储同步	storage migrate	数据库 schema 升级	P1
存储同步	storage etl --from tdx --to pg --kline 1d	K 线聚合清洗	P2
回测增强	analyze benchmark --compare SPY,399300	对比基准收益	P1
工程治理	dev doc --format html	生成命令文档	P1
工程治理	dev test --cmd account list	命令级冒烟测试	P2
模拟交易	trade slippage --model fixed --rate 0.001	滑点配置	P2
五、需要隐藏/废弃的命令
命令	处理方式	原因
task add	隐藏（--help 不显示）	Foundation P0 不支持
task start --daemon	隐藏	不支持后台
account split	废弃，提示用 algo plan	功能重复
analyze screener preset-list	改为 screener presets，旧命令保留别名	命名优化
六、命名统一对照表
动作	统一用词	示例
列出	list	account list
查看详情	show	account show --id
创建	create	account group create
添加（到集合）	add	watchlist add --code
移除	remove	watchlist remove --code
更新	update	account update --id
设置	set	stop set --code --loss
启用/禁用	enable / disable	risk rule enable --type
七、迁移对照表（供脚本适配）
旧命令	新命令	兼容期
analyze candle-pattern --code	analyze pattern --code	3 个月
execution bridge qmt-account	execution qmt account	3 个月
execution bridge qmt-asset	execution qmt asset	3 个月
account split	algo plan --target-id	立即废弃
monitor service-config set	monitor service config --bin	6 个月
analyze screener preset-list	screener presets	6 个月
八、优化后的命令统计
指标	优化前	优化后	变化
总命令节点	205	~180	-25（合并/废弃）
叶子命令	160	~155	-5
最大层级	3	3	不变
实验性命令暴露	是	否	隐藏 task add
九、实施建议
第一阶段（1-2 周）：隐藏 task 实验命令，废弃 account split

第二阶段（2-3 周）：重构 execution bridge 为 execution qmt

第三阶段（3-4 周）：拆分 analyze candle-pattern，新增 data collect

第四阶段（按需）：增加 dev doc、storage etl 等工程命令
