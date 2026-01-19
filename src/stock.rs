use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct StockQuote {
    pub ticker: String,
    pub price: f64,
    pub change: f64,
    pub change_percent: f64,
    pub date: String,
}

#[derive(Debug, Clone)]
pub struct DailyPrice {
    pub date: String,
    pub close: f64,
}

#[derive(Debug)]
pub struct PriceHistory {
    pub ticker: String,
    pub prices: Vec<DailyPrice>,
}

#[derive(Deserialize)]
struct YahooResponse {
    chart: ChartResult,
}

#[derive(Deserialize)]
struct ChartResult {
    result: Option<Vec<ChartData>>,
    error: Option<YahooError>,
}

#[derive(Deserialize)]
struct YahooError {
    description: String,
}

#[derive(Deserialize)]
struct ChartData {
    meta: MetaData,
    timestamp: Option<Vec<i64>>,
    indicators: Indicators,
}

#[derive(Deserialize)]
struct MetaData {
    #[serde(rename = "regularMarketPrice")]
    regular_market_price: Option<f64>,
    #[serde(rename = "previousClose")]
    previous_close: Option<f64>,
}

#[derive(Deserialize)]
struct Indicators {
    quote: Vec<QuoteData>,
}

#[derive(Deserialize)]
struct QuoteData {
    close: Option<Vec<Option<f64>>>,
}

pub async fn fetch_quote(ticker: &str) -> Result<StockQuote, Box<dyn Error>> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?range=1d&interval=1d",
        ticker.to_uppercase()
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await?;

    let data: YahooResponse = response.json().await?;

    if let Some(error) = data.chart.error {
        return Err(format!("Yahoo Finance error: {}", error.description).into());
    }

    let result = data
        .chart
        .result
        .and_then(|r| r.into_iter().next())
        .ok_or("No data returned for ticker")?;

    let price = result.meta.regular_market_price.unwrap_or(0.0);
    let previous_close = result.meta.previous_close.unwrap_or(price);
    let change = price - previous_close;
    let change_percent = if previous_close > 0.0 {
        (change / previous_close) * 100.0
    } else {
        0.0
    };

    let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    Ok(StockQuote {
        ticker: ticker.to_uppercase(),
        price,
        change,
        change_percent,
        date,
    })
}

pub async fn fetch_history(ticker: &str, days: u32) -> Result<PriceHistory, Box<dyn Error>> {
    let range = if days <= 5 {
        "5d"
    } else if days <= 30 {
        "1mo"
    } else if days <= 90 {
        "3mo"
    } else {
        "6mo"
    };

    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?range={}&interval=1d",
        ticker.to_uppercase(),
        range
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "Mozilla/5.0")
        .send()
        .await?;

    let data: YahooResponse = response.json().await?;

    if let Some(error) = data.chart.error {
        return Err(format!("Yahoo Finance error: {}", error.description).into());
    }

    let result = data
        .chart
        .result
        .and_then(|r| r.into_iter().next())
        .ok_or("No data returned for ticker")?;

    let timestamps = result.timestamp.unwrap_or_default();
    let closes = result
        .indicators
        .quote
        .first()
        .and_then(|q| q.close.as_ref())
        .cloned()
        .unwrap_or_default();

    let prices: Vec<DailyPrice> = timestamps
        .into_iter()
        .zip(closes.into_iter())
        .filter_map(|(ts, close)| {
            close.map(|c| {
                let date = chrono::DateTime::from_timestamp(ts, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                DailyPrice { date, close: c }
            })
        })
        .collect();

    Ok(PriceHistory {
        ticker: ticker.to_uppercase(),
        prices,
    })
}
