use crate::state::{State, PREVIEW_AUTHOR};
use engyls::config::parse_color_to_rgba;
use pango::FontDescription;
use pangocairo::functions as pc;
use crate::events::HoverTarget;

pub fn draw_author(cr: &cairo::Context, s: &mut State) {
    let a = &s.args.appearance;
    let (r, g, b, alpha) = parse_color_to_rgba(&a.text_color);
    let ax = a.author_x as f64;
    let ay = a.author_y as f64;

    let is_dragging_a = s.drag_mode == crate::events::DragMode::MoveAuthor;
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

    if a.bg_enabled {
        let (br, bg, bb, ba) = parse_color_to_rgba(&a.bg_color);
        let padding_h = 10.0;
        let padding_v = 4.0;
        let radius = 6.0;

        cr.save().unwrap();
        cr.push_group();
        cr.set_source_rgba(br, bg, bb, 1.0);

        let mut iter = author_layout.iter();
        loop {
            let (_, logical) = iter.line_extents();
            let (ink, _) = iter.line_readonly().unwrap().extents();
            
            let lx = (logical.x() as f64) / pango::SCALE as f64;
            let ly = (logical.y() as f64) / pango::SCALE as f64;
            let lw = (ink.width() as f64) / pango::SCALE as f64;
            let lh = (logical.height() as f64) / pango::SCALE as f64;

            let bx = ax + lx - padding_h;
            let by = ay + ly - padding_v;
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
        cr.paint_with_alpha(ba as f64).unwrap();
        cr.restore().unwrap();
    }

    if a.stroke_enabled {
        cr.move_to(ax, ay);
        let (sr, sg, sb, sa) = parse_color_to_rgba(&a.stroke_color);
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
