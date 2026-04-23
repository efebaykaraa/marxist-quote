use engyls::config::{AuthorsConfig, DisplayArgs, ConfigManager};
use relm4::{ComponentSender, gtk};
use adw::prelude::*;
use crate::app::input::AppInput;

#[derive(Clone)]
pub struct AppModel {
    pub authors: AuthorsConfig,
    pub settings: DisplayArgs,
    pub _authors_hash: String,
    pub _settings_hash: String,
}

impl AppModel {
    pub fn init_model() -> Self {
        let (authors, authors_hash) = ConfigManager::load_authors();
        let (settings, settings_hash) = ConfigManager::load_settings();
        
        AppModel {
            authors,
            settings,
            _authors_hash: authors_hash,
            _settings_hash: settings_hash,
        }
    }

    pub fn update_author_list(&self, sender: &ComponentSender<super::AppModel>) -> gtk::Box {
        let container = gtk::Box::new(gtk::Orientation::Vertical, 10);
        for author in &self.authors.authors {
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            row.append(&gtk::Label::builder()
                .label(&author.name)
                .hexpand(true)
                .halign(gtk::Align::Start)
                .build());
            
            let weight_spin = gtk::SpinButton::with_range(0.0, 10.0, 1.0);
            weight_spin.set_value(author.weight as f64);
            weight_spin.set_tooltip_text(Some("Relative weight: how often this author is picked compared to others."));
            
            let name = author.name.clone();
            let sender_clone = sender.clone();
            weight_spin.connect_value_changed(move |spin| {
                sender_clone.input(AppInput::UpdateAuthorWeight(name.clone(), spin.value() as u32));
            });
            
            row.append(&weight_spin);
            container.append(&row);
        }
        container
    }
}
