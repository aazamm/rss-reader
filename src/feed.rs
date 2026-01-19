use feed_rs::parser;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct Article {
    pub title: String,
    pub link: Option<String>,
    pub published: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug)]
pub struct FeedResult {
    pub title: String,
    pub articles: Vec<Article>,
}

pub async fn fetch_feed(url: &str) -> Result<FeedResult, Box<dyn Error>> {
    let response = reqwest::get(url).await?;
    let bytes = response.bytes().await?;
    let feed = parser::parse(&bytes[..])?;

    let title = feed
        .title
        .map(|t| t.content)
        .unwrap_or_else(|| "Untitled Feed".to_string());

    let articles = feed
        .entries
        .into_iter()
        .take(10)
        .map(|entry| {
            let title = entry
                .title
                .map(|t| t.content)
                .unwrap_or_else(|| "Untitled".to_string());
            let link = entry.links.first().map(|l| l.href.clone());
            let published = entry
                .published
                .or(entry.updated)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string());
            let content = entry
                .summary
                .map(|s| s.content)
                .or_else(|| entry.content.and_then(|c| c.body));
            Article {
                title,
                link,
                published,
                content,
            }
        })
        .collect();

    Ok(FeedResult { title, articles })
}
