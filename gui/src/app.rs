use relm4::{adw, gtk, ComponentParts, ComponentSender, SimpleComponent, RelmWidgetExt};
use adw::prelude::*;
use crate::config::{AuthorsConfig, SettingsConfig, Author, ConfigManager};

pub struct AppModel {
    authors: AuthorsConfig,
    settings: SettingsConfig,
    authors_hash: String,
    settings_hash: String,
}

#[derive(Debug)]
pub enum AppInput {
    Save,
    AddAuthor,
    RemoveAuthor(usize),
    UpdateAuthorWeight(usize, u32),
    UpdateAuthorName(usize, String),
    UpdateFont(String),
    UpdateFontSize(f64),
    UpdateTextColor(String),
    UpdateBgColor(String),
    UpdateBgEnabled(bool),
    UpdateStrokeColor(String),
    UpdateStrokeEnabled(bool),
    UpdateStrokeWidth(f64),
    UpdateShadowColor(String),
    UpdateShadowEnabled(bool),
    UpdateShadowOffset(f64),

    FetchQuoteNow,
    PickPosition,
    ReloadConfig,
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppInput;
    type Output = ();

    view! {
        adw::Window {
            set_title: Some("Marxist Quote Config"),
            set_default_size: (500, 750),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                
                adw::HeaderBar {
                    pack_start = &gtk::Button {
                        set_label: "Save & Apply",
                        add_css_class: "suggested-action",
                        connect_clicked => AppInput::Save,
                    },
                    pack_start = &gtk::Button {
                        set_label: "Interactive Picker",
                        connect_clicked => AppInput::PickPosition,
                    },
                    pack_end = &gtk::Button {
                        set_label: "Fetch Quote Now",
                        connect_clicked => AppInput::FetchQuoteNow,
                    }
                },

                gtk::ScrolledWindow {
                    set_vexpand: true,
                    
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_all: 16,
                        set_spacing: 16,


                        // -- Appearance Section --
                        gtk::Label {
                            set_label: "<b>Appearance</b>",
                            set_use_markup: true,
                            set_halign: gtk::Align::Start,
                        },
                        
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 8,
                            gtk::Label { set_label: "Font:" },
                            gtk::Entry {
                                set_text: &model.settings.appearance.font,
                                set_hexpand: true,
                                connect_changed[sender] => move |entry| {
                                    sender.input(AppInput::UpdateFont(entry.text().to_string()));
                                }
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 8,
                            gtk::Label { set_label: "Font Size:" },
                            gtk::SpinButton::with_range(8.0, 100.0, 1.0) {
                                set_value: model.settings.appearance.font_size as f64,
                                connect_value_changed[sender] => move |spin| {
                                    sender.input(AppInput::UpdateFontSize(spin.value()));
                                }
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 8,
                            gtk::Label { set_label: "Text Color Hex:" },
                            gtk::Entry {
                                set_text: &model.settings.appearance.text_color,
                                connect_changed[sender] => move |entry| {
                                    sender.input(AppInput::UpdateTextColor(entry.text().to_string()));
                                }
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 8,
                            gtk::CheckButton {
                                set_label: Some("Enable Background"),
                                set_active: model.settings.appearance.bg_enabled,
                                connect_toggled[sender] => move |btn| {
                                    sender.input(AppInput::UpdateBgEnabled(btn.is_active()));
                                }
                            },
                            gtk::Entry {
                                set_text: &model.settings.appearance.bg_color,
                                connect_changed[sender] => move |entry| {
                                    sender.input(AppInput::UpdateBgColor(entry.text().to_string()));
                                }
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 8,
                            gtk::CheckButton {
                                set_label: Some("Enable Stroke"),
                                set_active: model.settings.appearance.stroke_enabled,
                                connect_toggled[sender] => move |btn| {
                                    sender.input(AppInput::UpdateStrokeEnabled(btn.is_active()));
                                }
                            },
                            gtk::Entry {
                                set_text: &model.settings.appearance.stroke_color,
                                connect_changed[sender] => move |entry| {
                                    sender.input(AppInput::UpdateStrokeColor(entry.text().to_string()));
                                }
                            },
                            gtk::SpinButton::with_range(0.5, 10.0, 0.5) {
                                set_value: model.settings.appearance.stroke_width as f64,
                                connect_value_changed[sender] => move |spin| {
                                    sender.input(AppInput::UpdateStrokeWidth(spin.value()));
                                }
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 8,
                            gtk::CheckButton {
                                set_label: Some("Enable Shadow"),
                                set_active: model.settings.appearance.shadow_enabled,
                                connect_toggled[sender] => move |btn| {
                                    sender.input(AppInput::UpdateShadowEnabled(btn.is_active()));
                                }
                            },
                            gtk::Entry {
                                set_text: &model.settings.appearance.shadow_color,
                                connect_changed[sender] => move |entry| {
                                    sender.input(AppInput::UpdateShadowColor(entry.text().to_string()));
                                }
                            },
                            gtk::SpinButton::with_range(0.0, 20.0, 1.0) {
                                set_value: model.settings.appearance.shadow_offset as f64,
                                connect_value_changed[sender] => move |spin| {
                                    sender.input(AppInput::UpdateShadowOffset(spin.value()));
                                }
                            }
                        },

                        gtk::Separator {},

                        // -- Authors Section --
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            gtk::Label {
                                set_label: "<b>Authors</b>",
                                set_use_markup: true,
                                set_halign: gtk::Align::Start,
                                set_hexpand: true,
                            },
                            gtk::Button {
                                set_label: "Add Author",
                                connect_clicked => AppInput::AddAuthor,
                            }
                        },

                        #[local_ref]
                        author_list -> gtk::ListBox {
                            set_selection_mode: gtk::SelectionMode::None,
                            set_css_classes: &["boxed-list"],
                        }
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (authors, authors_hash) = ConfigManager::load_authors();
        let (settings, settings_hash) = ConfigManager::load_settings();
        let model = AppModel { authors, settings, authors_hash, settings_hash };

        let author_list = gtk::ListBox::new();
        author_list.set_selection_mode(gtk::SelectionMode::None);
        author_list.add_css_class("boxed-list");

        let widgets = view_output!();
        
        Self::update_author_list(&model, &author_list, &sender);

        ComponentParts { model, widgets }
    }

    fn update(
        &mut self,
        msg: Self::Input,
        _sender: ComponentSender<Self>,
    ) {
        match msg {
            AppInput::Save => {
                let mut settings_changed = false;
                let mut authors_changed = false;

                match ConfigManager::save_authors(&self.authors) {
                    Ok(new_hash) => {
                        if new_hash != self.authors_hash {
                            authors_changed = true;
                            self.authors_hash = new_hash;
                        }
                    }
                    Err(e) => eprintln!("Failed to save authors: {e}"),
                }

                match ConfigManager::save_settings(&self.settings) {
                    Ok(new_hash) => {
                        if new_hash != self.settings_hash {
                            settings_changed = true;
                            self.settings_hash = new_hash;
                        }
                    }
                    Err(e) => eprintln!("Failed to save settings: {e}"),
                }

                println!("Saved configuration successfully.");

                if authors_changed {
                    println!("Authors changed, fetching new quote...");
                    std::thread::spawn(|| {
                        if let Err(e) = crate::fetch::fetch_quote() {
                            eprintln!("Error fetching quote: {}", e);
                        }
                    });
                }

                if settings_changed {
                    println!("Settings changed, restarting engyls-quote...");
                    std::thread::spawn(|| {
                        let _ = std::process::Command::new("pkill")
                            .arg("-x")
                            .arg("engyls-quote")
                            .output();

                        let mut engyls_path = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("target/debug/gui"));
                        engyls_path.set_file_name("engyls-quote");
                        
                        let _ = std::process::Command::new(engyls_path)
                            .spawn();
                    });
                }
            }
            AppInput::AddAuthor => {
                self.authors.authors.push(Author {
                    name: "New Author".into(),
                    weight: 1,
                });
            }
            AppInput::RemoveAuthor(idx) => {
                if idx < self.authors.authors.len() {
                    self.authors.authors.remove(idx);
                }
            }
            AppInput::UpdateAuthorWeight(idx, weight) => {
                if let Some(author) = self.authors.authors.get_mut(idx) {
                    author.weight = weight;
                }
            }
            AppInput::UpdateAuthorName(idx, name) => {
                if let Some(author) = self.authors.authors.get_mut(idx) {
                    author.name = name;
                }
            }
            AppInput::UpdateFont(val) => self.settings.appearance.font = val,
            AppInput::UpdateFontSize(val) => self.settings.appearance.font_size = val as f32,
            AppInput::UpdateTextColor(val) => self.settings.appearance.text_color = val,
            AppInput::UpdateBgColor(val) => self.settings.appearance.bg_color = val,
            AppInput::UpdateBgEnabled(val) => self.settings.appearance.bg_enabled = val,
            AppInput::UpdateStrokeColor(val) => self.settings.appearance.stroke_color = val,
            AppInput::UpdateStrokeEnabled(val) => self.settings.appearance.stroke_enabled = val,
            AppInput::UpdateStrokeWidth(val) => self.settings.appearance.stroke_width = val as f32,
            AppInput::UpdateShadowColor(val) => self.settings.appearance.shadow_color = val,
            AppInput::UpdateShadowEnabled(val) => self.settings.appearance.shadow_enabled = val,
            AppInput::UpdateShadowOffset(val) => self.settings.appearance.shadow_offset = val as f32,

            AppInput::FetchQuoteNow => {
                std::thread::spawn(|| {
                    if let Err(e) = crate::fetch::fetch_quote() {
                        eprintln!("Error fetching quote manually: {}", e);
                    }
                });
            }
            AppInput::PickPosition => {
                let _ = ConfigManager::save_settings(&self.settings);
                let sender_clone = _sender.clone();
                std::thread::spawn(move || {
                    let mut place_path = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("target/debug/gui"));
                    place_path.set_file_name("engyls-place");

                    println!("Launching engyls-place: {:?}", place_path);
                    match std::process::Command::new(&place_path).output() {
                        Ok(output) => {
                            if !output.status.success() {
                                eprintln!("engyls-place exited with: {}", output.status);
                                eprintln!("stderr: {}", String::from_utf8_lossy(&output.stderr));
                            }
                        }
                        Err(e) => eprintln!("Failed to launch engyls-place: {}", e),
                    }
                    sender_clone.input(AppInput::ReloadConfig);
                });
            }
            AppInput::ReloadConfig => {
                let (authors, authors_hash) = ConfigManager::load_authors();
                let (settings, settings_hash) = ConfigManager::load_settings();
                self.authors = authors;
                self.settings = settings;
                self.authors_hash = authors_hash;
                self.settings_hash = settings_hash;
                println!("Reloaded configuration from disk.");
            }
        }
    }
}

impl AppModel {
    fn update_author_list(
        model: &AppModel,
        list_box: &gtk::ListBox,
        sender: &ComponentSender<AppModel>,
    ) {
        // Clear list box
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }

        for (idx, author) in model.authors.authors.iter().enumerate() {
            let row = adw::ActionRow::new();
            
            let entry = gtk::Entry::builder()
                .text(&author.name)
                .valign(gtk::Align::Center)
                .build();
                
            let sender_clone = sender.clone();
            entry.connect_changed(move |e| {
                sender_clone.input(AppInput::UpdateAuthorName(idx, e.text().to_string()));
            });
            row.add_prefix(&entry);

            let spin = gtk::SpinButton::with_range(1.0, 3.0, 1.0);
            spin.set_valign(gtk::Align::Center);
            spin.set_value(author.weight as f64);
            let sender_clone2 = sender.clone();
            spin.connect_value_changed(move |s| {
                sender_clone2.input(AppInput::UpdateAuthorWeight(idx, s.value() as u32));
            });
            row.add_suffix(&spin);

            let btn = gtk::Button::builder()
                .icon_name("user-trash-symbolic")
                .valign(gtk::Align::Center)
                .build();
            btn.add_css_class("destructive-action");
            let sender_clone3 = sender.clone();
            btn.connect_clicked(move |_| {
                sender_clone3.input(AppInput::RemoveAuthor(idx));
            });
            row.add_suffix(&btn);

            list_box.append(&row);
        }
    }
}
