pub mod types;
pub mod manager;
pub mod utils;

pub use types::{Appearance, DisplayArgs, Author, AuthorsConfig};
pub use manager::ConfigManager;
pub use utils::{parse_color_to_rgba, rgba_to_hex};
