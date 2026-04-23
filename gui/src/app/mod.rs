pub mod input;
pub mod model;

use relm4::{adw, gtk, ComponentParts, ComponentSender, Component, RelmWidgetExt};
use adw::prelude::*;
use engyls::config::ConfigManager;
use engyls::fetch;
pub use model::AppModel;
pub use input::AppInput;

pub struct AppWidgets {
    pub window: adw::Window,
    pub quote_entry: gtk::Entry,
    pub bg_entry: gtk::Entry,
    pub stroke_entry: gtk::Entry,
    pub shadow_entry: gtk::Entry,
    pub bg_scale: gtk::Scale,
    pub bg_spin: gtk::SpinButton,
}

impl Component for AppModel {
    type Init = ();
    type Input = AppInput;
    type Output = ();
    type CommandOutput = ();
    type Root = adw::Window;
    type Widgets = AppWidgets;

    fn init_root() -> Self::Root {
        adw::Window::builder()
            .default_width(500)
            .default_height(700)
            .build()
    }

    fn init(
        _params: (),
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppModel::init_model();

        let main_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.set_content(Some(&main_box));

        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("", "");
        header.set_title_widget(Some(&title));

        let save_btn = gtk::Button::with_label("Save & Apply");
        save_btn.add_css_class("suggested-action");
        let s_clone = sender.clone();
        save_btn.connect_clicked(move |_| s_clone.input(AppInput::Save));
        header.pack_start(&save_btn);

        let picker_btn = gtk::Button::with_label("Text Position");
        let s_clone = sender.clone();
        picker_btn.connect_clicked(move |_| s_clone.input(AppInput::ShowInteractivePicker));
        header.pack_end(&picker_btn);
        
        main_box.append(&header);

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_vexpand(true);
        main_box.append(&scroll);

        let content = gtk::Box::new(gtk::Orientation::Vertical, 20);
        content.set_margin_all(20);
        scroll.set_child(Some(&content));

        // --- Text Style ---
        content.append(&gtk::Label::builder().label("Text Style").css_classes(vec!["title-4".to_string()]).halign(gtk::Align::Start).build());

        let font_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        font_box.append(&gtk::Label::new(Some("Font:")));
        let combo = gtk::ComboBoxText::new();
        for f in ["Inter", "Roboto", "Ubuntu", "Cantarell", "sans-serif", "serif", "monospace"] {
            combo.append(Some(f), f);
        }
        combo.set_active_id(Some(&model.settings.appearance.font));
        let s_clone = sender.clone();
        combo.connect_changed(move |c| {
            if let Some(id) = c.active_id() { s_clone.input(AppInput::UpdateFont(id.to_string())); }
        });
        font_box.append(&combo);
        content.append(&font_box);

        let size_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        size_box.append(&gtk::Label::new(Some("Size:")));
        let size_spin = gtk::SpinButton::with_range(8.0, 120.0, 1.0);
        size_spin.set_value(model.settings.appearance.font_size as f64);
        let s_clone = sender.clone();
        size_spin.connect_value_changed(move |s| s_clone.input(AppInput::UpdateFontSize(s.value() as f32)));
        size_box.append(&size_spin);
        content.append(&size_box);

        let color_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        color_box.append(&gtk::Label::new(Some("Color:")));
        let quote_entry = gtk::Entry::new();
        quote_entry.set_text(&model.settings.appearance.text_color);
        quote_entry.set_hexpand(true);
        let s_clone = sender.clone();
        quote_entry.connect_changed(move |e| s_clone.input(AppInput::UpdateTextColor(e.text().to_string())));
        color_box.append(&quote_entry);
        
        let quote_color = gtk::ColorButton::new();
        let (r,g,b,a) = engyls::config::parse_color_to_rgba(&model.settings.appearance.text_color);
        quote_color.set_rgba(&gtk::gdk::RGBA::new(r as f32, g as f32, b as f32, a as f32));
        let s_clone = sender.clone();
        quote_color.connect_color_set(move |btn| {
            let rgba = btn.rgba();
            let hex = engyls::config::rgba_to_hex(rgba.red() as f64, rgba.green() as f64, rgba.blue() as f64, rgba.alpha() as f64);
            s_clone.input(AppInput::UpdateTextColor(hex));
        });
        color_box.append(&quote_color);
        content.append(&color_box);

        content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // --- Background ---
        let bg_title_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        bg_title_box.append(&gtk::Label::builder().label("Background").hexpand(true).halign(gtk::Align::Start).build());
        let bg_switch = gtk::Switch::new();
        bg_switch.set_active(model.settings.appearance.bg_enabled);
        let s_clone = sender.clone();
        bg_switch.connect_state_set(move |_, state| { s_clone.input(AppInput::UpdateBgEnabled(state)); gtk::glib::Propagation::Proceed });
        bg_title_box.append(&bg_switch);
        content.append(&bg_title_box);

        let bg_color_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        bg_color_box.append(&gtk::Label::new(Some("BG Color:")));
        let bg_entry = gtk::Entry::new();
        bg_entry.set_text(&model.settings.appearance.bg_color);
        bg_entry.set_hexpand(true);
        let s_clone = sender.clone();
        bg_entry.connect_changed(move |e| s_clone.input(AppInput::UpdateBgColor(e.text().to_string())));
        bg_color_box.append(&bg_entry);

        let bg_color_btn = gtk::ColorButton::new();
        let (r,g,b,a) = engyls::config::parse_color_to_rgba(&model.settings.appearance.bg_color);
        bg_color_btn.set_rgba(&gtk::gdk::RGBA::new(r as f32, g as f32, b as f32, a as f32));
        let s_clone = sender.clone();
        bg_color_btn.connect_color_set(move |btn| {
            let rgba = btn.rgba();
            let hex = engyls::config::rgba_to_hex(rgba.red() as f64, rgba.green() as f64, rgba.blue() as f64, rgba.alpha() as f64);
            s_clone.input(AppInput::UpdateBgColor(hex));
        });
        bg_color_box.append(&bg_color_btn);
        content.append(&bg_color_box);

        let opacity_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        opacity_box.append(&gtk::Label::new(Some("Opacity:")));
        let bg_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.01);
        bg_scale.set_hexpand(true);
        let (_,_,_,ba) = engyls::config::parse_color_to_rgba(&model.settings.appearance.bg_color);
        bg_scale.set_value(ba);
        let s_clone = sender.clone();
        bg_scale.connect_value_changed(move |s| s_clone.input(AppInput::UpdateBgOpacity(s.value() as f32)));
        opacity_box.append(&bg_scale);

        let bg_spin = gtk::SpinButton::with_range(0.0, 1.0, 0.05);
        bg_spin.set_value(ba);
        let s_clone = sender.clone();
        bg_spin.connect_value_changed(move |s| s_clone.input(AppInput::UpdateBgOpacity(s.value() as f32)));
        opacity_box.append(&bg_spin);
        content.append(&opacity_box);

        content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // --- Stroke ---
        let stroke_title_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        stroke_title_box.append(&gtk::Label::builder().label("Text Stroke").hexpand(true).halign(gtk::Align::Start).build());
        let stroke_switch = gtk::Switch::new();
        stroke_switch.set_active(model.settings.appearance.stroke_enabled);
        let s_clone = sender.clone();
        stroke_switch.connect_state_set(move |_, state| { s_clone.input(AppInput::UpdateStrokeEnabled(state)); gtk::glib::Propagation::Proceed });
        stroke_title_box.append(&stroke_switch);
        content.append(&stroke_title_box);

        let stroke_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        stroke_box.append(&gtk::Label::new(Some("Stroke:")));
        let stroke_entry = gtk::Entry::new();
        stroke_entry.set_text(&model.settings.appearance.stroke_color);
        stroke_entry.set_hexpand(true);
        let s_clone = sender.clone();
        stroke_entry.connect_changed(move |e| s_clone.input(AppInput::UpdateStrokeColor(e.text().to_string())));
        stroke_box.append(&stroke_entry);
        let stroke_spin = gtk::SpinButton::with_range(0.5, 10.0, 0.5);
        stroke_spin.set_value(model.settings.appearance.stroke_width as f64);
        let s_clone = sender.clone();
        stroke_spin.connect_value_changed(move |s| s_clone.input(AppInput::UpdateStrokeWidth(s.value() as f32)));
        stroke_box.append(&stroke_spin);
        content.append(&stroke_box);

        // --- Shadow ---
        let shadow_title_box = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        shadow_title_box.append(&gtk::Label::builder().label("Text Shadow").hexpand(true).halign(gtk::Align::Start).build());
        let shadow_switch = gtk::Switch::new();
        shadow_switch.set_active(model.settings.appearance.shadow_enabled);
        let s_clone = sender.clone();
        shadow_switch.connect_state_set(move |_, state| { s_clone.input(AppInput::UpdateShadowEnabled(state)); gtk::glib::Propagation::Proceed });
        shadow_title_box.append(&shadow_switch);
        content.append(&shadow_title_box);

        let shadow_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        shadow_box.append(&gtk::Label::new(Some("Shadow:")));
        let shadow_entry = gtk::Entry::new();
        shadow_entry.set_text(&model.settings.appearance.shadow_color);
        shadow_entry.set_hexpand(true);
        let s_clone = sender.clone();
        shadow_entry.connect_changed(move |e| s_clone.input(AppInput::UpdateShadowColor(e.text().to_string())));
        shadow_box.append(&shadow_entry);
        let shadow_spin = gtk::SpinButton::with_range(0.0, 20.0, 1.0);
        shadow_spin.set_value(model.settings.appearance.shadow_offset as f64);
        let s_clone = sender.clone();
        shadow_spin.connect_value_changed(move |s| s_clone.input(AppInput::UpdateShadowOffset(s.value() as f32)));
        shadow_box.append(&shadow_spin);
        content.append(&shadow_box);

        content.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

        // --- Authors ---
        let author_header = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        author_header.append(&gtk::Label::builder().label("Authors Weights").hexpand(true).halign(gtk::Align::Start).css_classes(vec!["title-4".to_string()]).build());
        let fetch_btn = gtk::Button::with_label("Fetch New Quote Now");
        let s_clone = sender.clone();
        fetch_btn.connect_clicked(move |_| s_clone.input(AppInput::FetchQuoteNow));
        author_header.append(&fetch_btn);
        content.append(&author_header);

        content.append(&model.update_author_list(&sender));

        let widgets = AppWidgets {
            window: root.clone(),
            quote_entry,
            bg_entry,
            stroke_entry,
            shadow_entry,
            bg_scale,
            bg_spin,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        log::debug!("Received input: {:?}", input);
        match input {
            AppInput::UpdateFont(val) => { if !val.is_empty() { self.settings.appearance.font = val; } }
            AppInput::UpdateFontSize(val) => self.settings.appearance.font_size = val,
            AppInput::UpdateTextColor(val) => { if !val.is_empty() && val.starts_with('#') { self.settings.appearance.text_color = val; } }
            AppInput::UpdateBgColor(val) => { if !val.is_empty() && val.starts_with('#') { self.settings.appearance.bg_color = val; } }
            AppInput::UpdateBgOpacity(a) => {
                let (r, g, b, _) = engyls::config::parse_color_to_rgba(&self.settings.appearance.bg_color);
                self.settings.appearance.bg_color = engyls::config::rgba_to_hex(r, g, b, a as f64);
            }
            AppInput::UpdateBgEnabled(val) => self.settings.appearance.bg_enabled = val,
            AppInput::UpdateStrokeColor(val) => { if !val.is_empty() && val.starts_with('#') { self.settings.appearance.stroke_color = val; } }
            AppInput::UpdateStrokeEnabled(val) => self.settings.appearance.stroke_enabled = val,
            AppInput::UpdateStrokeWidth(val) => self.settings.appearance.stroke_width = val,
            AppInput::UpdateShadowColor(val) => { if !val.is_empty() && val.starts_with('#') { self.settings.appearance.shadow_color = val; } }
            AppInput::UpdateShadowEnabled(val) => self.settings.appearance.shadow_enabled = val,
            AppInput::UpdateShadowOffset(val) => self.settings.appearance.shadow_offset = val,
            AppInput::UpdateAuthorWeight(name, weight) => {
                if let Some(author) = self.authors.authors.iter_mut().find(|a| a.name == name) {
                    author.weight = weight;
                }
            }
            AppInput::Save => {
                self.settings.appearance.position_hash = self.settings.calculate_position_hash();
                let _ = ConfigManager::save_authors(&self.authors);
                let _ = ConfigManager::save_settings(&self.settings);
            }
            AppInput::ShowInteractivePicker => {
                let bin_path = std::env::current_exe().unwrap().parent().unwrap().join("engyls-place");
                let json = serde_json::to_string(&self.settings).unwrap_or_default();
                let _ = std::process::Command::new(bin_path).arg(json).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn();
            }
            AppInput::FetchQuoteNow => {
                std::thread::spawn(|| { if let Err(e) = fetch::fetch_quote() { log::error!("Error: {}", e); } });
            }
        }
    }

    fn update_view(&self, widgets: &mut Self::Widgets, _sender: ComponentSender<Self>) {
        // --- MANUAL VIEW UPDATE WITH EQUALITY CHECKS ---
        let a = &self.settings.appearance;
        
        // Quote Color Entry
        if widgets.quote_entry.text() != a.text_color {
            widgets.quote_entry.set_text(&a.text_color);
        }

        // BG Color Entry
        if widgets.bg_entry.text() != a.bg_color {
            widgets.bg_entry.set_text(&a.bg_color);
        }

        // Opacity widgets
        let (_, _, _, ba) = engyls::config::parse_color_to_rgba(&a.bg_color);
        if (widgets.bg_scale.value() - ba).abs() > 0.001 {
            widgets.bg_scale.set_value(ba);
        }
        if (widgets.bg_spin.value() - ba).abs() > 0.001 {
            widgets.bg_spin.set_value(ba);
        }

        // Stroke
        if widgets.stroke_entry.text() != a.stroke_color {
            widgets.stroke_entry.set_text(&a.stroke_color);
        }

        // Shadow
        if widgets.shadow_entry.text() != a.shadow_color {
            widgets.shadow_entry.set_text(&a.shadow_color);
        }
    }
}
