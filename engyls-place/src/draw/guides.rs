use crate::state::State;
use crate::events::HoverTarget;
use crate::draw::HANDLE_SIZE;

pub fn draw_container_outline(cr: &cairo::Context, s: &mut State) {
    let a = &s.args.appearance;
    let qx = a.quote_x as f64;
    let qy = a.quote_y as f64;
    let qw = a.quote_max_width as f64;
    let qh = a.quote_max_height as f64;

    let is_hovered_q = matches!(s.hover,
        HoverTarget::QuoteBody | HoverTarget::QuoteResizeRight |
        HoverTarget::QuoteResizeBottom);
    
    let is_resizing = matches!(s.drag_mode,
        crate::events::DragMode::ResizeWidth | crate::events::DragMode::ResizeHeight);

    // Outline
    cr.set_source_rgba(0.4, 0.7, 1.0, if is_hovered_q || is_resizing { 0.7 } else { 0.25 });
    cr.set_line_width(if is_hovered_q || is_resizing { 2.0 } else { 1.0 });
    cr.set_dash(&[6.0, 4.0], 0.0);
    cr.rectangle(qx, qy, qw, qh);
    cr.stroke().unwrap();
    cr.set_dash(&[], 0.0);

    // Handles
    if is_hovered_q || is_resizing {
        cr.set_source_rgba(0.3, 0.8, 1.0, 0.9);

        // Right edge
        let rh_x = qx + qw - HANDLE_SIZE / 2.0;
        let rh_y = qy + qh / 2.0 - 15.0;
        cr.rectangle(rh_x, rh_y, HANDLE_SIZE, 30.0);
        cr.fill().unwrap();

        // Bottom edge
        let bh_x = qx + qw / 2.0 - 15.0;
        let bh_y = qy + qh - HANDLE_SIZE / 2.0;
        cr.rectangle(bh_x, bh_y, 30.0, HANDLE_SIZE);
        cr.fill().unwrap();
    }
}

pub fn draw_alignment_guides(cr: &cairo::Context, s: &State) {
    let is_dragging_q = s.drag_mode == crate::events::DragMode::MoveQuote;
    let is_dragging_a = s.drag_mode == crate::events::DragMode::MoveAuthor;

    if is_dragging_q || is_dragging_a {
        cr.save().unwrap();
        cr.set_source_rgba(1.0, 0.2, 0.2, 0.8); // Brighter red
        cr.set_line_width(2.0);
        cr.set_dash(&[8.0, 4.0], 0.0);
        
        // Vertical Center
        cr.move_to(s.screen_width / 2.0, 0.0);
        cr.line_to(s.screen_width / 2.0, s.screen_height);
        cr.stroke().unwrap();
        
        // Left margin (25px)
        cr.move_to(25.0, 0.0);
        cr.line_to(25.0, s.screen_height);
        cr.stroke().unwrap();
        
        // Right margin (25px)
        cr.move_to(s.screen_width - 25.0, 0.0);
        cr.line_to(s.screen_width - 25.0, s.screen_height);
        cr.stroke().unwrap();

        // Horizontal Top/Bottom margins (25px)
        cr.move_to(0.0, 25.0);
        cr.line_to(s.screen_width, 25.0);
        cr.stroke().unwrap();

        cr.move_to(0.0, s.screen_height - 25.0);
        cr.line_to(s.screen_width, s.screen_height - 25.0);
        cr.stroke().unwrap();
        
        cr.restore().unwrap();
    }
}
