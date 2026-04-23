use adw::prelude::*;
use serde::{Deserialize, Serialize};

mod types {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub enum WindowLevel {
        Normal,
        AlwaysOnTop,
        AlwaysOnBottom,
    }

    impl Default for WindowLevel {
        fn default() -> Self { Self::Normal }
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
}

use types::{parse_color, DisplayArgs};
use gtk::{glib, Application};
use pango::FontDescription;
use pangocairo::functions as pc;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use sha1::{Sha1, Digest};

const HANDLE_SIZE: f64 = 10.0;
const HIT_MARGIN: f64 = 15.0;

/// Long Marx quote used as a preview to visually show clipping
const PREVIEW_QUOTE: &str = "The history of all hitherto existing society is the history of class \
struggles. Freeman and slave, patrician and plebeian, lord and serf, guild-master and journeyman, \
in a word, oppressor and oppressed, stood in constant opposition to one another, carried on an \
uninterrupted, now hidden, now open fight, a fight that each time ended, either in a revolutionary \
reconstitution of society at large, or in the common ruin of the contending classes. In the earlier \
epochs of history, we find almost everywhere a complicated arrangement of society into various \
orders, a manifold gradation of social rank. In ancient Rome we have patricians, knights, plebeians, \
slaves; in the Middle Ages, feudal lords, vassals, guild-masters, journeymen, apprentices, serfs; \
in almost all of these classes, again, subordinate gradations. The modern bourgeois society that has \
sprouted from the ruins of feudal society has not done away with class antagonisms.";

const PREVIEW_AUTHOR: &str = "— Karl Marx";

#[derive(Clone, Copy, PartialEq)]
enum HoverTarget {
    None,
    QuoteBody,
    QuoteResizeRight,
    QuoteResizeBottom,
    QuoteResizeCorner,
    AuthorBody,
}

#[derive(Clone, Copy, PartialEq)]
enum DragMode {
    None,
    MoveQuote,
    MoveAuthor,
    ResizeWidth,
    ResizeHeight,
    ResizeBoth,
}

#[derive(Clone)]
struct State {
    args: DisplayArgs,
    hover: HoverTarget,
    drag_mode: DragMode,
    drag_start_val_x: i32,
    drag_start_val_y: i32,
    drag_start_width: i32,
    drag_start_height: i32,
    // Updated each draw: how many chars fit in the container
    visible_chars: usize,
    // Computed from pango: height of one text line in pixels
    line_height: i32,
}

fn main() -> glib::ExitCode {
    let app = Application::builder()
        .application_id("com.github.engyls.Place")
        .build();

    app.connect_startup(|_| {
        let _ = adw::init();
    });

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
    let config_path = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("~/.config"))
        .join("marxist_quote")
        .join("settings.json");

    let mut args: DisplayArgs = DisplayArgs::default();

    if let Ok(contents) = std::fs::read_to_string(&config_path) {
        let raw_json: String = contents
            .lines()
            .filter(|line| !line.starts_with("hash:"))
            .collect::<Vec<_>>()
            .join("\n");

        if let Ok(parsed) = serde_json::from_str(&raw_json) {
            args = parsed;
        }
    }

    let state = Rc::new(RefCell::new(State {
        args,
        hover: HoverTarget::None,
        drag_mode: DragMode::None,
        drag_start_val_x: 0,
        drag_start_val_y: 0,
        drag_start_width: 0,
        drag_start_height: 0,
        visible_chars: 0,
        line_height: 0,
    }));

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Place Quote")
        .decorated(false)
        .fullscreened(true)
        .build();

    // Semi-transparent overlay
    window.add_css_class("transparent");
    let provider = gtk::CssProvider::new();
    provider.load_from_data("window.transparent { background: rgba(36, 36, 36, 0.5); }");
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let overlay = gtk::Overlay::new();
    window.set_child(Some(&overlay));

    let drawing_area = gtk::DrawingArea::new();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);

    // --- Draw ---
    let state_draw = state.clone();
    drawing_area.set_draw_func(move |_, cr, _w, _h| {
        let mut s = state_draw.borrow_mut();
        draw_scene(cr, &mut s);
    });

    overlay.set_child(Some(&drawing_area));

    // --- Hover ---
    let motion = gtk::EventControllerMotion::new();
    let state_motion = state.clone();
    let da_motion = drawing_area.clone();
    motion.connect_motion(move |_, x, y| {
        let mut s = state_motion.borrow_mut();
        if s.drag_mode != DragMode::None { return; }
        s.hover = hit_test(&s, x, y);
        da_motion.queue_draw();

        let cursor = match s.hover {
            HoverTarget::QuoteBody | HoverTarget::AuthorBody => "grab",
            HoverTarget::QuoteResizeRight => "ew-resize",
            HoverTarget::QuoteResizeBottom => "ns-resize",
            HoverTarget::QuoteResizeCorner => "nwse-resize",
            HoverTarget::None => "default",
        };
        da_motion.set_cursor_from_name(Some(cursor));
    });
    drawing_area.add_controller(motion);

    // --- Drag ---
    let drag = gtk::GestureDrag::new();
    drag.set_button(gtk::gdk::BUTTON_PRIMARY);

    let state_begin = state.clone();
    drag.connect_drag_begin(move |_, _sx, _sy| {
        let mut s = state_begin.borrow_mut();
        match s.hover {
            HoverTarget::QuoteBody => {
                s.drag_mode = DragMode::MoveQuote;
                s.drag_start_val_x = s.args.appearance.quote_x;
                s.drag_start_val_y = s.args.appearance.quote_y;
            }
            HoverTarget::AuthorBody => {
                s.drag_mode = DragMode::MoveAuthor;
                s.drag_start_val_x = s.args.appearance.author_x;
                s.drag_start_val_y = s.args.appearance.author_y;
            }
            HoverTarget::QuoteResizeRight => {
                s.drag_mode = DragMode::ResizeWidth;
                s.drag_start_width = s.args.appearance.quote_max_width;
            }
            HoverTarget::QuoteResizeBottom => {
                s.drag_mode = DragMode::ResizeHeight;
                s.drag_start_height = s.args.appearance.quote_max_height;
            }
            HoverTarget::QuoteResizeCorner => {
                s.drag_mode = DragMode::ResizeBoth;
                s.drag_start_width = s.args.appearance.quote_max_width;
                s.drag_start_height = s.args.appearance.quote_max_height;
            }
            HoverTarget::None => {
                s.drag_mode = DragMode::None;
            }
        }
    });

    let state_update = state.clone();
    let da_update = drawing_area.clone();
    drag.connect_drag_update(move |_, ox, oy| {
        let mut s = state_update.borrow_mut();
        match s.drag_mode {
            DragMode::MoveQuote => {
                s.args.appearance.quote_x = s.drag_start_val_x + ox as i32;
                s.args.appearance.quote_y = s.drag_start_val_y + oy as i32;
            }
            DragMode::MoveAuthor => {
                s.args.appearance.author_x = s.drag_start_val_x + ox as i32;
                s.args.appearance.author_y = s.drag_start_val_y + oy as i32;
            }
            DragMode::ResizeWidth => {
                s.args.appearance.quote_max_width = (s.drag_start_width + ox as i32).max(200);
            }
            DragMode::ResizeHeight => {
                let raw = (s.drag_start_height + oy as i32).max(50);
                s.args.appearance.quote_max_height = snap_to_line(raw, s.line_height);
            }
            DragMode::ResizeBoth => {
                s.args.appearance.quote_max_width = (s.drag_start_width + ox as i32).max(200);
                let raw = (s.drag_start_height + oy as i32).max(50);
                s.args.appearance.quote_max_height = snap_to_line(raw, s.line_height);
            }
            DragMode::None => {}
        }
        // Update max_quote_chars from visible_chars on every resize tick
        if matches!(s.drag_mode, DragMode::ResizeWidth | DragMode::ResizeHeight | DragMode::ResizeBoth) {
            // Will be recalculated in draw
        }
        da_update.queue_draw();
    });

    let state_end = state.clone();
    let da_end = drawing_area.clone();
    drag.connect_drag_end(move |_, _, _| {
        let mut s = state_end.borrow_mut();
        // Commit the visible_chars count from the last draw
        s.args.appearance.max_quote_chars = s.visible_chars;
        s.drag_mode = DragMode::None;
        da_end.set_cursor_from_name(Some("default"));
        da_end.queue_draw();
    });

    drawing_area.add_controller(drag);

    // --- Info label ---
    let info_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    info_box.set_halign(gtk::Align::Center);
    info_box.set_valign(gtk::Align::Start);
    info_box.set_margin_top(24);
    let info_label = gtk::Label::new(Some("Drag to move  •  Edges to resize  •  Corner for both"));
    info_label.add_css_class("dim-label");
    let info_provider = gtk::CssProvider::new();
    info_provider.load_from_data(".dim-label { color: rgba(255,255,255,0.7); font-size: 16px; }");
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().unwrap(),
        &info_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    info_box.append(&info_label);
    overlay.add_overlay(&info_box);

    // --- Buttons ---
    let button_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    button_box.set_halign(gtk::Align::End);
    button_box.set_valign(gtk::Align::End);
    button_box.set_margin_end(24);
    button_box.set_margin_bottom(24);

    let cancel_btn = gtk::Button::with_label("Cancel");
    cancel_btn.add_css_class("destructive-action");
    let save_btn = gtk::Button::with_label("Save and Exit");
    save_btn.add_css_class("suggested-action");

    button_box.append(&cancel_btn);
    button_box.append(&save_btn);
    overlay.add_overlay(&button_box);

    let win_cancel = window.clone();
    cancel_btn.connect_clicked(move |_| { win_cancel.close(); });

    let state_save = state.clone();
    let win_save = window.clone();
    let cfg_path = config_path.clone();
    save_btn.connect_clicked(move |_| {
        let s = state_save.borrow();

        let json_str = serde_json::to_string_pretty(&s.args).unwrap();
        let mut hasher = Sha1::new();
        hasher.update(json_str.as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        if let Some(parent) = cfg_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&cfg_path, format!("{}\nhash:{}", json_str, hash));

        println!("Saved. Container {}×{}, max_quote_chars={}",
            s.args.appearance.quote_max_width,
            s.args.appearance.quote_max_height,
            s.args.appearance.max_quote_chars);
        win_save.close();
    });

    window.present();
}

// ─── Hit testing ────────────────────────────────────────────────────────

// ─── Helpers ────────────────────────────────────────────────────────────

/// Snap a raw pixel height to the nearest whole-line boundary
fn snap_to_line(raw: i32, line_height: i32) -> i32 {
    if line_height <= 0 {
        return raw.max(50);
    }
    let lines = ((raw as f64) / (line_height as f64)).round() as i32;
    (lines.max(1) * line_height).max(line_height)
}

fn hit_test(s: &State, x: f64, y: f64) -> HoverTarget {
    let a = &s.args.appearance;
    let qx = a.quote_x as f64;
    let qy = a.quote_y as f64;
    let qw = a.quote_max_width as f64;
    let qh = a.quote_max_height as f64;

    // Corner (bottom-right)
    if x >= qx + qw - HANDLE_SIZE && x <= qx + qw + HIT_MARGIN
        && y >= qy + qh - HANDLE_SIZE && y <= qy + qh + HIT_MARGIN
    {
        return HoverTarget::QuoteResizeCorner;
    }
    // Right edge
    if x >= qx + qw - HANDLE_SIZE && x <= qx + qw + HIT_MARGIN
        && y >= qy && y <= qy + qh
    {
        return HoverTarget::QuoteResizeRight;
    }
    // Bottom edge
    if y >= qy + qh - HANDLE_SIZE && y <= qy + qh + HIT_MARGIN
        && x >= qx && x <= qx + qw
    {
        return HoverTarget::QuoteResizeBottom;
    }
    // Quote body
    if x >= qx - HIT_MARGIN && x <= qx + qw + HIT_MARGIN
        && y >= qy - HIT_MARGIN && y <= qy + qh + HIT_MARGIN
    {
        return HoverTarget::QuoteBody;
    }

    // Author
    let ax = a.author_x as f64;
    let ay = a.author_y as f64;
    let aw = 300.0;
    let ah = 40.0;
    if x >= ax - HIT_MARGIN && x <= ax + aw + HIT_MARGIN
        && y >= ay - HIT_MARGIN && y <= ay + ah + HIT_MARGIN
    {
        return HoverTarget::AuthorBody;
    }

    HoverTarget::None
}

// ─── Drawing ────────────────────────────────────────────────────────────

fn draw_scene(cr: &cairo::Context, s: &mut State) {
    let a = &s.args.appearance;
    let (r, g, b, alpha) = parse_color(&a.text_color);
    let (bg_r, bg_g, bg_b, bg_a) = parse_color(&a.bg_color);

    let qx = a.quote_x as f64;
    let qy = a.quote_y as f64;
    let qw = a.quote_max_width as f64;
    let qh = a.quote_max_height as f64;
    let ax = a.author_x as f64;
    let ay = a.author_y as f64;

    let is_hovered_q = matches!(s.hover,
        HoverTarget::QuoteBody | HoverTarget::QuoteResizeRight |
        HoverTarget::QuoteResizeBottom | HoverTarget::QuoteResizeCorner);
    let is_resizing = matches!(s.drag_mode,
        DragMode::ResizeWidth | DragMode::ResizeHeight | DragMode::ResizeBoth);
    let is_dragging_q = s.drag_mode == DragMode::MoveQuote;

    // ── Container outline (always visible) ──
    cr.set_source_rgba(0.4, 0.7, 1.0, if is_hovered_q || is_resizing { 0.7 } else { 0.25 });
    cr.set_line_width(if is_hovered_q || is_resizing { 2.0 } else { 1.0 });
    cr.set_dash(&[6.0, 4.0], 0.0);
    cr.rectangle(qx, qy, qw, qh);
    cr.stroke().unwrap();
    cr.set_dash(&[], 0.0);

    // ── Resize handles ──
    if is_hovered_q || is_resizing {
        cr.set_source_rgba(0.3, 0.8, 1.0, 0.9);

        // Right edge handle (vertical bar)
        let rh_x = qx + qw - HANDLE_SIZE / 2.0;
        let rh_y = qy + qh / 2.0 - 15.0;
        cr.rectangle(rh_x, rh_y, HANDLE_SIZE, 30.0);
        cr.fill().unwrap();

        // Bottom edge handle (horizontal bar)
        let bh_x = qx + qw / 2.0 - 15.0;
        let bh_y = qy + qh - HANDLE_SIZE / 2.0;
        cr.rectangle(bh_x, bh_y, 30.0, HANDLE_SIZE);
        cr.fill().unwrap();

        // Corner handle (square)
        let cx = qx + qw - HANDLE_SIZE;
        let cy = qy + qh - HANDLE_SIZE;
        cr.rectangle(cx, cy, HANDLE_SIZE, HANDLE_SIZE);
        cr.fill().unwrap();
    }

    // ── Quote text (clipped to container) ──
    cr.save().unwrap();
    cr.rectangle(qx, qy, qw, qh);
    cr.clip();

    if is_dragging_q {
        cr.push_group();
    }

    let quote_layout = pc::create_layout(cr);
    let mut q_font = FontDescription::new();
    q_font.set_family(&a.font);
    q_font.set_size((a.font_size as i32) * pango::SCALE);
    quote_layout.set_font_description(Some(&q_font));
    quote_layout.set_text(PREVIEW_QUOTE);
    quote_layout.set_width(a.quote_max_width * pango::SCALE);
    quote_layout.set_wrap(pango::WrapMode::Word);
    quote_layout.set_alignment(pango::Alignment::Center);

    // Count visible characters by iterating pango layout lines
    let mut visible_chars: usize = 0;
    let max_h = a.quote_max_height;

    // Compute line height from first line for snapping
    let mut iter = quote_layout.iter();
    {
        let (_ink, logical) = iter.line_extents();
        let lh = logical.height() / pango::SCALE;
        if lh > 0 {
            s.line_height = lh;
        }
    }
    loop {
        if let Some(line) = iter.line_readonly() {
            let (_ink, logical) = iter.line_extents();
            let line_y = logical.y() / pango::SCALE;
            let line_h = logical.height() / pango::SCALE;

            if line_y + line_h > max_h {
                // This line is clipped — count partial if needed
                break;
            }

            let start = line.start_index() as usize;
            let len = line.length() as usize;
            if let Some(slice) = PREVIEW_QUOTE.get(start..start + len) {
                visible_chars += slice.chars().count();
            }
        }
        if !iter.next_line() {
            break;
        }
    }
    s.visible_chars = visible_chars;

    // Background
    if a.bg_enabled {
        let padding = 15.0;
        let radius = 10.0;
        cr.set_source_rgba(bg_r, bg_g, bg_b, bg_a);
        let (pw, ph) = quote_layout.pixel_size();
        let bx = qx - padding;
        let by = qy - padding;
        let bw = pw as f64 + padding * 2.0;
        let bh = ph as f64 + padding * 2.0;
        cr.new_sub_path();
        cr.arc(bx + bw - radius, by + radius, radius, -std::f64::consts::FRAC_PI_2, 0.0);
        cr.arc(bx + bw - radius, by + bh - radius, radius, 0.0, std::f64::consts::FRAC_PI_2);
        cr.arc(bx + radius, by + bh - radius, radius, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
        cr.arc(bx + radius, by + radius, radius, std::f64::consts::PI, -std::f64::consts::FRAC_PI_2);
        cr.close_path();
        cr.fill().unwrap();
    }

    // Stroke
    cr.move_to(qx, qy);
    if a.stroke_enabled {
        let (sr, sg, sb, sa) = parse_color(&a.stroke_color);
        cr.set_source_rgba(sr, sg, sb, sa);
        cr.set_line_width(a.stroke_width as f64);
        pc::layout_path(cr, &quote_layout);
        cr.stroke().unwrap();
    }

    // Text
    cr.move_to(qx, qy);
    cr.set_source_rgba(r, g, b, alpha);
    pc::show_layout(cr, &quote_layout);

    if is_dragging_q {
        cr.pop_group_to_source().unwrap();
        cr.paint_with_alpha(0.7).unwrap();
    }

    cr.restore().unwrap();

    // ── Char count info (below container) ──
    cr.set_source_rgba(0.3, 0.8, 1.0, 1.0);
    cr.move_to(qx, qy + qh + 14.0);
    let info = pc::create_layout(cr);
    let mut ifont = FontDescription::new();
    ifont.set_family("sans-serif");
    ifont.set_size(13 * pango::SCALE);
    info.set_font_description(Some(&ifont));
    let label = format!("Container {}×{} px  •  {} visible chars  •  max_quote_chars={}",
        a.quote_max_width, a.quote_max_height, visible_chars, a.max_quote_chars);
    info.set_text(&label);
    pc::show_layout(cr, &info);

    // ── Author block ──
    let is_dragging_a = s.drag_mode == DragMode::MoveAuthor;
    let is_hovered_a = s.hover == HoverTarget::AuthorBody;

    if is_hovered_a || is_dragging_a {
        cr.set_source_rgba(0.4, 0.7, 1.0, 0.6);
        cr.set_line_width(2.0);
        cr.set_dash(&[6.0, 4.0], 0.0);
        cr.rectangle(ax - 8.0, ay - 8.0, 300.0, 40.0);
        cr.stroke().unwrap();
        cr.set_dash(&[], 0.0);
    }

    if is_dragging_a { cr.save().unwrap(); cr.push_group(); }

    let author_layout = pc::create_layout(cr);
    let mut afont = FontDescription::new();
    afont.set_family(&a.font);
    afont.set_size(((a.font_size * 0.8) as i32) * pango::SCALE);
    author_layout.set_font_description(Some(&afont));
    author_layout.set_text(PREVIEW_AUTHOR);

    if a.stroke_enabled {
        cr.move_to(ax, ay);
        let (sr, sg, sb, sa) = parse_color(&a.stroke_color);
        cr.set_source_rgba(sr, sg, sb, sa);
        cr.set_line_width(a.stroke_width as f64);
        pc::layout_path(cr, &author_layout);
        cr.stroke().unwrap();
    }

    cr.move_to(ax, ay);
    cr.set_source_rgba(r, g, b, alpha);
    pc::show_layout(cr, &author_layout);

    if is_dragging_a {
        cr.pop_group_to_source().unwrap();
        cr.paint_with_alpha(0.7).unwrap();
        cr.restore().unwrap();
    }
}
