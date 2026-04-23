use adw::prelude::*;
use gtk::{glib, Application};
use std::cell::RefCell;
use std::rc::Rc;

mod state;
mod draw;
mod events;

use state::State;
use draw::draw_scene;
use events::{hit_test, snap_to_line, HoverTarget, DragMode};
use engyls::config::ConfigManager;

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
    let args_os: Vec<String> = std::env::args().collect();
    let args = if args_os.len() > 1 {
        match serde_json::from_str::<engyls::config::DisplayArgs>(&args_os[1]) {
            Ok(a) => a,
            Err(_) => ConfigManager::load_settings().0,
        }
    } else {
        ConfigManager::load_settings().0
    };

    let (sw, sh) = if let Some(display) = gtk::gdk::Display::default() {
        if let Some(monitor) = display.monitors().item(0).and_then(|m| m.downcast::<gtk::gdk::Monitor>().ok()) {
            let geometry = monitor.geometry();
            (geometry.width() as f64, geometry.height() as f64)
        } else {
            (1920.0, 1080.0)
        }
    } else {
        (1920.0, 1080.0)
    };

    let state = Rc::new(RefCell::new(State {
        args,
        hover: HoverTarget::None,
        drag_mode: DragMode::None,
        drag_start_val_x: 0,
        drag_start_val_y: 0,
        drag_start_width: 0,
        drag_start_height: 0,
        capacity_chars: 0,
        line_height: 0,
        screen_width: sw,
        screen_height: sh,
    }));

    let window = gtk::ApplicationWindow::builder()
        .application(app)
        .title("Place Quote")
        .decorated(false)
        .fullscreened(true)
        .build();

    // Semi-transparent Libadwaita gray overlay
    window.add_css_class("transparent-bg");
    let provider = gtk::CssProvider::new();
    provider.load_from_data(".transparent-bg { background: rgba(36, 36, 36, 0.5); }");
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
                let raw_x = s.drag_start_val_x + ox as i32;
                s.args.appearance.quote_x = crate::events::snap_x(raw_x, s.args.appearance.quote_max_width, s.screen_width);
                s.args.appearance.quote_y = s.drag_start_val_y + oy as i32;
            }
            DragMode::MoveAuthor => {
                let raw_x = s.drag_start_val_x + ox as i32;
                s.args.appearance.author_x = crate::events::snap_x(raw_x, 300, s.screen_width); // Approx author width
                s.args.appearance.author_y = s.drag_start_val_y + oy as i32;
            }
            DragMode::ResizeWidth => {
                s.args.appearance.quote_max_width = (s.drag_start_width + ox as i32).max(200);
            }
            DragMode::ResizeHeight => {
                let raw = (s.drag_start_height + oy as i32).max(s.line_height);
                s.args.appearance.quote_max_height = snap_to_line(raw, s.line_height).max(s.line_height);
            }
            DragMode::None => {}
        }
        da_update.queue_draw();
    });

    let state_end = state.clone();
    let da_end = drawing_area.clone();
    drag.connect_drag_end(move |_, _, _| {
        let mut s = state_end.borrow_mut();
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
    let info_label = gtk::Label::new(Some("Drag to move  •  Edges to resize"));
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
    save_btn.connect_clicked(move |_| {
        let mut s = state_save.borrow_mut();
        s.args.appearance.max_quote_chars = s.capacity_chars;
        s.args.appearance.position_hash = s.args.calculate_position_hash();

        // Save
        let _ = ConfigManager::save_settings(&s.args);
        println!("Saved. Container {}×{}, max_quote_chars={}",
            s.args.appearance.quote_max_width,
            s.args.appearance.quote_max_height,
            s.args.appearance.max_quote_chars);
        win_save.close();
    });

    window.present();
}
