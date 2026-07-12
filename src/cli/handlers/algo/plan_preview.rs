//! 算法切片计划的生成与输出（table/json）。
//!
//! 从原生 `algo.rs` 拆出，承载 [`run_algo_plan`] 及其渲染函数。

use crate::core::{QuantixError, Result};
use crate::execution::algo::{
    AlgoParams, AlgoType, AlgorithmExecutor, SlicePlan, TwapExecutor, VwapExecutor,
};

#[allow(clippy::too_many_arguments)]
pub async fn run_algo_plan(
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
            return Err(QuantixError::Unsupported(format!(
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
    print!(
        "  买卖方向:    {}",
        if side == "buy" { "买入" } else { "卖出" }
    );
    println!();
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
