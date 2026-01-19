use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Investment {
    pub ticker: String,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    pub feeds: Vec<String>,
    #[serde(default)]
    pub investments: Vec<Investment>,
}

impl Config {
    pub fn load() -> io::Result<Self> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(Config::default());
        }
        let content = fs::read_to_string(&path)?;
        serde_json::from_str(&content)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn save(&self) -> io::Result<()> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(&path, content)
    }

    pub fn add_feed(&mut self, url: &str) -> bool {
        if self.feeds.contains(&url.to_string()) {
            return false;
        }
        self.feeds.push(url.to_string());
        true
    }

    pub fn remove_feed(&mut self, url: &str) -> bool {
        if let Some(pos) = self.feeds.iter().position(|f| f == url) {
            self.feeds.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn add_investment(&mut self, ticker: &str, name: Option<String>) -> bool {
        let ticker_upper = ticker.to_uppercase();
        if self.investments.iter().any(|i| i.ticker == ticker_upper) {
            return false;
        }
        self.investments.push(Investment {
            ticker: ticker_upper,
            name,
        });
        true
    }

    pub fn remove_investment(&mut self, ticker: &str) -> bool {
        let ticker_upper = ticker.to_uppercase();
        if let Some(pos) = self.investments.iter().position(|i| i.ticker == ticker_upper) {
            self.investments.remove(pos);
            true
        } else {
            false
        }
    }
}

fn config_path() -> io::Result<PathBuf> {
    let config_dir = dirs::config_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Config directory not found"))?;
    Ok(config_dir.join("rss-reader").join("config.json"))
}
