use cairo::Context;
use gtk::prelude::*;
use pango::{FontDescription, Layout};
use pangocairo::functions as pc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WindowLevel {
    Normal,
    AlwaysOnTop,
    AlwaysOnBottom,
}

impl Default for WindowLevel {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appearance {
    pub font: String,
    pub font_size: f32,
    pub text_color: String,
    pub bg_color: String,
    pub bg_enabled: bool,
    pub stroke_color: String,
    pub stroke_enabled: bool,
    pub stroke_width: f32,
    pub shadow_color: String,
    pub shadow_enabled: bool,
    pub shadow_offset: f32,
    pub quote_x: i32,
    pub quote_y: i32,
    pub author_x: i32,
    pub author_y: i32,
    #[serde(default = "default_quote_max_width")]
    pub quote_max_width: i32,
    #[serde(default = "default_quote_max_height")]
    pub quote_max_height: i32,
    #[serde(default = "default_max_quote_chars")]
    pub max_quote_chars: usize,
}

fn default_quote_max_width() -> i32 { 800 }
fn default_quote_max_height() -> i32 { 300 }
fn default_max_quote_chars() -> usize { 500 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayArgs {
    pub appearance: Appearance,
    #[serde(default)]
    pub window_level: WindowLevel,
}

impl Default for DisplayArgs {
    fn default() -> Self {
        Self {
            appearance: Appearance {
                font: "Inter".into(),
                font_size: 24.0,
                text_color: "#ffffff".into(),
                bg_color: "#00000080".into(),
                bg_enabled: false,
                stroke_color: "#000000".into(),
                stroke_enabled: true,
                stroke_width: 2.0,
                shadow_color: "#000000ff".into(),
                shadow_enabled: true,
                shadow_offset: 2.0,
                quote_x: 100,
                quote_y: 100,
                author_x: 100,
                author_y: 200,
                quote_max_width: 800,
                quote_max_height: 300,
                max_quote_chars: 500,
            },
            window_level: WindowLevel::AlwaysOnBottom,
        }
    }
}

pub fn run_display(args: DisplayArgs, quote_text: &str, author_text: &str) {
    // Force X11 backend so GNOME allows absolute positioning and hiding from taskbar
    unsafe {
        std::env::set_var("GDK_BACKEND", "x11");
    }

    if gtk::init().is_err() {
        eprintln!("Failed to initialize GTK.");
        std::process::exit(1);
    }

    let window = gtk::Window::new(gtk::WindowType::Popup);
    window.set_title("Engyls Desktop Widget");
    window.set_decorated(false);
    window.set_skip_taskbar_hint(true);
    window.set_skip_pager_hint(true);

    // Quote display is always on bottom
    window.set_keep_below(true);

    window.set_accept_focus(false);

    // Make window transparent
    if let Some(screen) = gtk::prelude::WidgetExt::screen(&window) {
        if let Some(visual) = screen.rgba_visual() {
            if screen.is_composited() {
                window.set_visual(Some(&visual));
            }
        }
    }
    window.set_app_paintable(true);

    // Normal display mode
    if let Some(display) = gtk::gdk::Display::default() {
        let n = display.n_monitors();
        let mut w = 0;
        let mut h = 0;
        for i in 0..n {
            if let Some(monitor) = display.monitor(i) {
                let geom = monitor.geometry();
                w = w.max(geom.x() + geom.width());
                h = h.max(geom.y() + geom.height());
            }
        }
        if w > 0 && h > 0 {
            window.resize(w, h);
        }
    }
    window.move_(0, 0);
    
    // Input shape mask empty to click through
    let empty_region = cairo::Region::create();
    window.input_shape_combine_region(Some(&empty_region));

    let cfg = args.clone();
    let q = quote_text.to_string();
    let a = author_text.to_string();
    window.connect_draw(move |_, cr| {
        draw_widget(cr, &cfg, &q, &a);
        gtk::glib::Propagation::Proceed
    });

    window.show_all();
    gtk::main();
}

pub fn parse_color(hex: &str) -> (f64, f64, f64, f64) {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 || hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap_or(255) as f64 / 255.0
        } else {
            1.0
        };
        (r, g, b, a)
    } else {
        (1.0, 1.0, 1.0, 1.0)
    }
}

pub fn draw_widget(cr: &Context, cfg: &DisplayArgs, quote: &str, author: &str) {
    let app = &cfg.appearance;
    let (r, g, b, a) = parse_color(&app.text_color);
    let (bg_r, bg_g, bg_b, bg_a) = parse_color(&app.bg_color);
    
    // Create layouts
    let quote_layout = pc::create_layout(cr);
    let mut q_font = FontDescription::new();
    q_font.set_family(&app.font);
    q_font.set_size((app.font_size as i32) * pango::SCALE);
    quote_layout.set_font_description(Some(&q_font));
    quote_layout.set_text(quote);
    quote_layout.set_width(app.quote_max_width * pango::SCALE);
    quote_layout.set_height(app.quote_max_height * pango::SCALE);
    quote_layout.set_wrap(pango::WrapMode::Word);
    quote_layout.set_alignment(pango::Alignment::Center);
    quote_layout.set_ellipsize(pango::EllipsizeMode::End);

    let author_layout = pc::create_layout(cr);
    let mut a_font = FontDescription::new();
    a_font.set_family(&app.font);
    a_font.set_size(((app.font_size * 0.8) as i32) * pango::SCALE);
    author_layout.set_font_description(Some(&a_font));
    author_layout.set_text(author);

    let q_x = app.quote_x as f64;
    let q_y = app.quote_y as f64;
    let a_x = app.author_x as f64;
    let a_y = app.author_y as f64;

    let draw_layout = |layout: &Layout, x: f64, y: f64| {
        let (width, height) = layout.pixel_size();
        
        if app.bg_enabled {
            let padding = 15.0;
            let radius = 10.0;
            
            cr.set_source_rgba(bg_r, bg_g, bg_b, bg_a);
            
            let bx = x - padding;
            let by = y - padding;
            let bw = width as f64 + padding * 2.0;
            let bh = height as f64 + padding * 2.0;
            
            cr.new_sub_path();
            cr.arc(bx + bw - radius, by + radius, radius, -std::f64::consts::FRAC_PI_2, 0.0);
            cr.arc(bx + bw - radius, by + bh - radius, radius, 0.0, std::f64::consts::FRAC_PI_2);
            cr.arc(bx + radius, by + bh - radius, radius, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
            cr.arc(bx + radius, by + radius, radius, std::f64::consts::PI, -std::f64::consts::FRAC_PI_2);
            cr.close_path();
            cr.fill().unwrap();
        }

        cr.move_to(x, y);

        if app.stroke_enabled {
            let (sr, sg, sb, sa) = parse_color(&app.stroke_color);
            cr.set_source_rgba(sr, sg, sb, sa);
            cr.set_line_width(app.stroke_width as f64);
            pc::layout_path(cr, layout);
            cr.stroke().unwrap();
        }

        cr.move_to(x, y);
        cr.set_source_rgba(r, g, b, a);
        pc::show_layout(cr, layout);
    };

    draw_layout(&quote_layout, q_x, q_y);
    draw_layout(&author_layout, a_x, a_y);
}
