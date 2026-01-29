# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build release binary
cargo build --release

# Run directly (during development)
cargo run -- <command>

# Run tests
cargo test
```

The binary is built to `target/release/aaron_rss`.

## Architecture

This is a Rust CLI tool for RSS feed aggregation with investment tracking and sentiment analysis.

### Source Files (`src/`)

- **main.rs** - CLI entry point using clap derive macros. Defines command structure:
  - Feed commands: `add`, `remove`, `list`, `fetch`
  - Stock commands: `stock add|remove|list|quote`
  - Analysis commands: `scan`, `analyze`

- **feed.rs** - RSS/Atom feed fetching using `feed-rs`. Returns `FeedResult` with title and up to 10 articles per feed.

- **storage.rs** - JSON-based config persistence at `~/.config/rss-reader/config.json`. Manages feed URLs and investment tracking list (ticker + optional company name).

- **stock.rs** - Stock price data via Yahoo Finance API. Provides current quotes and 30-day price history.

- **analysis.rs** - Sentiment analysis and stock correlation. Uses regex for ticker/company name matching and keyword-based sentiment classification (positive/negative/neutral).

### Data Flow

CLI commands load config from storage, fetch feeds or stock data via HTTP, run analysis if requested, display results, and save updated config.

## Sub-Projects (Untracked)

The repository contains additional untracked projects in subdirectories:

- **bmi-calculator/** - PHP web app with AWS deployment scripts
- **financial-rss-web/** - C# .NET 10.0 web API (3-layer architecture)
- **aazamm_tasks_meetings/** - Rust CLI for team task management (binary: `atm`)
