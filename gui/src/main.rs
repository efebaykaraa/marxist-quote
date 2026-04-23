mod app;
mod config;
mod fetch;

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--fetch") {
        if let Err(e) = fetch::fetch_quote() {
            eprintln!("Failed to fetch quote: {}", e);
            std::process::exit(1);
        }
        return;
    }

    // Start GUI
    let app = relm4::RelmApp::new("com.github.marxist_quote");
    app.run::<app::AppModel>(());
}
