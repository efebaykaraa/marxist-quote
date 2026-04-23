use engyls::{DisplayArgs, run_display};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("marxist_quote")
        .join("settings.json");

    let cache_file = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("marxist_quote")
        .join("current_quote.txt");

    let mut args: DisplayArgs = DisplayArgs::default();

    if let Ok(contents) = std::fs::read_to_string(&config_path) {
        // Strip the hash line at the bottom
        let json_content: String = contents
            .lines()
            .filter(|line| !line.starts_with("hash:"))
            .collect::<Vec<_>>()
            .join("\n");

        if let Ok(parsed) = serde_json::from_str(&json_content) {
            args = parsed;
        }
    }

    let raw_text = std::fs::read_to_string(cache_file).unwrap_or_default();
    
    // Parse "quote" — Author
    let (quote_text, author_text) = if let Some((q, a)) = raw_text.rsplit_once(" — ") {
        (q.trim().trim_matches('"').to_string(), a.trim().to_string())
    } else {
        (raw_text.trim().to_string(), String::new())
    };

    run_display(args, &quote_text, &author_text);

    Ok(())
}
