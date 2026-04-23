pub mod quote;
pub mod author;
pub mod guides;

use crate::state::State;
use pango::FontDescription;
use pangocairo::functions as pc;

pub const HANDLE_SIZE: f64 = 10.0;

pub fn draw_scene(cr: &cairo::Context, s: &mut State) {
    let a = &s.args.appearance;
    let qw = a.quote_max_width as f64;
    let qh = a.quote_max_height as f64;

    // ── Calculate capacity based on 'a's ──
    let a_layout = pc::create_layout(cr);
    let mut a_font = FontDescription::new();
    a_font.set_family(&a.font);
    a_font.set_size((a.font_size as i32) * pango::SCALE);
    a_layout.set_font_description(Some(&a_font));
    a_layout.set_text("a");
    let (aw, ah) = a_layout.pixel_size();
    if aw > 0 && ah > 0 {
        let cols = (qw / aw as f64).floor() as usize;
        let rows = (qh / ah as f64).floor() as usize;
        s.capacity_chars = cols * rows;
        s.line_height = ah;
    }

    // ── Draw Layers ──
    guides::draw_container_outline(cr, s);
    quote::draw_quote(cr, s);
    author::draw_author(cr, s);
    guides::draw_alignment_guides(cr, s);
}
