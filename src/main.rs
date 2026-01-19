mod analysis;
mod feed;
mod stock;
mod storage;

use clap::{Parser, Subcommand};
use storage::Config;

#[derive(Parser)]
#[command(name = "aaron_rss")]
#[command(about = "A simple command-line RSS reader")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new feed URL
    Add { url: String },
    /// Remove a feed URL
    Remove { url: String },
    /// List all subscribed feeds
    List,
    /// Fetch and display recent articles
    Fetch {
        /// Optional: fetch from a specific feed URL only
        url: Option<String>,
    },
    /// Manage tracked stock investments
    Stock {
        #[command(subcommand)]
        action: StockAction,
    },
    /// Scan feeds for mentions of tracked investments
    Scan,
    /// Analyze news and price correlation for a ticker
    Analyze {
        /// Stock ticker symbol
        ticker: String,
    },
}

#[derive(Subcommand)]
enum StockAction {
    /// Add a stock ticker to track
    Add {
        ticker: String,
        /// Optional company name for better matching
        #[arg(short, long)]
        name: Option<String>,
    },
    /// Remove a tracked ticker
    Remove { ticker: String },
    /// List all tracked investments
    List,
    /// Get current quote for a ticker
    Quote { ticker: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { url } => cmd_add(&url),
        Commands::Remove { url } => cmd_remove(&url),
        Commands::List => cmd_list(),
        Commands::Fetch { url } => cmd_fetch(url).await,
        Commands::Stock { action } => cmd_stock(action).await,
        Commands::Scan => cmd_scan().await,
        Commands::Analyze { ticker } => cmd_analyze(&ticker).await,
    }
}

fn cmd_add(url: &str) {
    let mut config = Config::load().unwrap_or_default();
    if config.add_feed(url) {
        if let Err(e) = config.save() {
            eprintln!("Error saving config: {}", e);
            return;
        }
        println!("Added feed: {}", url);
    } else {
        println!("Feed already exists: {}", url);
    }
}

fn cmd_remove(url: &str) {
    let mut config = Config::load().unwrap_or_default();
    if config.remove_feed(url) {
        if let Err(e) = config.save() {
            eprintln!("Error saving config: {}", e);
            return;
        }
        println!("Removed feed: {}", url);
    } else {
        println!("Feed not found: {}", url);
    }
}

fn cmd_list() {
    let config = Config::load().unwrap_or_default();
    if config.feeds.is_empty() {
        println!("No feeds subscribed. Use 'aaron_rss add <url>' to add a feed.");
        return;
    }
    println!("Subscribed feeds:");
    for (i, feed) in config.feeds.iter().enumerate() {
        println!("  {}. {}", i + 1, feed);
    }
}

async fn cmd_fetch(url: Option<String>) {
    let urls = match url {
        Some(u) => vec![u],
        None => {
            let config = Config::load().unwrap_or_default();
            if config.feeds.is_empty() {
                println!("No feeds subscribed. Use 'aaron_rss add <url>' to add a feed.");
                return;
            }
            config.feeds
        }
    };

    for feed_url in &urls {
        println!("\nFetching: {}", feed_url);
        match feed::fetch_feed(feed_url).await {
            Ok(result) => {
                println!("== {} ==", result.title);
                if result.articles.is_empty() {
                    println!("  No articles found.");
                } else {
                    for article in &result.articles {
                        let date = article
                            .published
                            .as_deref()
                            .unwrap_or("No date");
                        println!("\n  [{}]", date);
                        println!("  {}", article.title);
                        if let Some(link) = &article.link {
                            println!("  {}", link);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error fetching {}: {}", feed_url, e);
            }
        }
    }
}

async fn cmd_stock(action: StockAction) {
    match action {
        StockAction::Add { ticker, name } => {
            let mut config = Config::load().unwrap_or_default();
            if config.add_investment(&ticker, name.clone()) {
                if let Err(e) = config.save() {
                    eprintln!("Error saving config: {}", e);
                    return;
                }
                let display = match name {
                    Some(n) => format!("{} ({})", ticker.to_uppercase(), n),
                    None => ticker.to_uppercase(),
                };
                println!("Added investment: {}", display);
            } else {
                println!("Investment already tracked: {}", ticker.to_uppercase());
            }
        }
        StockAction::Remove { ticker } => {
            let mut config = Config::load().unwrap_or_default();
            if config.remove_investment(&ticker) {
                if let Err(e) = config.save() {
                    eprintln!("Error saving config: {}", e);
                    return;
                }
                println!("Removed investment: {}", ticker.to_uppercase());
            } else {
                println!("Investment not found: {}", ticker.to_uppercase());
            }
        }
        StockAction::List => {
            let config = Config::load().unwrap_or_default();
            if config.investments.is_empty() {
                println!("No investments tracked. Use 'aaron_rss stock add <ticker>' to add one.");
                return;
            }
            println!("Tracked investments:");
            for (i, inv) in config.investments.iter().enumerate() {
                let display = match &inv.name {
                    Some(n) => format!("{} ({})", inv.ticker, n),
                    None => inv.ticker.clone(),
                };
                println!("  {}. {}", i + 1, display);
            }
        }
        StockAction::Quote { ticker } => {
            println!("Fetching quote for {}...", ticker.to_uppercase());
            match stock::fetch_quote(&ticker).await {
                Ok(quote) => {
                    let change_sign = if quote.change >= 0.0 { "+" } else { "" };
                    println!(
                        "\n{}: ${:.2} ({}{:.2}, {}{:.2}%)",
                        quote.ticker,
                        quote.price,
                        change_sign,
                        quote.change,
                        change_sign,
                        quote.change_percent
                    );
                }
                Err(e) => {
                    eprintln!("Error fetching quote: {}", e);
                }
            }
        }
    }
}

async fn cmd_scan() {
    let config = Config::load().unwrap_or_default();

    if config.investments.is_empty() {
        println!("No investments tracked. Use 'aaron_rss stock add <ticker>' to add one.");
        return;
    }

    if config.feeds.is_empty() {
        println!("No feeds subscribed. Use 'aaron_rss add <url>' to add a feed.");
        return;
    }

    println!("Scanning feeds for investment mentions...\n");

    let mut all_articles = Vec::new();

    for feed_url in &config.feeds {
        match feed::fetch_feed(feed_url).await {
            Ok(result) => {
                all_articles.extend(result.articles);
            }
            Err(e) => {
                eprintln!("Error fetching {}: {}", feed_url, e);
            }
        }
    }

    let mentions = analysis::find_mentions(&all_articles, &config.investments);

    if mentions.is_empty() {
        println!("No mentions found for tracked investments.");
        return;
    }

    println!("Found {} mentions:\n", mentions.len());

    for mention in &mentions {
        let date = mention.article.published.as_deref().unwrap_or("No date");
        let sentiment_indicator = match mention.sentiment {
            analysis::Sentiment::Positive => "+",
            analysis::Sentiment::Negative => "-",
            analysis::Sentiment::Neutral => "~",
        };
        println!(
            "[{}] {} [{}] {}",
            mention.ticker, sentiment_indicator, date, mention.article.title
        );
        if let Some(link) = &mention.article.link {
            println!("    {}", link);
        }
    }
}

async fn cmd_analyze(ticker: &str) {
    let config = Config::load().unwrap_or_default();
    let ticker_upper = ticker.to_uppercase();

    let investment = config
        .investments
        .iter()
        .find(|i| i.ticker == ticker_upper);

    if investment.is_none() {
        println!(
            "Ticker {} is not being tracked. Use 'aaron_rss stock add {}' first.",
            ticker_upper, ticker_upper
        );
        return;
    }

    println!("Analyzing {} ...\n", ticker_upper);

    // Fetch price history
    println!("Fetching price history...");
    let prices = match stock::fetch_history(ticker, 30).await {
        Ok(history) => {
            println!("Got {} days of price data.\n", history.prices.len());
            history.prices
        }
        Err(e) => {
            eprintln!("Error fetching price history: {}", e);
            Vec::new()
        }
    };

    // Display recent prices
    if !prices.is_empty() {
        println!("Recent prices:");
        for price in prices.iter().rev().take(5).rev() {
            println!("  {}: ${:.2}", price.date, price.close);
        }
        println!();
    }

    // Fetch and scan articles
    if config.feeds.is_empty() {
        println!("No feeds to scan. Add some feeds with 'aaron_rss add <url>'.");
        return;
    }

    println!("Scanning feeds for mentions...");
    let mut all_articles = Vec::new();

    for feed_url in &config.feeds {
        if let Ok(result) = feed::fetch_feed(feed_url).await {
            all_articles.extend(result.articles);
        }
    }

    let single_investment = vec![investment.unwrap().clone()];
    let mentions = analysis::find_mentions(&all_articles, &single_investment);

    if mentions.is_empty() {
        println!("No recent news mentions found for {}.", ticker_upper);
        return;
    }

    println!("Found {} mentions.\n", mentions.len());

    // Correlate with prices
    let correlations = analysis::correlate(&mentions, &prices);

    println!("News & Price Correlation:");
    println!("{:-<80}", "");

    for corr in &correlations {
        let sentiment_str = match corr.sentiment {
            analysis::Sentiment::Positive => "Positive",
            analysis::Sentiment::Negative => "Negative",
            analysis::Sentiment::Neutral => "Neutral ",
        };

        let price_str = match (corr.price, corr.price_change) {
            (Some(p), Some(c)) => {
                let sign = if c >= 0.0 { "+" } else { "" };
                format!("${:.2} ({}{:.1}%)", p, sign, c)
            }
            (Some(p), None) => format!("${:.2}", p),
            _ => "N/A".to_string(),
        };

        println!(
            "[{}] {} | {} | {}",
            corr.date, sentiment_str, price_str, corr.article_title
        );
    }
}
