mod app;

use std::env;
use engyls::fetch;
use engyls::config::ConfigManager;
use simplelog::*;
use std::fs::File;

fn main() {
    let log_path = ConfigManager::config_dir().join("gui.log");
    let _ = std::fs::create_dir_all(ConfigManager::config_dir());
    
    let _ = CombinedLogger::init(vec![
        TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
        WriteLogger::new(LevelFilter::Debug, Config::default(), File::create(log_path).unwrap()),
    ]);

    log::info!("Starting Marxist Quote GUI...");

    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--fetch") {
        if let Err(e) = fetch::fetch_quote() {
            log::error!("Error fetching quote: {}", e);
        }
        return;
    }

    // Start GUI
    let app = relm4::RelmApp::new("com.github.marxist_quote");
    app.run::<app::AppModel>(());
}
