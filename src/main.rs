mod feed;
mod storage;

use clap::{Parser, Subcommand};
use storage::Config;

#[derive(Parser)]
#[command(name = "rss")]
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
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add { url } => cmd_add(&url),
        Commands::Remove { url } => cmd_remove(&url),
        Commands::List => cmd_list(),
        Commands::Fetch { url } => cmd_fetch(url).await,
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
        println!("No feeds subscribed. Use 'rss add <url>' to add a feed.");
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
                println!("No feeds subscribed. Use 'rss add <url>' to add a feed.");
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
