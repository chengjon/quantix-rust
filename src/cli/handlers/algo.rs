#![allow(clippy::too_many_arguments)]

//! Algorithmic Trading CLI Handler
//!
//! 处理 TWAP/VWAP 算法交易命令

use crate::core::{QuantixError, Result};
use crate::execution::algo::{
    AlgoContext, AlgoParams, AlgoStatus, AlgoType, AlgorithmExecutor, SlicePlan, TwapExecutor,
    VwapExecutor,
};
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::AlgoCommands;

// 全局算法管理器 (简化版，单进程内存存储)
lazy_static::lazy_static! {
    static ref ALGO_STORE: Arc<RwLock<AlgoStore>> = Arc::new(RwLock::new(AlgoStore::new()));
}

/// 算法存储
struct AlgoStore {
    /// TWAP 执行器
    twap: TwapExecutor,
    /// VWAP 执行器
    vwap: VwapExecutor,
    /// 算法上下文 (用于状态查询)
    contexts: HashMap<String, AlgoContext>,
    /// 算法类型映射
    algo_types: HashMap<String, AlgoType>,
}

impl AlgoStore {
    fn new() -> Self {
        Self {
            twap: TwapExecutor::new(),
            vwap: VwapExecutor::new(),
            contexts: HashMap::new(),
            algo_types: HashMap::new(),
        }
    }
}

pub async fn run_algo_command(cmd: AlgoCommands) -> Result<()> {
    match cmd {
        AlgoCommands::Create {
            code,
            side,
            quantity,
            algo_type,
            duration,
            price,
            slices,
            interval,
            no_randomize,
        } => {
            run_algo_create(
                code,
                side,
                quantity,
                algo_type,
                duration,
                price,
                slices,
                interval,
                no_randomize,
            )
            .await
        }
        AlgoCommands::Start { algo_id } => run_algo_start(algo_id).await,
        AlgoCommands::Pause { algo_id } => run_algo_pause(algo_id).await,
        AlgoCommands::Resume { algo_id } => run_algo_resume(algo_id).await,
        AlgoCommands::Cancel { algo_id } => run_algo_cancel(algo_id).await,
        AlgoCommands::Status { algo_id } => run_algo_status(algo_id).await,
        AlgoCommands::List => run_algo_list().await,
        AlgoCommands::Plan {
            code,
            side,
            quantity,
            algo_type,
            duration,
            slices,
            interval,
            output,
        } => {
            run_algo_plan(
                code, side, quantity, algo_type, duration, slices, interval, output,
            )
            .await
        }
    }
}

async fn run_algo_create(
    code: String,
    side: String,
    quantity: i64,
    algo_type: String,
    duration: u32,
    price: Option<f64>,
    slices: Option<u32>,
    interval: Option<u64>,
    no_randomize: bool,
) -> Result<()> {
    // 解析算法类型
    let algo_type_enum = match algo_type.to_lowercase().as_str() {
        "twap" => AlgoType::TWAP,
        "vwap" => AlgoType::VWAP,
        _ => {
            return Err(QuantixError::Unsupported(format!(
                "不支持的算法类型: {}",
                algo_type
            )));
        }
    };

    // 验证方向
    if side != "buy" && side != "sell" {
        return Err(QuantixError::Other(format!(
            "方向必须是 buy 或 sell: {}",
            side
        )));
    }

    // 构建参数
    let now = Utc::now();
    let _end_time = now + Duration::minutes(duration as i64);

    let mut params = match algo_type_enum {
        AlgoType::TWAP => AlgoParams::twap(code.clone(), side.clone(), quantity, duration),
        AlgoType::VWAP => AlgoParams::vwap(code.clone(), side.clone(), quantity, duration),
        _ => return Err(QuantixError::Other("不支持的算法类型".to_string())),
    };

    // 设置可选参数
    if let Some(p) = price {
        params = params.with_price_limit(
            Decimal::try_from(p).map_err(|e| QuantixError::Other(e.to_string()))?,
        );
    }
    if let Some(s) = slices {
        params = params.with_slice_count(s);
    }
    if let Some(i) = interval {
        params = params.with_interval(i);
    }
    if no_randomize {
        params = params.no_randomize();
    }

    // 初始化算法
    let mut store = ALGO_STORE.write().await;
    let algo_id = match algo_type_enum {
        AlgoType::TWAP => store.twap.initialize(params.clone()).await?,
        AlgoType::VWAP => store.vwap.initialize(params.clone()).await?,
        _ => return Err(QuantixError::Other("不支持的算法类型".to_string())),
    };

    // 创建上下文用于状态跟踪
    let context = AlgoContext::new(params.clone(), algo_id.clone());
    store.contexts.insert(algo_id.clone(), context);
    store.algo_types.insert(algo_id.clone(), algo_type_enum);

    println!("✓ 算法任务已创建");
    println!();
    println!("  算法 ID: {}", algo_id);
    println!("  类型:    {}", algo_type.to_uppercase());
    println!("  股票:    {}", code);
    println!("  方向:    {}", if side == "buy" { "买入" } else { "卖出" });
    println!("  数量:    {} 股", quantity);
    println!("  时长:    {} 分钟", duration);
    if let Some(p) = price {
        println!("  限价:    {:.2}", p);
    }
    println!();
    println!("启动算法: quantix algo start --algo-id {}", algo_id);

    Ok(())
}

async fn run_algo_start(algo_id: String) -> Result<()> {
    let mut store = ALGO_STORE.write().await;

    let algo_type = store
        .algo_types
        .get(&algo_id)
        .ok_or_else(|| QuantixError::Other(format!("算法不存在: {}", algo_id)))?;

    match *algo_type {
        AlgoType::TWAP => store.twap.start(&algo_id).await?,
        AlgoType::VWAP => store.vwap.start(&algo_id).await?,
        _ => return Err(QuantixError::Other("不支持的算法类型".to_string())),
    }

    // 更新上下文状态
    if let Some(ctx) = store.contexts.get_mut(&algo_id) {
        ctx.state.status = AlgoStatus::Running;
        ctx.state.started_at = Some(Utc::now());
    }

    println!("✓ 算法已启动: {}", algo_id);
    println!();
    println!("查看状态: quantix algo status --algo-id {}", algo_id);
    println!("暂停算法: quantix algo pause --algo-id {}", algo_id);
    println!("取消算法: quantix algo cancel --algo-id {}", algo_id);

    Ok(())
}

async fn run_algo_pause(algo_id: String) -> Result<()> {
    let mut store = ALGO_STORE.write().await;

    let algo_type = store
        .algo_types
        .get(&algo_id)
        .ok_or_else(|| QuantixError::Other(format!("算法不存在: {}", algo_id)))?;

    match *algo_type {
        AlgoType::TWAP => store.twap.pause(&algo_id).await?,
        AlgoType::VWAP => store.vwap.pause(&algo_id).await?,
        _ => return Err(QuantixError::Other("不支持的算法类型".to_string())),
    }

    if let Some(ctx) = store.contexts.get_mut(&algo_id) {
        ctx.state.status = AlgoStatus::Paused;
    }

    println!("✓ 算法已暂停: {}", algo_id);
    println!("恢复算法: quantix algo resume --algo-id {}", algo_id);

    Ok(())
}

async fn run_algo_resume(algo_id: String) -> Result<()> {
    let mut store = ALGO_STORE.write().await;

    let algo_type = store
        .algo_types
        .get(&algo_id)
        .ok_or_else(|| QuantixError::Other(format!("算法不存在: {}", algo_id)))?;

    match *algo_type {
        AlgoType::TWAP => store.twap.resume(&algo_id).await?,
        AlgoType::VWAP => store.vwap.resume(&algo_id).await?,
        _ => return Err(QuantixError::Other("不支持的算法类型".to_string())),
    }

    if let Some(ctx) = store.contexts.get_mut(&algo_id) {
        ctx.state.status = AlgoStatus::Running;
    }

    println!("✓ 算法已恢复: {}", algo_id);

    Ok(())
}

async fn run_algo_cancel(algo_id: String) -> Result<()> {
    let mut store = ALGO_STORE.write().await;

    let algo_type = store
        .algo_types
        .get(&algo_id)
        .ok_or_else(|| QuantixError::Other(format!("算法不存在: {}", algo_id)))?;

    match *algo_type {
        AlgoType::TWAP => store.twap.cancel(&algo_id).await?,
        AlgoType::VWAP => store.vwap.cancel(&algo_id).await?,
        _ => return Err(QuantixError::Other("不支持的算法类型".to_string())),
    }

    if let Some(ctx) = store.contexts.get_mut(&algo_id) {
        ctx.state.status = AlgoStatus::Cancelled;
        ctx.state.completed_at = Some(Utc::now());
    }

    println!("✓ 算法已取消: {}", algo_id);

    Ok(())
}

async fn run_algo_status(algo_id: String) -> Result<()> {
    let store = ALGO_STORE.read().await;

    let context = store
        .contexts
        .get(&algo_id)
        .ok_or_else(|| QuantixError::Other(format!("算法不存在: {}", algo_id)))?;

    let algo_type = store
        .algo_types
        .get(&algo_id)
        .map(|t| t.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let state = &context.state;

    println!("算法状态");
    println!("─────────────────────────────────────");
    println!("  算法 ID:     {}", state.algo_id);
    println!("  类型:        {}", algo_type);
    println!("  股票:        {}", state.symbol);
    println!("  方向:        {}", state.side);
    println!("  状态:        {}", format_status(state.status));
    println!();
    println!("执行进度");
    println!("─────────────────────────────────────");
    println!("  目标数量:    {} 股", state.target_quantity);
    println!("  已成交:      {} 股", state.filled_quantity);
    println!("  剩余:        {} 股", state.remaining_quantity());
    println!("  完成度:      {:.2}%", state.completion_percent());
    println!("  订单数:      {}", state.order_count);
    println!("  成交数:      {}", state.fill_count);
    println!();
    println!("时间信息");
    println!("─────────────────────────────────────");
    println!(
        "  创建时间:    {}",
        state.created_at.format("%Y-%m-%d %H:%M:%S")
    );
    if let Some(started) = state.started_at {
        println!("  开始时间:    {}", started.format("%Y-%m-%d %H:%M:%S"));
    }
    if let Some(completed) = state.completed_at {
        println!("  完成时间:    {}", completed.format("%Y-%m-%d %H:%M:%S"));
    }

    if state.filled_quantity > 0 {
        println!();
        println!("成交统计");
        println!("─────────────────────────────────────");
        println!("  平均成交价:  {:.4}", state.avg_fill_price);
        println!("  成交金额:    {:.2}", state.total_amount);
    }

    Ok(())
}

fn format_status(status: AlgoStatus) -> String {
    match status {
        AlgoStatus::Pending => "⏳ 等待启动".to_string(),
        AlgoStatus::Running => "🟢 运行中".to_string(),
        AlgoStatus::Paused => "⏸️ 已暂停".to_string(),
        AlgoStatus::Completed => "✅ 已完成".to_string(),
        AlgoStatus::Cancelled => "❌ 已取消".to_string(),
        AlgoStatus::Error => "⚠️ 错误".to_string(),
    }
}

async fn run_algo_list() -> Result<()> {
    let store = ALGO_STORE.read().await;

    if store.contexts.is_empty() {
        println!("当前没有算法任务");
        println!();
        println!(
            "创建算法: quantix algo create --code 600519.SH --side buy --quantity 10000 --algo-type twap"
        );
        return Ok(());
    }

    println!(
        "{:<22} {:<6} {:<12} {:<6} {:<12} {:<8}",
        "算法 ID", "类型", "股票", "方向", "状态", "进度"
    );
    println!("{}", "-".repeat(76));

    for (id, ctx) in store.contexts.iter() {
        let algo_type = store
            .algo_types
            .get(id)
            .map(|t| t.to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let short_id = if id.len() > 20 {
            &id[..20]
        } else {
            id.as_str()
        };

        println!(
            "{:<22} {:<6} {:<12} {:<6} {:<12} {:.1}%",
            short_id,
            algo_type,
            ctx.state.symbol,
            ctx.state.side,
            format_status(ctx.state.status)
                .replace("⏳ ", "")
                .replace("🟢 ", "")
                .replace("⏸️ ", "")
                .replace("✅ ", "")
                .replace("❌ ", "")
                .replace("⚠️ ", ""),
            ctx.state.completion_percent()
        );
    }

    Ok(())
}

async fn run_algo_plan(
    code: String,
    side: String,
    quantity: i64,
    algo_type: String,
    duration: u32,
    slices: Option<u32>,
    interval: Option<u64>,
    output: String,
) -> Result<()> {
    // 解析算法类型
    let algo_type_enum = match algo_type.to_lowercase().as_str() {
        "twap" => AlgoType::TWAP,
        "vwap" => AlgoType::VWAP,
        _ => {
            return Err(QuantixError::Other(format!(
                "不支持的算法类型: {}",
                algo_type
            )));
        }
    };

    // 构建参数
    let mut params = match algo_type_enum {
        AlgoType::TWAP => AlgoParams::twap(code.clone(), side.clone(), quantity, duration),
        AlgoType::VWAP => AlgoParams::vwap(code.clone(), side.clone(), quantity, duration),
        _ => return Err(QuantixError::Other("不支持的算法类型".to_string())),
    };

    if let Some(s) = slices {
        params = params.with_slice_count(s);
    }
    if let Some(i) = interval {
        params = params.with_interval(i);
    }
    params = params.no_randomize(); // 预览时不随机化
    params.validate().map_err(QuantixError::Other)?;

    // 生成切片计划
    let plan = match algo_type_enum {
        AlgoType::TWAP => TwapExecutor::new().get_slice_plan(&params)?,
        AlgoType::VWAP => VwapExecutor::new().get_slice_plan(&params)?,
        _ => return Err(QuantixError::Other("不支持的算法类型".to_string())),
    };

    match output.to_lowercase().as_str() {
        "json" => output_plan_json(&plan, &code, &side, &algo_type)?,
        "table" => output_plan_table(&plan, &code, &side, &algo_type)?,
        _ => {
            return Err(QuantixError::Other(format!(
                "不支持的输出格式: {output}，仅支持 table 或 json"
            )));
        }
    }

    Ok(())
}

fn output_plan_table(plan: &SlicePlan, code: &str, side: &str, algo_type: &str) -> Result<()> {
    println!("切片计划预览");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    println!("  算法类型:    {}", algo_type.to_uppercase());
    println!("  股票代码:    {}", code);
    println!(
        "  买卖方向:    {}",
        if side == "buy" { "买入" } else { "卖出" }
    );
    println!("  总数量:      {} 股", plan.total_quantity);
    println!("  切片数量:    {}", plan.slices.len());
    println!(
        "  开始时间:    {}",
        plan.start_time.format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "  结束时间:    {}",
        plan.end_time.format("%Y-%m-%d %H:%M:%S")
    );
    println!();

    println!(
        "{:<6} {:<12} {:<10} {:<10} {:<8}",
        "#", "计划时间", "数量", "累计", "权重"
    );
    println!("{}", "-".repeat(56));

    let mut cumulative: i64 = 0;
    for s in &plan.slices {
        cumulative += s.quantity;
        let weight = s
            .volume_weight
            .map(|w| format!("{:.1}", w))
            .unwrap_or_else(|| "-".to_string());

        println!(
            "{:<6} {:<12} {:<10} {:<10} {:<8}",
            s.index + 1,
            s.scheduled_time.format("%H:%M:%S"),
            s.quantity,
            cumulative,
            weight
        );
    }

    Ok(())
}

fn output_plan_json(plan: &SlicePlan, code: &str, side: &str, algo_type: &str) -> Result<()> {
    let slices: Vec<serde_json::Value> = plan
        .slices
        .iter()
        .map(|s| {
            serde_json::json!({
                "index": s.index,
                "scheduled_time": s.scheduled_time.to_rfc3339(),
                "quantity": s.quantity,
                "price": s.price,
                "volume_weight": s.volume_weight,
            })
        })
        .collect();

    let json = serde_json::json!({
        "algo_type": algo_type.to_uppercase(),
        "symbol": code,
        "side": side,
        "total_quantity": plan.total_quantity,
        "slice_count": plan.slices.len(),
        "start_time": plan.start_time.to_rfc3339(),
        "end_time": plan.end_time.to_rfc3339(),
        "slices": slices,
    });

    println!("{}", serde_json::to_string_pretty(&json)?);
    Ok(())
}
