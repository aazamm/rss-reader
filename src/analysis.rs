use crate::feed::Article;
use crate::stock::DailyPrice;
use crate::storage::Investment;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct ArticleMention {
    pub article: Article,
    pub ticker: String,
    pub sentiment: Sentiment,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sentiment {
    Positive,
    Negative,
    Neutral,
}

impl std::fmt::Display for Sentiment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sentiment::Positive => write!(f, "Positive"),
            Sentiment::Negative => write!(f, "Negative"),
            Sentiment::Neutral => write!(f, "Neutral"),
        }
    }
}

#[derive(Debug)]
pub struct Correlation {
    pub date: String,
    pub article_title: String,
    pub sentiment: Sentiment,
    pub price: Option<f64>,
    pub price_change: Option<f64>,
}

const POSITIVE_WORDS: &[&str] = &[
    "gain", "gains", "surge", "surges", "surging", "rise", "rises", "rising",
    "profit", "profits", "beat", "beats", "bullish", "growth", "growing",
    "rally", "rallies", "soar", "soars", "soaring", "jump", "jumps",
    "record", "high", "upgrade", "upgrades", "strong", "success", "win",
];

const NEGATIVE_WORDS: &[&str] = &[
    "fall", "falls", "falling", "drop", "drops", "dropping", "loss", "losses",
    "miss", "misses", "bearish", "decline", "declines", "declining", "crash",
    "crashes", "plunge", "plunges", "plunging", "sink", "sinks", "sinking",
    "low", "downgrade", "downgrades", "weak", "fail", "fails", "cut", "cuts",
];

pub fn find_mentions(articles: &[Article], investments: &[Investment]) -> Vec<ArticleMention> {
    let mut mentions = Vec::new();

    for article in articles {
        let text = format!(
            "{} {}",
            article.title,
            article.content.as_deref().unwrap_or("")
        )
        .to_uppercase();

        for investment in investments {
            let ticker_pattern = format!(r"\b{}\b", regex::escape(&investment.ticker));
            let ticker_re = Regex::new(&ticker_pattern).unwrap();

            let mut found = ticker_re.is_match(&text);

            if !found {
                if let Some(ref name) = investment.name {
                    let name_pattern = format!(r"\b{}\b", regex::escape(&name.to_uppercase()));
                    if let Ok(name_re) = Regex::new(&name_pattern) {
                        found = name_re.is_match(&text);
                    }
                }
            }

            if found {
                let full_text = format!(
                    "{} {}",
                    article.title,
                    article.content.as_deref().unwrap_or("")
                );
                let sentiment = analyze_sentiment(&full_text);

                mentions.push(ArticleMention {
                    article: article.clone(),
                    ticker: investment.ticker.clone(),
                    sentiment,
                });
            }
        }
    }

    mentions
}

pub fn analyze_sentiment(text: &str) -> Sentiment {
    let lower = text.to_lowercase();

    let positive_count = POSITIVE_WORDS
        .iter()
        .filter(|&&word| {
            let pattern = format!(r"\b{}\b", word);
            Regex::new(&pattern)
                .map(|re| re.is_match(&lower))
                .unwrap_or(false)
        })
        .count();

    let negative_count = NEGATIVE_WORDS
        .iter()
        .filter(|&&word| {
            let pattern = format!(r"\b{}\b", word);
            Regex::new(&pattern)
                .map(|re| re.is_match(&lower))
                .unwrap_or(false)
        })
        .count();

    if positive_count > negative_count {
        Sentiment::Positive
    } else if negative_count > positive_count {
        Sentiment::Negative
    } else {
        Sentiment::Neutral
    }
}

pub fn correlate(
    mentions: &[ArticleMention],
    prices: &[DailyPrice],
) -> Vec<Correlation> {
    let mut correlations = Vec::new();

    for mention in mentions {
        let article_date = mention
            .article
            .published
            .as_deref()
            .and_then(|d| d.split_whitespace().next())
            .unwrap_or("");

        let price_entry = prices.iter().find(|p| p.date == article_date);

        let price_change = if let Some(entry) = price_entry {
            let idx = prices.iter().position(|p| p.date == entry.date).unwrap_or(0);
            if idx > 0 {
                let prev = prices[idx - 1].close;
                Some(((entry.close - prev) / prev) * 100.0)
            } else {
                None
            }
        } else {
            None
        };

        correlations.push(Correlation {
            date: article_date.to_string(),
            article_title: mention.article.title.clone(),
            sentiment: mention.sentiment,
            price: price_entry.map(|p| p.close),
            price_change,
        });
    }

    correlations
}
