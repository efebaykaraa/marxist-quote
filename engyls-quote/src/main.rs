use std::path::PathBuf;
use gtk::prelude::*;
use pango::FontDescription;
use pangocairo::functions as pc;
use engyls::config::{DisplayArgs, ConfigManager, parse_color_to_rgba};

fn main() -> anyhow::Result<()> {
    let (args, _) = ConfigManager::load_settings();

    let cache_file = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("marxist_quote")
        .join("current_quote.txt");

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

pub fn run_display(args: DisplayArgs, quote_text: &str, author_text: &str) {
    // Force X11 backend so GNOME allows absolute positioning and hiding from taskbar
    unsafe {
        std::env::set_var("GDK_BACKEND", "x11");
    }
    
    gtk::init().expect("Failed to initialize GTK.");

    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Marxist Quote");
    window.set_decorated(false);
    window.set_app_paintable(true);
    window.set_keep_below(true);
    window.set_skip_taskbar_hint(true);
    window.set_skip_pager_hint(true);
    window.set_accept_focus(false);
    window.set_type_hint(gtk::gdk::WindowTypeHint::Desktop);

    let screen = gtk::prelude::WidgetExt::screen(&window).unwrap();
    let visual = screen.rgba_visual().unwrap();
    window.set_visual(Some(&visual));

    let args_draw = args.clone();
    let q_text = quote_text.to_string();
    let a_text = author_text.to_string();

    window.connect_draw(move |_, cr| {
        draw_quote(cr, &args_draw, &q_text, &a_text);
        false.into()
    });

    window.show_all();
    
    // Move to fixed position
    window.move_(args.appearance.quote_x, args.appearance.quote_y);

    gtk::main();
}

fn draw_quote(cr: &cairo::Context, args: &DisplayArgs, quote: &str, author: &str) {
    let a = &args.appearance;
    let (r, g, b, alpha) = parse_color_to_rgba(&a.text_color);
    let (bg_r, bg_g, bg_b, bg_a) = parse_color_to_rgba(&a.bg_color);

    let layout = pc::create_layout(cr);
    let mut font_desc = FontDescription::new();
    font_desc.set_family(&a.font);
    font_desc.set_size((a.font_size as i32) * pango::SCALE);
    layout.set_font_description(Some(&font_desc));
    layout.set_text(quote);
    layout.set_width(a.quote_max_width * pango::SCALE);
    layout.set_wrap(pango::WrapMode::Word);
    layout.set_alignment(pango::Alignment::Center);

    // Background (Instagram Style)
    if a.bg_enabled {
        let padding_h = 12.0;
        let padding_v = 6.0;
        let radius = 8.0;

        cr.save().unwrap();
        cr.push_group();
        cr.set_source_rgba(bg_r, bg_g, bg_b, 1.0);

        let mut iter = layout.iter();
        loop {
            let (_, logical) = iter.line_extents();
            let (ink, _) = iter.line_readonly().unwrap().extents();
            
            let lw = (ink.width() as f64) / pango::SCALE as f64;
            let lh = (logical.height() as f64) / pango::SCALE as f64;
            let ly = (logical.y() as f64) / pango::SCALE as f64;
            let lx = (a.quote_max_width as f64 - lw) / 2.0;

            let bx = lx - padding_h;
            let by = ly - padding_v;
            let bw = lw + padding_h * 2.0;
            let bh = lh + padding_v * 2.0;

            // Clip backgrounds to container height if text is long
            if ly + lh * 0.8 < (a.quote_max_height as f64) {
                cr.new_sub_path();
                cr.arc(bx + bw - radius, by + radius, radius, -std::f64::consts::FRAC_PI_2, 0.0);
                cr.arc(bx + bw - radius, by + bh - radius, radius, 0.0, std::f64::consts::FRAC_PI_2);
                cr.arc(bx + radius, by + bh - radius, radius, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
                cr.arc(bx + radius, by + radius, radius, std::f64::consts::PI, -std::f64::consts::FRAC_PI_2);
                cr.close_path();
                cr.fill().unwrap();
            }

            if !iter.next_line() { break; }
        }
        
        cr.pop_group_to_source().unwrap();
        cr.paint_with_alpha(bg_a as f64).unwrap();
        cr.restore().unwrap();
    }

    // Stroke
    cr.move_to(0.0, 0.0);
    if a.stroke_enabled {
        let (sr, sg, sb, sa) = parse_color_to_rgba(&a.stroke_color);
        cr.set_source_rgba(sr, sg, sb, sa);
        cr.set_line_width(a.stroke_width as f64);
        pc::layout_path(cr, &layout);
        cr.stroke().unwrap();
    }

    // Shadow
    if a.shadow_enabled {
        let (shr, shg, shb, sha) = parse_color_to_rgba(&a.shadow_color);
        cr.set_source_rgba(shr, shg, shb, sha);
        cr.save().unwrap();
        cr.translate(a.shadow_offset as f64, a.shadow_offset as f64);
        pc::show_layout(cr, &layout);
        cr.restore().unwrap();
    }

    // Main Text
    cr.move_to(0.0, 0.0);
    cr.set_source_rgba(r, g, b, alpha);
    pc::show_layout(cr, &layout);

    // Author
    if !author.is_empty() {
        let author_layout = pc::create_layout(cr);
        let mut afont = FontDescription::new();
        afont.set_family(&a.font);
        afont.set_size(((a.font_size * 0.8) as i32) * pango::SCALE);
        author_layout.set_font_description(Some(&afont));
        author_layout.set_text(author);

        let ox = (a.author_x - a.quote_x) as f64;
        let oy = (a.author_y - a.quote_y) as f64;

        if a.bg_enabled {
            let (r, g, b, a_val) = parse_color_to_rgba(&a.bg_color);
            let padding_h = 10.0;
            let padding_v = 4.0;
            let radius = 6.0;

            cr.save().unwrap();
            cr.push_group();
            cr.set_source_rgba(r, g, b, 1.0);

            let mut iter = author_layout.iter();
            loop {
                let (_, logical) = iter.line_extents();
                let (ink, _) = iter.line_readonly().unwrap().extents();
                
                let lx = (logical.x() as f64) / pango::SCALE as f64;
                let ly = (logical.y() as f64) / pango::SCALE as f64;
                let lw = (ink.width() as f64) / pango::SCALE as f64;
                let lh = (logical.height() as f64) / pango::SCALE as f64;

                let bx = ox + lx - padding_h;
                let by = oy + ly - padding_v;
                let bw = lw + padding_h * 2.0;
                let bh = lh + padding_v * 2.0;

                cr.new_sub_path();
                cr.arc(bx + bw - radius, by + radius, radius, -std::f64::consts::FRAC_PI_2, 0.0);
                cr.arc(bx + bw - radius, by + bh - radius, radius, 0.0, std::f64::consts::FRAC_PI_2);
                cr.arc(bx + radius, by + bh - radius, radius, std::f64::consts::FRAC_PI_2, std::f64::consts::PI);
                cr.arc(bx + radius, by + radius, radius, std::f64::consts::PI, -std::f64::consts::FRAC_PI_2);
                cr.close_path();
                cr.fill().unwrap();

                if !iter.next_line() { break; }
            }
            
            cr.pop_group_to_source().unwrap();
            cr.paint_with_alpha(a_val as f64).unwrap();
            cr.restore().unwrap();
        }

        cr.move_to(ox, oy);

        if a.stroke_enabled {
            let (sr, sg, sb, sa) = parse_color_to_rgba(&a.stroke_color);
            cr.set_source_rgba(sr, sg, sb, sa);
            cr.set_line_width(a.stroke_width as f64);
            pc::layout_path(cr, &author_layout);
            cr.stroke().unwrap();
        }

        cr.move_to(ox, oy);
        cr.set_source_rgba(r, g, b, alpha);
        pc::show_layout(cr, &author_layout);
    }
}
