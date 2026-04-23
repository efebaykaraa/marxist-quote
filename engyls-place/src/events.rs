use crate::state::State;
use crate::draw::HANDLE_SIZE;

const HIT_MARGIN: f64 = 15.0;

#[derive(Clone, Copy, PartialEq)]
pub enum HoverTarget {
    None,
    QuoteBody,
    QuoteResizeRight,
    QuoteResizeBottom,
    AuthorBody,
}

#[derive(Clone, Copy, PartialEq)]
pub enum DragMode {
    None,
    MoveQuote,
    MoveAuthor,
    ResizeWidth,
    ResizeHeight,
}

pub fn hit_test(s: &State, x: f64, y: f64) -> HoverTarget {
    let a = &s.args.appearance;
    let qx = a.quote_x as f64;
    let qy = a.quote_y as f64;
    let qw = a.quote_max_width as f64;
    let qh = a.quote_max_height as f64;

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

pub fn snap_x(x: i32, width: i32, sw: f64) -> i32 {
    let threshold = 15.0;
    let center_x = x as f64 + (width as f64 / 2.0);
    let screen_center = sw / 2.0;
    
    // Snap center
    if (center_x - screen_center).abs() < threshold {
        return (screen_center - (width as f64 / 2.0)) as i32;
    }
    
    // Snap left (25px padding)
    if (x as f64 - 25.0).abs() < threshold {
        return 25;
    }
    
    // Snap right (25px padding)
    if (x as f64 + width as f64 - (sw - 25.0)).abs() < threshold {
        return (sw - 25.0 - width as f64) as i32;
    }
    
    x
}

pub fn snap_to_line(raw: i32, line_height: i32) -> i32 {
    if line_height <= 0 {
        return raw;
    }
    let lines = ((raw as f64) / (line_height as f64)).round() as i32;
    lines * line_height
}
