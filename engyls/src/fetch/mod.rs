pub mod pool;
pub mod wikiquote;

use crate::config::ConfigManager;
use rand::Rng;
use std::path::PathBuf;
pub use pool::QuotePool;
pub use wikiquote::fetch_wikiquote;

/// Fetch a random quote from WikiQuote for a weighted-random author and save it to the cache.
pub fn fetch_quote() -> anyhow::Result<()> {
    let (authors_cfg, _) = ConfigManager::load_authors();
    let (mut settings_cfg, _) = ConfigManager::load_settings();
    
    let authors = authors_cfg.authors;
    let total_weight: u32 = authors.iter().map(|a| a.weight).sum();
    if total_weight == 0 {
        anyhow::bail!("Total weight of authors is zero");
    }

    let mut rng = rand::rng();
    let mut chosen_weight = rng.random_range(0..total_weight);
    let mut selected_author = authors.first().map(|a| a.name.as_str()).unwrap_or("Karl Marx");

    for author in &authors {
        if chosen_weight < author.weight {
            selected_author = &author.name;
            break;
        }
        chosen_weight -= author.weight;
    }

    let current_hash = settings_cfg.calculate_position_hash();
    let max_chars = settings_cfg.appearance.max_quote_chars;

    println!("Picking quote for {} (max chars: {}, hash: {})", selected_author, max_chars, current_hash);

    let mut pool = QuotePool::load(selected_author).unwrap_or_else(|| QuotePool {
        position_hash: String::new(),
        quotes: Vec::new(),
    });

    // If hash changed or pool empty, refetch
    if pool.position_hash != current_hash || pool.quotes.is_empty() {
        println!("Hash mismatch or empty pool. Refetching from WikiQuote...");
        let new_quotes = fetch_wikiquote(selected_author)?;
        pool.quotes = new_quotes;
        pool.position_hash = current_hash.clone();
        settings_cfg.appearance.position_hash = current_hash;
        let _ = ConfigManager::save_settings(&settings_cfg);
    }

    // Pick and filter
    let mut chosen_quote = String::new();
    while !pool.quotes.is_empty() {
        let idx = rng.random_range(0..pool.quotes.len());
        let q = pool.quotes[idx].clone();
        
        if q.chars().count() <= max_chars {
            chosen_quote = q;
            break;
        } else {
            println!("Quote too long ({} chars), removing from pool.", q.chars().count());
            pool.quotes.remove(idx);
        }
    }

    if chosen_quote.is_empty() {
        anyhow::bail!("No fitting quotes found for {} in current pool. Try resizing or wait for next fetch.", selected_author);
    }

    let _ = pool.save(selected_author);

    let formatted = format!("\"{}\" — {}", chosen_quote, selected_author);
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("marxist_quote");
    let _ = std::fs::create_dir_all(&cache_dir);
    let _ = std::fs::write(cache_dir.join("current_quote.txt"), &formatted);

    println!("Selected: {}", formatted);
    Ok(())
}

/// Check if at least one quote for ANY configured author fits the character limit.
pub fn any_quote_fits_all_authors(max_chars: usize) -> anyhow::Result<bool> {
    let (authors_cfg, _) = ConfigManager::load_authors();
    for author in &authors_cfg.authors {
        let quotes = fetch_wikiquote(&author.name)?;
        if quotes.iter().any(|q| q.chars().count() <= max_chars) {
            return Ok(true);
        }
    }
    Ok(false)
}
