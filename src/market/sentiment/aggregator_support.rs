use chrono::Utc;

use super::provider::SentimentProvider;
use super::types::{SentimentScore, SocialMention};

pub(super) async fn collect_source_scores(
    providers: &[Box<dyn SentimentProvider>],
    code: &str,
) -> Vec<SentimentScore> {
    let mut source_scores = Vec::new();

    for provider in providers {
        if !provider.is_available() {
            continue;
        }

        if let Ok(score) = provider.get_score(code).await {
            source_scores.push(score);
        }
    }

    source_scores
}

pub(super) fn compute_overall_score(source_scores: &[SentimentScore]) -> f64 {
    let mut total_score = 0.0;
    let mut total_weight = 0.0;

    for score in source_scores {
        let weight = score.sample_count as f64;
        total_score += score.score * weight;
        total_weight += weight;
    }

    if total_weight > 0.0 {
        total_score / total_weight
    } else {
        0.0
    }
}

pub(super) async fn collect_recent_mentions(
    providers: &[Box<dyn SentimentProvider>],
    code: &str,
    per_provider_limit: usize,
    total_limit: usize,
) -> Vec<SocialMention> {
    let mut recent_mentions = Vec::new();
    for provider in providers {
        if provider.is_available() {
            if let Ok(mentions) = provider.get_mentions(code, per_provider_limit).await {
                recent_mentions.extend(mentions);
            }
        }
    }

    sort_and_limit_mentions(&mut recent_mentions, total_limit);
    recent_mentions
}

pub(super) fn available_provider_names(
    providers: &[Box<dyn SentimentProvider>],
) -> Vec<String> {
    providers
        .iter()
        .filter(|provider| provider.is_available())
        .map(|provider| provider.name().to_string())
        .collect()
}

fn sort_and_limit_mentions(mentions: &mut Vec<SocialMention>, total_limit: usize) {
    mentions.sort_by(|a, b| b.published_at.cmp(&a.published_at));
    mentions.truncate(total_limit);
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn sample_score(source: &str, score: f64, sample_count: usize) -> SentimentScore {
        SentimentScore {
            source: source.to_string(),
            score,
            sample_count,
            updated_at: Utc.with_ymd_and_hms(2026, 4, 1, 12, 0, 0).unwrap(),
        }
    }

    fn sample_mention(id: u32, published_at: Option<chrono::DateTime<Utc>>) -> SocialMention {
        SocialMention {
            platform: "x".to_string(),
            content: format!("mention-{id}"),
            author: None,
            published_at,
            engagement: None,
            sentiment: None,
            url: None,
        }
    }

    #[test]
    fn compute_overall_score_uses_sample_count_weighting() {
        let scores = vec![
            sample_score("a", 0.5, 10),
            sample_score("b", -0.25, 30),
            sample_score("c", 1.0, 0),
        ];

        assert!((compute_overall_score(&scores) - -0.0625).abs() < f64::EPSILON);
    }

    #[test]
    fn sort_and_limit_mentions_orders_desc_and_keeps_limit() {
        let mut mentions = vec![
            sample_mention(1, Some(Utc.with_ymd_and_hms(2026, 4, 1, 9, 0, 0).unwrap())),
            sample_mention(2, None),
            sample_mention(3, Some(Utc.with_ymd_and_hms(2026, 4, 1, 11, 0, 0).unwrap())),
        ];

        sort_and_limit_mentions(&mut mentions, 2);

        assert_eq!(mentions.len(), 2);
        assert_eq!(mentions[0].content, "mention-3");
        assert_eq!(mentions[1].content, "mention-1");
    }
}
