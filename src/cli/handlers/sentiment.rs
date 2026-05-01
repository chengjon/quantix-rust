use super::fundamental::truncate_str;
use super::*;

// ============================================================
// 舆情分析命令
// ============================================================

/// 处理舆情命令
pub async fn run_sentiment_command(cmd: SentimentCommands) -> Result<()> {
    match cmd {
        SentimentCommands::Show { code } => run_sentiment_show(&code).await,
        SentimentCommands::History { code, days } => run_sentiment_history(&code, days).await,
        SentimentCommands::Mentions { code, max } => run_sentiment_mentions(&code, max).await,
    }
}

async fn run_sentiment_show(code: &str) -> Result<()> {
    println!("📊 舆情分析");
    println!("   代码: {}", code);
    println!();

    let aggregator = SentimentAggregator::new(vec![]);
    let data = aggregator.get_sentiment(code).await?;

    let level = data.sentiment_level;
    println!("{} 情绪等级: {}", level.emoji(), level.label());
    println!("📈 情绪指数: {:.2}", data.overall_score);
    println!(
        "📊 趋势方向: {}",
        match data.trend {
            SentimentTrend::RisingFast => "↑ 快速上升",
            SentimentTrend::Rising => "↑ 上升",
            SentimentTrend::Stable => "→ 平稳",
            SentimentTrend::Falling => "↓ 下降",
            SentimentTrend::FallingFast => "↓ 快速下降",
        }
    );
    println!();

    if data.source_scores.is_empty() {
        println!("💡 暂无舆情数据源，请配置 SentimentProvider");
        println!("   可用提供商: Adanos, EastMoney Guba, Sina Finance");
    } else {
        println!("📋 各来源得分:");
        println!("{:<15} {:<10} {:<10}", "来源", "得分", "样本数");
        println!("{}", "-".repeat(35));
        for score in &data.source_scores {
            println!(
                "{:<15} {:<10.2} {:<10}",
                score.source, score.score, score.sample_count
            );
        }
    }

    Ok(())
}

async fn run_sentiment_history(code: &str, days: u32) -> Result<()> {
    println!("📊 舆情历史趋势");
    println!("   代码: {}", code);
    println!("   天数: {}", days);
    println!();

    let aggregator = SentimentAggregator::new(vec![]);
    let history = aggregator.get_history(code, days).await?;

    if history.is_empty() {
        println!("📅 历史数据: (暂无数据)");
        println!();
        println!("💡 历史舆情需要配置 SentimentProvider 后获取");
    } else {
        println!("📅 历史数据:");
        println!("{:<20} {:<10} {:<10}", "时间", "得分", "样本数");
        println!("{}", "-".repeat(40));
        for point in &history {
            let ts = point.timestamp.format("%Y-%m-%d %H:%M");
            println!(
                "{:<20} {:<10.2} {:<10}",
                ts, point.score, point.sample_count
            );
        }
    }

    Ok(())
}

async fn run_sentiment_mentions(code: &str, max: usize) -> Result<()> {
    println!("💬 社交媒体提及");
    println!("   代码: {}", code);
    println!("   最大数量: {}", max);
    println!();

    let aggregator = SentimentAggregator::new(vec![]);
    let mentions = aggregator.get_mentions(code, max).await?;

    if mentions.is_empty() {
        println!("📱 最近提及: (暂无数据)");
        println!();
        println!("💡 社交媒体数据需要配置 SentimentProvider 后获取");
    } else {
        println!("📱 最近提及:");
        for m in &mentions {
            let time = m
                .published_at
                .map(|t| t.format("%m-%d %H:%M").to_string())
                .unwrap_or("-".to_string());
            let sentiment_str = m
                .sentiment
                .map(|s| format!("{:.1}", s))
                .unwrap_or("-".to_string());
            println!(
                "   {} [{}] {} ( sentiment: {} )",
                time,
                m.platform,
                truncate_str(&m.content, 40),
                sentiment_str
            );
            if let Some(author) = &m.author {
                println!("      作者: {}", author);
            }
        }
    }

    Ok(())
}
