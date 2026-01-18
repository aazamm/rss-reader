# RSS Reader CLI

A simple command-line RSS reader written in Rust that allows you to manage and read RSS/Atom feeds from the terminal.

## Installation

```bash
cargo build --release
```

The binary will be at `target/release/rss`.

## Usage

```bash
# Add a feed
rss add <url>

# Remove a feed
rss remove <url>

# List all subscribed feeds
rss list

# Fetch and display recent articles from all feeds
rss fetch

# Fetch articles from a specific feed
rss fetch <url>
```

## Example

```bash
$ rss add https://blog.rust-lang.org/feed.xml
Added feed: https://blog.rust-lang.org/feed.xml

$ rss list
Subscribed feeds:
  1. https://blog.rust-lang.org/feed.xml

$ rss fetch
Fetching: https://blog.rust-lang.org/feed.xml
== Rust Blog ==

  [2026-01-14 00:00]
  What does it take to ship Rust in safety-critical?
  https://blog.rust-lang.org/2026/01/14/what-does-it-take-to-ship-rust-in-safety-critical/
  ...
```

## Configuration

Feed subscriptions are stored in `~/.config/rss-reader/config.json`.
