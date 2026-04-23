use crate::config::ConfigManager;
use rand::Rng;
use scraper::{Html, Selector};
use std::path::PathBuf;

/// Fetch a random quote from WikiQuote for a weighted-random author and save it to the cache.
pub fn fetch_quote() -> anyhow::Result<()> {
    let (authors_cfg, _) = ConfigManager::load_authors();
    let authors = authors_cfg.authors;

    // Pick author based on weights
    let total_weight: u32 = authors.iter().map(|a| a.weight).sum();
    if total_weight == 0 {
        anyhow::bail!("Total weight of authors is zero");
    }

    let mut rng = rand::rng();
    let mut chosen_weight = rng.random_range(0..total_weight);

    let mut selected_author = authors
        .first()
        .map(|a| a.name.as_str())
        .unwrap_or("Karl Marx");

    for author in &authors {
        if chosen_weight < author.weight {
            selected_author = &author.name;
            break;
        }
        chosen_weight -= author.weight;
    }

    let (settings_cfg, _) = ConfigManager::load_settings();
    let max_chars = settings_cfg.appearance.max_quote_chars;

    println!("Fetching quote for: {} (max {} chars)", selected_author, max_chars);

    let quotes = fetch_wikiquote(selected_author)?;

    // Filter quotes by max character count from container size
    let candidates: Vec<&String> = quotes.iter()
        .filter(|q| q.len() <= max_chars)
        .collect();

    let chosen = if candidates.is_empty() {
        if quotes.is_empty() {
            anyhow::bail!("No quotes found for {}", selected_author);
        }
        // Fallback: pick shortest if none fit
        quotes.iter().min_by_key(|q| q.len()).unwrap()
    } else {
        let idx = rng.random_range(0..candidates.len());
        candidates[idx]
    };

    // Format: "quote" — Author
    let formatted = format!("\"{}\" — {}", chosen, selected_author);

    // Save to cache
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("marxist_quote");

    std::fs::create_dir_all(&cache_dir)?;

    let cache_file = cache_dir.join("current_quote.txt");
    std::fs::write(&cache_file, &formatted)?;

    println!("Successfully saved new quote.");
    println!("{}", formatted);

    Ok(())
}

/// Query the WikiQuote MediaWiki API to get quotes for an author.
///
/// Strategy:
/// 1. Fetch the list of sections for the author's page.
/// 2. Identify "Quotes" sections (top-level sections containing actual quotes).
/// 3. For each sub-section under "Quotes", fetch the HTML and extract `<li>` text.
fn fetch_wikiquote(author: &str) -> anyhow::Result<Vec<String>> {
    let page_title = author.replace(' ', "_");

    // Step 1: Get sections to find quote sub-sections
    let sections_url = format!(
        "https://en.wikiquote.org/w/api.php?action=parse&page={}&format=json&prop=sections",
        urlencoded(&page_title)
    );

    let sections_body: String = ureq::get(&sections_url)
        .header("User-Agent", "MarxistQuoteApp/1.0 (Linux; Desktop Widget)")
        .call()?
        .body_mut()
        .read_to_string()?;

    let sections_json: serde_json::Value = serde_json::from_str(&sections_body)?;

    let sections = sections_json["parse"]["sections"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Failed to parse sections from WikiQuote API response"))?;

    // Find section indices that are sub-sections of "Quotes" (toclevel 2 under the Quotes heading)
    // Also include toclevel 1 "Quotes" section itself as fallback
    let mut quote_section_indices: Vec<String> = Vec::new();
    let mut in_quotes_section = false;

    for section in sections {
        let toclevel = section["toclevel"].as_u64().unwrap_or(0);
        let line = section["line"].as_str().unwrap_or("");
        let index = section["index"].as_str().unwrap_or("");

        if toclevel == 1 {
            in_quotes_section = line == "Quotes";
            // Don't add the top-level "Quotes" header itself — it rarely has quotes directly
        }

        if in_quotes_section && toclevel == 2 && !index.is_empty() {
            quote_section_indices.push(index.to_string());
        }
    }

    // If there are no sub-sections, try fetching section 1 directly (the "Quotes" header)
    if quote_section_indices.is_empty() {
        if let Some(quotes_section) = sections.iter().find(|s| {
            s["line"].as_str().unwrap_or("") == "Quotes"
        }) {
            if let Some(idx) = quotes_section["index"].as_str() {
                quote_section_indices.push(idx.to_string());
            }
        }
    }

    // Fallback: if still nothing, try sections 1 through 5
    if quote_section_indices.is_empty() {
        for i in 1..=5 {
            quote_section_indices.push(i.to_string());
        }
    }

    let mut all_quotes: Vec<String> = Vec::new();

    // Step 2: Fetch HTML for each section and extract quotes
    for section_idx in &quote_section_indices {
        let section_url = format!(
            "https://en.wikiquote.org/w/api.php?action=parse&page={}&format=json&prop=text&section={}",
            urlencoded(&page_title),
            section_idx
        );

        let body: String = match ureq::get(&section_url)
            .header("User-Agent", "MarxistQuoteApp/1.0 (Linux; Desktop Widget)")
            .call()
        {
            Ok(mut resp) => resp.body_mut().read_to_string()?,
            Err(_) => continue,
        };

        let json: serde_json::Value = match serde_json::from_str(&body) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let html_str = match json["parse"]["text"]["*"].as_str() {
            Some(s) => s,
            None => continue,
        };

        let extracted = extract_quotes_from_html(html_str);
        all_quotes.extend(extracted);

        // Cap at a reasonable number to avoid excessive API calls
        if all_quotes.len() >= 200 {
            break;
        }
    }

    Ok(all_quotes)
}

/// Extract quotes from WikiQuote section HTML.
///
/// WikiQuote structures quotes as top-level `<li>` elements inside `<ul>` blocks.
/// Nested `<ul>` within a `<li>` typically contains attribution/source info.
/// We extract only the direct text content of top-level `<li>` elements.
fn extract_quotes_from_html(html: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let ul_sel = Selector::parse("div.mw-parser-output > ul").unwrap();
    let li_sel = Selector::parse(":scope > li").unwrap();

    let mut quotes = Vec::new();

    for ul in document.select(&ul_sel) {
        for li in ul.select(&li_sel) {
            // Get the direct text of this <li>, excluding nested <ul> (attribution)
            let quote_text = extract_direct_text(&li);
            let cleaned = clean_quote(&quote_text);

            // Filter: must be reasonably long and not just a source/attribution
            if cleaned.len() >= 20 && !is_attribution(&cleaned) {
                quotes.push(cleaned);
            }
        }
    }

    quotes
}

/// Extract direct text content from an element, skipping nested <ul>/<dl> (attribution blocks).
fn extract_direct_text(element: &scraper::ElementRef) -> String {
    let mut text = String::new();
    for child in element.children() {
        match child.value() {
            scraper::node::Node::Text(t) => {
                text.push_str(t);
            }
            scraper::node::Node::Element(el) => {
                let tag = el.name();
                // Skip nested lists (attribution) and edit section links
                if tag == "ul" || tag == "dl" || tag == "span" && el.attr("class").map_or(false, |c| c.contains("editsection")) {
                    continue;
                }
                // Recurse into inline elements (b, i, a, etc.)
                if let Some(child_ref) = scraper::ElementRef::wrap(child) {
                    if tag == "sup" {
                        continue; // Skip footnote references
                    }
                    text.push_str(&extract_direct_text(&child_ref));
                }
            }
            _ => {}
        }
    }
    text
}

/// Clean up extracted quote text.
fn clean_quote(text: &str) -> String {
    let mut s = text.trim().to_string();

    // Remove leading/trailing quotation marks
    for mark in &['"', '"', '"', '«', '»', '\u{201C}', '\u{201D}'] {
        if s.starts_with(*mark) {
            s = s.trim_start_matches(*mark).to_string();
        }
        if s.ends_with(*mark) {
            s = s.trim_end_matches(*mark).to_string();
        }
    }

    // Collapse whitespace
    let s = s.split_whitespace().collect::<Vec<_>>().join(" ");

    s.trim().to_string()
}

/// Heuristic: check if text looks like an attribution rather than a quote.
fn is_attribution(text: &str) -> bool {
    let lower = text.to_lowercase();

    // Attributions typically start with these patterns
    let attribution_prefixes = [
        "as quoted in",
        "letter from",
        "letter to",
        "quoted in",
        "source:",
        "variant:",
        "see also",
        "compare:",
        "attributed",
        "paraphrase",
        "often misquoted",
        "sometimes attributed",
        "this is often",
    ];

    for prefix in &attribution_prefixes {
        if lower.starts_with(prefix) {
            return true;
        }
    }

    false
}

/// Minimal URL encoding for page titles.
fn urlencoded(s: &str) -> String {
    s.replace(' ', "%20")
        .replace('&', "%26")
        .replace('?', "%3F")
        .replace('#', "%23")
}
