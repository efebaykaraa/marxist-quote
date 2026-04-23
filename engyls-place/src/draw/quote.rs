use crate::state::{State, PREVIEW_QUOTE};
use engyls::config::parse_color_to_rgba;
use pango::FontDescription;
use pangocairo::functions as pc;

pub fn draw_quote(cr: &cairo::Context, s: &mut State) {
    let a = &s.args.appearance;
    let (r, g, b, alpha) = parse_color_to_rgba(&a.text_color);
    let (bg_r, bg_g, bg_b, bg_a) = parse_color_to_rgba(&a.bg_color);

    let qx = a.quote_x as f64;
    let qy = a.quote_y as f64;
    let qw = a.quote_max_width as f64;
    let qh = a.quote_max_height as f64;

    let is_dragging_q = s.drag_mode == crate::events::DragMode::MoveQuote;

    // ── Quote background ──
    if a.bg_enabled {
        let padding_h = 12.0;
        let padding_v = 6.0;
        let radius = 8.0;
        
        cr.save().unwrap();
        cr.push_group();
        cr.set_source_rgba(bg_r, bg_g, bg_b, 1.0);

        let quote_layout_bg = pc::create_layout(cr);
        let mut q_font_bg = FontDescription::new();
        q_font_bg.set_family(&a.font);
        q_font_bg.set_size((a.font_size as i32) * pango::SCALE);
        quote_layout_bg.set_font_description(Some(&q_font_bg));
        quote_layout_bg.set_text(PREVIEW_QUOTE);
        quote_layout_bg.set_width(a.quote_max_width * pango::SCALE);
        quote_layout_bg.set_wrap(pango::WrapMode::Word);
        quote_layout_bg.set_alignment(pango::Alignment::Center);

        let mut iter = quote_layout_bg.iter();
        loop {
            let (_, logical) = iter.line_extents();
            let (ink, _) = iter.line_readonly().unwrap().extents();
            
            let lw = (ink.width() as f64) / pango::SCALE as f64;
            let lh = (logical.height() as f64) / pango::SCALE as f64;
            let ly = (logical.y() as f64) / pango::SCALE as f64;
            let lx = (qw - lw) / 2.0;

            let bx = qx + lx - padding_h;
            let by = qy + ly - padding_v;
            let bw = lw + padding_h * 2.0;
            let bh = lh + padding_v * 2.0;

            // Only draw if the line is mostly visible
            if ly + lh * 0.8 < qh {
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

    // ── Quote Text ──
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

    // Stroke
    cr.move_to(qx, qy);
    if a.stroke_enabled {
        let (sr, sg, sb, sa) = parse_color_to_rgba(&a.stroke_color);
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

    // Info label
    cr.set_source_rgba(0.3, 0.8, 1.0, 1.0);
    cr.move_to(qx, qy + qh + 14.0);
    let info = pc::create_layout(cr);
    let mut ifont = FontDescription::new();
    ifont.set_family("sans-serif");
    ifont.set_size(13 * pango::SCALE);
    info.set_font_description(Some(&ifont));
    let label = format!("Container {}×{} px", a.quote_max_width, a.quote_max_height);
    info.set_text(&label);
    pc::show_layout(cr, &info);
}
